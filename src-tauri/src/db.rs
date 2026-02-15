use crate::error::{AppError, AppResult};
use crate::models::{ChecklistItem, Template};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_db() {
        let result = init_db(":memory:");
        assert!(result.is_ok());
    }
}
