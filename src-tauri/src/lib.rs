mod commands;
mod db;
mod error;
mod models;
mod services;

use commands::{escalations, llm, settings, templates, tickets};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::new().build())
        .plugin(tauri_plugin_stronghold::Builder::new(|password| {
            use argon2::{Argon2, PasswordHasher};
            use argon2::password_hash::SaltString;
            let salt = SaltString::from_b64("dGlja2V0LWhhbmRvZmYtc2FsdA").unwrap();
            let argon2 = Argon2::default();
            argon2
                .hash_password(password.as_ref(), &salt)
                .expect("failed to hash password")
                .hash
                .unwrap()
                .as_bytes()
                .to_vec()
        }).build())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Initialize database
            let app_data_dir = app.handle().path().app_data_dir()
                .expect("Failed to get app data directory");
            std::fs::create_dir_all(&app_data_dir)
                .expect("Failed to create app data directory");
            let db_path = app_data_dir.join("tickets.db");
            db::init_db(db_path.to_str().unwrap())
                .expect("Failed to initialize database");
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
        .expect("error while running tauri application");
}
