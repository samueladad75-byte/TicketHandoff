use crate::commands::settings::get_jira_client;
use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{Escalation, EscalationInput, EscalationStatus, EscalationSummary};
use crate::services::template_engine;
use tauri::AppHandle;

#[tauri::command]
pub fn save_escalation(input: EscalationInput) -> Result<i64, String> {
    save_escalation_impl(input).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_escalation(id: i64) -> Result<Escalation, String> {
    get_escalation_impl(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn list_escalations() -> Result<Vec<EscalationSummary>, String> {
    list_escalations_impl().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_escalation(id: i64) -> Result<(), String> {
    delete_escalation_impl(id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn render_markdown(input: EscalationInput) -> Result<String, String> {
    render_markdown_impl(input).map_err(|e| e.to_string())
}

fn save_escalation_impl(input: EscalationInput) -> AppResult<i64> {
    let mut db_guard = db::get_connection()?;
    let conn = db_guard.as_mut().ok_or(AppError::Db(rusqlite::Error::InvalidQuery))?;

    let checklist_json = serde_json::to_string(&input.checklist)
        .map_err(|e| AppError::Validation(format!("Failed to serialize checklist: {}", e)))?;

    let id = conn.query_row(
        "INSERT INTO escalations
        (ticket_id, template_id, problem_summary, checklist, current_status, next_steps, llm_summary, llm_confidence, status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING id",
        rusqlite::params![
            input.ticket_id,
            input.template_id,
            input.problem_summary,
            checklist_json,
            input.current_status,
            input.next_steps,
            input.llm_summary,
            input.llm_confidence,
            "draft",
        ],
        |row| row.get(0),
    )?;

    // Write audit log
    conn.execute(
        "INSERT INTO audit_log (escalation_id, action, details) VALUES (?, ?, ?)",
        rusqlite::params![
            id,
            "created",
            serde_json::to_string(&serde_json::json!({
                "ticket_id": input.ticket_id,
                "template_id": input.template_id,
            })).unwrap_or_default(),
        ],
    )?;

    Ok(id)
}

fn get_escalation_impl(id: i64) -> AppResult<Escalation> {
    let mut db_guard = db::get_connection()?;
    let conn = db_guard.as_mut().ok_or(AppError::Db(rusqlite::Error::InvalidQuery))?;

    let escalation = conn.query_row(
        "SELECT id, ticket_id, template_id, problem_summary, checklist, current_status, next_steps,
        llm_summary, llm_confidence, markdown_output, status, posted_at, created_at, updated_at
        FROM escalations WHERE id = ?",
        [id],
        |row| {
            let checklist_json: String = row.get(4)?;
            let checklist = serde_json::from_str(&checklist_json).unwrap_or_default();
            let status_str: String = row.get(10)?;

            Ok(Escalation {
                id: row.get(0)?,
                ticket_id: row.get(1)?,
                template_id: row.get(2)?,
                problem_summary: row.get(3)?,
                checklist,
                current_status: row.get(5)?,
                next_steps: row.get(6)?,
                llm_summary: row.get(7)?,
                llm_confidence: row.get(8)?,
                markdown_output: row.get(9)?,
                status: EscalationStatus::from_str(&status_str),
                posted_at: row.get(11)?,
                created_at: row.get(12)?,
                updated_at: row.get(13)?,
            })
        },
    )?;

    Ok(escalation)
}

fn list_escalations_impl() -> AppResult<Vec<EscalationSummary>> {
    let mut db_guard = db::get_connection()?;
    let conn = db_guard.as_mut().ok_or(AppError::Db(rusqlite::Error::InvalidQuery))?;

    let mut stmt = conn.prepare(
        "SELECT id, ticket_id, problem_summary, status, created_at
        FROM escalations
        ORDER BY created_at DESC"
    )?;

    let summaries = stmt.query_map([], |row| {
        let status_str: String = row.get(3)?;
        Ok(EscalationSummary {
            id: row.get(0)?,
            ticket_id: row.get(1)?,
            problem_summary: row.get(2)?,
            status: EscalationStatus::from_str(&status_str),
            created_at: row.get(4)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(summaries)
}

fn delete_escalation_impl(id: i64) -> AppResult<()> {
    let mut db_guard = db::get_connection()?;
    let conn = db_guard.as_mut().ok_or(AppError::Db(rusqlite::Error::InvalidQuery))?;

    // Delete audit log entries first (FK constraint)
    conn.execute("DELETE FROM audit_log WHERE escalation_id = ?", [id])?;

    // Delete escalation
    let rows_affected = conn.execute("DELETE FROM escalations WHERE id = ?", [id])?;

    if rows_affected == 0 {
        return Err(AppError::NotFound(format!("Escalation {} not found", id)));
    }

    Ok(())
}

fn render_markdown_impl(input: EscalationInput) -> AppResult<String> {
    // Fetch template if template_id is provided
    let template = if let Some(template_id) = input.template_id {
        let mut db_guard = db::get_connection()?;
        let conn = db_guard.as_mut().ok_or(AppError::Db(rusqlite::Error::InvalidQuery))?;

        let mut stmt = conn.prepare(
            "SELECT id, name, description, category, checklist_items, l2_team FROM templates WHERE id = ?"
        )?;

        stmt.query_row([template_id], |row| {
            let checklist_json: String = row.get(4)?;
            let checklist_items = serde_json::from_str(&checklist_json).unwrap_or_default();

            Ok(crate::models::Template {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
                checklist_items,
                l2_team: row.get(5)?,
            })
        }).ok()
    } else {
        None
    };

    template_engine::render_markdown(template.as_ref(), &input)
}

#[tauri::command]
pub async fn post_escalation(
    app: AppHandle,
    id: i64,
    file_paths: Vec<String>,
) -> Result<(), String> {
    post_escalation_impl(app, id, file_paths)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn retry_post_escalation(
    app: AppHandle,
    id: i64,
    file_paths: Vec<String>,
) -> Result<(), String> {
    retry_post_escalation_impl(app, id, file_paths)
        .await
        .map_err(|e| e.to_string())
}

async fn post_escalation_impl(
    app: AppHandle,
    id: i64,
    file_paths: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load escalation
    let escalation = get_escalation_impl(id)?;

    // Render markdown
    let input = EscalationInput {
        ticket_id: escalation.ticket_id.clone(),
        template_id: escalation.template_id,
        problem_summary: escalation.problem_summary.clone(),
        checklist: escalation.checklist.clone(),
        current_status: escalation.current_status.clone(),
        next_steps: escalation.next_steps.clone(),
        llm_summary: escalation.llm_summary.clone(),
        llm_confidence: escalation.llm_confidence.clone(),
    };
    let markdown = render_markdown_impl(input)?;

    // Get Jira client
    let client = get_jira_client(app).await?;

    // Post comment
    match client.post_comment(&escalation.ticket_id, &markdown).await {
        Ok(_) => {},
        Err(e) => {
            // Update status to post_failed
            update_escalation_status(id, "post_failed", Some(&markdown), Some(&e.to_string()))?;
            return Err(e.into());
        }
    }

    // Upload attachments
    let mut failed_files = Vec::new();
    for file_path in &file_paths {
        let path = std::path::Path::new(file_path);
        if let Err(e) = client.attach_file(&escalation.ticket_id, path).await {
            failed_files.push(format!("{}: {}", file_path, e));
        }
    }

    if !failed_files.is_empty() {
        let error_msg = format!("Failed to attach {} file(s):\n{}", failed_files.len(), failed_files.join("\n"));
        update_escalation_status(id, "post_failed", Some(&markdown), Some(&error_msg))?;
        return Err(error_msg.into());
    }

    // Update status to posted
    update_escalation_status(id, "posted", Some(&markdown), None)?;

    // Write audit log
    write_audit_log(id, "posted", &serde_json::json!({
        "ticket_id": escalation.ticket_id,
        "files_attached": file_paths.len(),
        "had_llm_summary": escalation.llm_summary.is_some(),
    }))?;

    Ok(())
}

async fn retry_post_escalation_impl(
    app: AppHandle,
    id: i64,
    file_paths: Vec<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Load escalation
    let escalation = get_escalation_impl(id)?;

    // Use existing markdown if available, otherwise render
    let markdown = if let Some(existing_markdown) = escalation.markdown_output {
        existing_markdown
    } else {
        let input = EscalationInput {
            ticket_id: escalation.ticket_id.clone(),
            template_id: escalation.template_id,
            problem_summary: escalation.problem_summary.clone(),
            checklist: escalation.checklist.clone(),
            current_status: escalation.current_status.clone(),
            next_steps: escalation.next_steps.clone(),
            llm_summary: escalation.llm_summary.clone(),
            llm_confidence: escalation.llm_confidence.clone(),
        };
        render_markdown_impl(input)?
    };

    // Get Jira client
    let client = get_jira_client(app).await?;

    // Post comment
    match client.post_comment(&escalation.ticket_id, &markdown).await {
        Ok(_) => {},
        Err(e) => {
            update_escalation_status(id, "post_failed", Some(&markdown), Some(&e.to_string()))?;
            return Err(e.into());
        }
    }

    // Upload attachments
    let mut failed_files = Vec::new();
    for file_path in &file_paths {
        let path = std::path::Path::new(file_path);
        if let Err(e) = client.attach_file(&escalation.ticket_id, path).await {
            failed_files.push(format!("{}: {}", file_path, e));
        }
    }

    if !failed_files.is_empty() {
        let error_msg = format!("Failed to attach {} file(s):\n{}", failed_files.len(), failed_files.join("\n"));
        update_escalation_status(id, "post_failed", Some(&markdown), Some(&error_msg))?;
        return Err(error_msg.into());
    }

    // Update status to posted
    update_escalation_status(id, "posted", Some(&markdown), None)?;

    // Write audit log
    write_audit_log(id, "retry_posted", &serde_json::json!({
        "ticket_id": escalation.ticket_id,
        "files_attached": file_paths.len(),
    }))?;

    Ok(())
}

fn update_escalation_status(
    id: i64,
    status: &str,
    markdown_output: Option<&str>,
    error_details: Option<&str>,
) -> AppResult<()> {
    let mut db_guard = db::get_connection()?;
    let conn = db_guard.as_mut().ok_or(AppError::Db(rusqlite::Error::InvalidQuery))?;

    let posted_at = if status == "posted" {
        Some(chrono::Utc::now().to_rfc3339())
    } else {
        None
    };

    conn.execute(
        "UPDATE escalations SET status = ?, markdown_output = ?, posted_at = ?, updated_at = datetime('now') WHERE id = ?",
        rusqlite::params![status, markdown_output, posted_at, id],
    )?;

    // Write audit log for status change
    if let Some(error) = error_details {
        write_audit_log(id, status, &serde_json::json!({
            "error": error,
        }))?;
    }

    Ok(())
}

fn write_audit_log(escalation_id: i64, action: &str, details: &serde_json::Value) -> AppResult<()> {
    let mut db_guard = db::get_connection()?;
    let conn = db_guard.as_mut().ok_or(AppError::Db(rusqlite::Error::InvalidQuery))?;

    conn.execute(
        "INSERT INTO audit_log (escalation_id, action, details) VALUES (?, ?, ?)",
        rusqlite::params![
            escalation_id,
            action,
            serde_json::to_string(details).unwrap_or_default(),
        ],
    )?;

    Ok(())
}
