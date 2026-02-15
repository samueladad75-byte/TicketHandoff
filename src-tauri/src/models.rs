use serde::{Deserialize, Serialize};

// === Templates ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Template {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub category: String,
    pub checklist_items: Vec<ChecklistItem>,
    pub l2_team: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub text: String,
    pub checked: bool,
}

// === Escalations ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Escalation {
    pub id: i64,
    pub ticket_id: String,
    pub template_id: Option<i64>,
    pub problem_summary: String,
    pub checklist: Vec<ChecklistItem>,
    pub current_status: String,
    pub next_steps: String,
    pub llm_summary: Option<String>,
    pub llm_confidence: Option<String>,
    pub markdown_output: Option<String>,
    pub status: EscalationStatus,
    pub posted_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EscalationStatus {
    Draft,
    Posted,
    PostedWithErrors,
    PostFailed,
}

impl EscalationStatus {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            EscalationStatus::Draft => "draft",
            EscalationStatus::Posted => "posted",
            EscalationStatus::PostedWithErrors => "posted_with_errors",
            EscalationStatus::PostFailed => "post_failed",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "posted" => EscalationStatus::Posted,
            "posted_with_errors" => EscalationStatus::PostedWithErrors,
            "post_failed" => EscalationStatus::PostFailed,
            _ => EscalationStatus::Draft,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationSummary {
    pub id: i64,
    pub ticket_id: String,
    pub problem_summary: String,
    pub status: EscalationStatus,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationInput {
    pub ticket_id: String,
    pub template_id: Option<i64>,
    pub problem_summary: String,
    pub checklist: Vec<ChecklistItem>,
    pub current_status: String,
    pub next_steps: String,
    pub llm_summary: Option<String>,
    pub llm_confidence: Option<String>,
}

// === Jira ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraTicket {
    pub key: String,
    pub summary: String,
    pub description: Option<String>,
    pub status: String,
    pub reporter: Option<JiraUser>,
    pub assignee: Option<JiraUser>,
    pub comments: Vec<JiraComment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraUser {
    pub display_name: String,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JiraComment {
    pub author: String,
    pub body: String,
    pub created: String,
}

// === LLM ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LLMSummaryResult {
    pub summary: String,
    pub confidence: String,
    pub confidence_reason: String,
}

// === Settings ===

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub jira_base_url: String,
    pub jira_email: String,
    pub jira_api_token: String,
    pub ollama_endpoint: String,
    pub ollama_model: String,
}
