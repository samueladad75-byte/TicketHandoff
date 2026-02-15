use crate::error::{AppError, AppResult};
use crate::models::{ChecklistItem, LLMSummaryResult};
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct OllamaClient {
    endpoint: String,
    model: String,
    client: reqwest::Client,
}

impl OllamaClient {
    pub fn new(endpoint: String, model: String) -> AppResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self {
            endpoint,
            model,
            client,
        })
    }

    pub async fn is_available(&self) -> AppResult<bool> {
        let url = format!("{}/api/tags", self.endpoint);

        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub async fn summarize(&self, checklist: &[ChecklistItem], problem: &str) -> AppResult<LLMSummaryResult> {
        // Build the prompt
        let prompt = self.build_prompt(checklist, problem);

        // Call Ollama API
        let url = format!("{}/api/generate", self.endpoint);

        let request_body = OllamaGenerateRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .header(CONTENT_TYPE, "application/json")
            .json(&request_body)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(AppError::Ollama(format!(
                "Ollama API error: {}",
                response.status()
            )));
        }

        let ollama_response: OllamaGenerateResponse = response.json().await?;

        // Calculate confidence based on checklist
        let (confidence, confidence_reason) = self.calculate_confidence(checklist);

        Ok(LLMSummaryResult {
            summary: ollama_response.response,
            confidence,
            confidence_reason,
        })
    }

    fn build_prompt(&self, checklist: &[ChecklistItem], problem: &str) -> String {
        let mut checklist_text = String::new();
        for item in checklist {
            let checkbox = if item.checked { "[x]" } else { "[ ]" };
            checklist_text.push_str(&format!("- {} {}\n", checkbox, item.text));
        }

        format!(
            r#"You are summarizing troubleshooting steps for an L2 support engineer.

Given the following problem and checklist of troubleshooting steps, generate a structured summary.

Problem: {}

Troubleshooting checklist:
{}

Generate output in exactly this format:

✓ Completed steps:
- [step description]

✗ Steps not attempted:
- [step description]

? Recommendations for L2:
- [what L2 should investigate next]

Keep it concise. Only include steps from the checklist above. Do not invent steps."#,
            problem, checklist_text
        )
    }

    fn calculate_confidence(&self, checklist: &[ChecklistItem]) -> (String, String) {
        let total = checklist.len();
        let checked = checklist.iter().filter(|item| item.checked).count();

        if total == 0 {
            return ("Low".to_string(), "No troubleshooting steps provided".to_string());
        }

        let percentage = (checked as f64 / total as f64) * 100.0;

        // Confidence heuristic from plan:
        // High: 5+ items, 60%+ checked
        // Medium: 3-4 items OR <60% checked
        // Low: <3 items
        if total >= 5 && percentage >= 60.0 {
            (
                "High".to_string(),
                format!("Based on {} checklist items, {} completed ({:.0}%)", total, checked, percentage),
            )
        } else if total >= 3 && total <= 4 {
            (
                "Medium".to_string(),
                format!("Based on {} checklist items, {} completed ({:.0}%)", total, checked, percentage),
            )
        } else if total >= 5 && percentage < 60.0 {
            (
                "Medium".to_string(),
                format!("Based on {} checklist items, only {} completed ({:.0}%)", total, checked, percentage),
            )
        } else {
            (
                "Low".to_string(),
                format!("Only {} checklist items provided", total),
            )
        }
    }
}

#[derive(Debug, Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaGenerateResponse {
    response: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_high() {
        let client = OllamaClient::new("http://localhost:11434".to_string(), "llama3".to_string()).unwrap();
        let checklist = vec![
            ChecklistItem { text: "Step 1".to_string(), checked: true },
            ChecklistItem { text: "Step 2".to_string(), checked: true },
            ChecklistItem { text: "Step 3".to_string(), checked: true },
            ChecklistItem { text: "Step 4".to_string(), checked: true },
            ChecklistItem { text: "Step 5".to_string(), checked: false },
            ChecklistItem { text: "Step 6".to_string(), checked: false },
        ];
        let (confidence, _) = client.calculate_confidence(&checklist);
        assert_eq!(confidence, "High");
    }

    #[test]
    fn test_confidence_medium() {
        let client = OllamaClient::new("http://localhost:11434".to_string(), "llama3".to_string()).unwrap();
        let checklist = vec![
            ChecklistItem { text: "Step 1".to_string(), checked: true },
            ChecklistItem { text: "Step 2".to_string(), checked: false },
            ChecklistItem { text: "Step 3".to_string(), checked: false },
        ];
        let (confidence, _) = client.calculate_confidence(&checklist);
        assert_eq!(confidence, "Medium");
    }

    #[test]
    fn test_confidence_low() {
        let client = OllamaClient::new("http://localhost:11434".to_string(), "llama3".to_string()).unwrap();
        let checklist = vec![
            ChecklistItem { text: "Step 1".to_string(), checked: true },
        ];
        let (confidence, _) = client.calculate_confidence(&checklist);
        assert_eq!(confidence, "Low");
    }

    #[test]
    fn test_prompt_formatting() {
        let client = OllamaClient::new("http://localhost:11434".to_string(), "llama3".to_string()).unwrap();
        let checklist = vec![
            ChecklistItem { text: "Restarted VPN".to_string(), checked: true },
            ChecklistItem { text: "Checked logs".to_string(), checked: false },
        ];
        let prompt = client.build_prompt(&checklist, "VPN connection fails");
        assert!(prompt.contains("VPN connection fails"));
        assert!(prompt.contains("[x] Restarted VPN"));
        assert!(prompt.contains("[ ] Checked logs"));
    }
}
