use crate::models::ApiConfig;

#[tauri::command]
pub fn save_api_config(config: ApiConfig) -> Result<(), String> {
    // Placeholder - will implement in Phase 2
    Err(String::from("Settings not implemented yet"))
}

#[tauri::command]
pub fn get_api_config() -> Result<Option<ApiConfig>, String> {
    // Placeholder - will implement in Phase 2
    Ok(None)
}

#[tauri::command]
pub fn test_jira_connection() -> Result<String, String> {
    // Placeholder - will implement in Phase 2
    Err(String::from("Connection test not implemented yet"))
}
