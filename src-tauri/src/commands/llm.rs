use crate::models::{ChecklistItem, LLMSummaryResult};

#[tauri::command]
pub fn summarize_with_llm(
    checklist: Vec<ChecklistItem>,
    problem_summary: String,
) -> Result<LLMSummaryResult, String> {
    // Placeholder - will implement in Phase 3
    Err(String::from("LLM summarization not implemented yet"))
}
