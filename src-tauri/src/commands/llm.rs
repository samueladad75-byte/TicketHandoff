use crate::db;
use crate::models::{ChecklistItem, LLMSummaryResult};
use crate::services::ollama::OllamaClient;

#[tauri::command]
pub async fn summarize_with_llm(
    checklist: Vec<ChecklistItem>,
    problem_summary: String,
) -> Result<LLMSummaryResult, String> {
    summarize_with_llm_impl(checklist, problem_summary)
        .await
        .map_err(|e| e.to_string())
}

async fn summarize_with_llm_impl(
    checklist: Vec<ChecklistItem>,
    problem_summary: String,
) -> Result<LLMSummaryResult, Box<dyn std::error::Error>> {
    // Get Ollama config from database
    let config = db::get_api_config()?
        .ok_or("No API config found. Please configure Ollama in Settings.")?;

    // Create Ollama client
    let client = OllamaClient::new(config.ollama_endpoint, config.ollama_model)?;

    // Check if Ollama is available
    if !client.is_available().await? {
        return Err("Ollama is not running. Start it with `ollama serve` or skip the AI summary.".into());
    }

    // Generate summary
    let result = client.summarize(&checklist, &problem_summary).await?;

    Ok(result)
}
