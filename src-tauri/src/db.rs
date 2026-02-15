use crate::error::{AppError, AppResult};
use crate::models::{ApiConfig, ChecklistItem};
use once_cell::sync::Lazy;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use std::sync::Mutex;

type DbPool = r2d2::Pool<SqliteConnectionManager>;
type PooledConnection = r2d2::PooledConnection<SqliteConnectionManager>;

static DB_POOL: Lazy<Mutex<Option<DbPool>>> = Lazy::new(|| Mutex::new(None));

pub fn init_db(db_path: &str) -> AppResult<()> {
    // Create connection pool
    let manager = SqliteConnectionManager::file(db_path);
    let pool = r2d2::Pool::builder()
        .max_size(15)
        .build(manager)
        .map_err(|e| AppError::Db(format!("Failed to create pool: {}", e).into()))?;

    // Get a connection for migrations
    let conn = pool
        .get()
        .map_err(|e| AppError::Db(format!("Failed to get connection: {}", e).into()))?;

    // Run migrations
    run_migrations(&conn)?;

    // Release connection
    drop(conn);

    // Store pool globally
    let mut pool_guard = DB_POOL
        .lock()
        .map_err(|_| AppError::Db("Pool lock poisoned".into()))?;
    *pool_guard = Some(pool);

    // Seed templates if empty
    seed_templates()?;

    Ok(())
}

fn run_migrations(conn: &rusqlite::Connection) -> AppResult<()> {
    // Create schema_migrations table if it doesn't exist
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
        [],
    )?;

    // Check which migrations have been applied
    let applied_version: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    // Apply migration 001 if needed
    if applied_version < 1 {
        let migration_001 = include_str!("../migrations/001_init.sql");
        conn.execute_batch(migration_001)?;
        conn.execute("INSERT INTO schema_migrations (version) VALUES (1)", [])?;
    }

    // Apply migration 002 if needed
    if applied_version < 2 {
        let migration_002 = include_str!("../migrations/002_security.sql");
        conn.execute_batch(migration_002)?;
        // Note: 002_security.sql inserts its own version record
    }

    Ok(())
}

pub fn seed_templates() -> AppResult<()> {
    let conn = get_connection()?;

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

pub fn get_connection() -> AppResult<PooledConnection> {
    let pool_guard = DB_POOL
        .lock()
        .map_err(|_| AppError::Db("Pool lock poisoned".into()))?;

    pool_guard
        .as_ref()
        .ok_or(AppError::Db("Database not initialized".into()))?
        .get()
        .map_err(|e| AppError::Db(e.to_string().into()))
}

pub fn save_api_config(config: &ApiConfig) -> AppResult<()> {
    let conn = get_connection()?;

    // Save email and Ollama config to database (Jira base_url and token go to keychain)
    conn.execute(
        "INSERT OR REPLACE INTO api_config (id, jira_email, ollama_endpoint, ollama_model, updated_at)
         VALUES (1, ?, ?, ?, datetime('now'))",
        params![config.jira_email, config.ollama_endpoint, config.ollama_model],
    )?;

    Ok(())
}

pub fn get_api_config() -> AppResult<Option<ApiConfig>> {
    let conn = get_connection()?;

    // Get email and Ollama config from database
    let result = conn.query_row(
        "SELECT jira_email, ollama_endpoint, ollama_model FROM api_config WHERE id = 1",
        [],
        |row| {
            Ok(ApiConfig {
                jira_base_url: String::new(), // Placeholder, will be filled from keychain
                jira_email: row.get(0)?,
                jira_api_token: String::new(), // Placeholder, will be filled from keychain
                ollama_endpoint: row.get(1)?,
                ollama_model: row.get(2)?,
            })
        },
    );

    match result {
        Ok(config) => Ok(Some(config)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::DbSql(e)),
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
