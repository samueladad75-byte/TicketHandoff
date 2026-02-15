use crate::commands::settings::get_jira_client;
use crate::models::JiraTicket;
use tauri::AppHandle;

#[tauri::command]
pub async fn fetch_jira_ticket(app: AppHandle, ticket_id: String) -> Result<JiraTicket, String> {
    fetch_jira_ticket_impl(app, ticket_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn post_to_jira(app: AppHandle, ticket_id: String, comment: String) -> Result<(), String> {
    post_to_jira_impl(app, ticket_id, comment)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn attach_files_to_jira(
    _app: AppHandle,
    _ticket_id: String,
    _file_paths: Vec<String>,
) -> Result<(), String> {
    // Phase 4 implementation - not yet implemented
    Err(String::from("File attachment not implemented yet"))
}

async fn fetch_jira_ticket_impl(
    app: AppHandle,
    ticket_id: String,
) -> Result<JiraTicket, Box<dyn std::error::Error>> {
    let client = get_jira_client(app).await?;
    let ticket = client.fetch_issue(&ticket_id).await?;
    Ok(ticket)
}

async fn post_to_jira_impl(
    app: AppHandle,
    ticket_id: String,
    comment: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = get_jira_client(app).await?;
    client.post_comment(&ticket_id, &comment).await?;
    Ok(())
}
