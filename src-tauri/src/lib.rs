mod commands;
mod db;
mod error;
mod keychain;
mod models;
mod services;

use commands::{escalations, llm, settings, templates, tickets};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize database with proper error handling
            let app_data_dir = app.handle().path().app_data_dir()
                .map_err(|e| format!("Cannot access app data directory: {}", e))?;

            std::fs::create_dir_all(&app_data_dir)
                .map_err(|e| format!("Cannot create app directory: {}. Check disk permissions.", e))?;

            let db_path = app_data_dir.join("tickets.db");
            let db_path_str = db_path.to_str()
                .ok_or("Invalid database path with non-UTF8 characters")?;

            db::init_db(db_path_str)
                .map_err(|e| format!("Database initialization failed: {}\n\nPlease restart the app or check permissions.", e))?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            templates::list_templates,
            templates::get_template,
            escalations::save_escalation,
            escalations::get_escalation,
            escalations::list_escalations,
            escalations::delete_escalation,
            escalations::render_markdown,
            escalations::post_escalation,
            escalations::retry_post_escalation,
            tickets::fetch_jira_ticket,
            tickets::post_to_jira,
            tickets::attach_files_to_jira,
            llm::summarize_with_llm,
            settings::save_api_config,
            settings::get_api_config,
            settings::test_jira_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application")
}
