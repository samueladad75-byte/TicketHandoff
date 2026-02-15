use crate::error::{AppError, AppResult};
use crate::models::{ApiConfig, ChecklistItem};
use rusqlite::{Connection, params};
use std::sync::Mutex;
use once_cell::sync::Lazy;

static DB_CONNECTION: Lazy<Mutex<Option<Connection>>> = Lazy::new(|| Mutex::new(None));

pub fn init_db(db_path: &str) -> AppResult<()> {
    let conn = Connection::open(db_path)?;

    // Run migrations
    let migration = include_str!("../migrations/001_init.sql");
    conn.execute_batch(migration)?;

    // Store connection
    let mut db = DB_CONNECTION.lock().unwrap();
    *db = Some(conn);

    // Seed templates if empty
    seed_templates()?;

    Ok(())
}

pub fn seed_templates() -> AppResult<()> {
    let mut db_guard = get_connection()?;
    let conn = db_guard.as_mut().ok_or(AppError::Db(rusqlite::Error::InvalidQuery))?;

    // Check if templates already exist
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM templates", [], |row| row.get(0))?;
    if count > 0 {
        return Ok(());
    }

    // Load and insert seed templates
    let templates_json = vec![
        include_str!("../../assets/templates/network-vpn.json"),
        include_str!("../../assets/templates/app-crash.json"),
        include_str!("../../assets/templates/access-permissions.json"),
    ];

    for template_json in templates_json {
        #[derive(serde::Deserialize)]
        struct TemplateJson {
            name: String,
            description: String,
            category: String,
            checklist_items: Vec<ChecklistItem>,
            l2_team: Option<String>,
        }

        let template: TemplateJson = serde_json::from_str(template_json)
            .map_err(|e| AppError::Validation(format!("Failed to parse template: {}", e)))?;

        let checklist_json = serde_json::to_string(&template.checklist_items)
            .map_err(|e| AppError::Validation(format!("Failed to serialize checklist: {}", e)))?;

        conn.execute(
            "INSERT INTO templates (name, description, category, checklist_items, l2_team) VALUES (?, ?, ?, ?, ?)",
            params![
                template.name,
                template.description,
                template.category,
                checklist_json,
                template.l2_team,
            ],
        )?;
    }

    Ok(())
}

pub fn get_connection() -> AppResult<std::sync::MutexGuard<'static, Option<Connection>>> {
    Ok(DB_CONNECTION.lock().unwrap())
}

pub fn save_api_config(config: &ApiConfig) -> AppResult<()> {
    let mut db_guard = get_connection()?;
    let conn = db_guard.as_mut().ok_or(AppError::Db(rusqlite::Error::InvalidQuery))?;

    // Use INSERT OR REPLACE with id=1 to ensure only one config exists
    conn.execute(
        "INSERT OR REPLACE INTO api_config (id, jira_base_url, jira_email, jira_api_token, ollama_endpoint, ollama_model, updated_at)
         VALUES (1, ?, ?, ?, ?, ?, datetime('now'))",
        params![
            config.jira_base_url,
            config.jira_email,
            config.jira_api_token,
            config.ollama_endpoint,
            config.ollama_model,
        ],
    )?;

    Ok(())
}

pub fn get_api_config() -> AppResult<Option<ApiConfig>> {
    let mut db_guard = get_connection()?;
    let conn = db_guard.as_mut().ok_or(AppError::Db(rusqlite::Error::InvalidQuery))?;

    let result = conn.query_row(
        "SELECT jira_base_url, jira_email, jira_api_token, ollama_endpoint, ollama_model FROM api_config WHERE id = 1",
        [],
        |row| {
            Ok(ApiConfig {
                jira_base_url: row.get(0)?,
                jira_email: row.get(1)?,
                jira_api_token: row.get(2)?,
                ollama_endpoint: row.get(3)?,
                ollama_model: row.get(4)?,
            })
        },
    );

    match result {
        Ok(config) => Ok(Some(config)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Db(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_db() {
        let result = init_db(":memory:");
        assert!(result.is_ok());
    }
}
