use crate::db;
use crate::models::ApiConfig;
use crate::services::jira::JiraClient;
use tauri::AppHandle;

#[tauri::command]
pub async fn save_api_config(_app: AppHandle, config: ApiConfig) -> Result<(), String> {
    save_api_config_impl(config)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_api_config(_app: AppHandle) -> Result<Option<ApiConfig>, String> {
    get_api_config_impl()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn test_jira_connection(_app: AppHandle) -> Result<String, String> {
    test_jira_connection_impl()
        .await
        .map_err(|e| e.to_string())
}

fn save_api_config_impl(config: ApiConfig) -> Result<(), Box<dyn std::error::Error>> {
    db::save_api_config(&config)?;
    Ok(())
}

fn get_api_config_impl() -> Result<Option<ApiConfig>, Box<dyn std::error::Error>> {
    let config = db::get_api_config()?;
    if let Some(mut cfg) = config {
        // Mask the API token for display
        cfg.jira_api_token = "••••••".to_string();
        Ok(Some(cfg))
    } else {
        Ok(None)
    }
}

fn get_api_config_for_use() -> Result<Option<ApiConfig>, Box<dyn std::error::Error>> {
    Ok(db::get_api_config()?)
}

async fn test_jira_connection_impl() -> Result<String, Box<dyn std::error::Error>> {
    let config = get_api_config_for_use()?
        .ok_or("No API config found. Please configure Jira credentials first.")?;

    let client = JiraClient::new(
        config.jira_base_url,
        config.jira_email,
        config.jira_api_token,
    )?;

    let display_name = client.test_connection().await?;
    Ok(format!("Connected as {}", display_name))
}

// Helper function used by ticket commands
pub async fn get_jira_client(_app: AppHandle) -> Result<JiraClient, Box<dyn std::error::Error>> {
    let config = get_api_config_for_use()?
        .ok_or("No API config found. Please configure Jira credentials in Settings.")?;

    Ok(JiraClient::new(
        config.jira_base_url,
        config.jira_email,
        config.jira_api_token,
    )?)
}
