use crate::db;
use crate::error::{AppError, AppResult};
use crate::models::{ChecklistItem, Template};

#[tauri::command]
pub fn list_templates() -> Result<Vec<Template>, String> {
    list_templates_impl().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_template(id: i64) -> Result<Template, String> {
    get_template_impl(id).map_err(|e| e.to_string())
}

fn list_templates_impl() -> AppResult<Vec<Template>> {
    let mut db_guard = db::get_connection()?;
    let conn = db_guard.as_mut().ok_or(AppError::Db(rusqlite::Error::InvalidQuery))?;

    let mut stmt = conn.prepare(
        "SELECT id, name, description, category, checklist_items, l2_team FROM templates ORDER BY category, name"
    )?;

    let templates = stmt.query_map([], |row| {
        let checklist_json: String = row.get(4)?;
        let checklist_items: Vec<ChecklistItem> = serde_json::from_str(&checklist_json)
            .unwrap_or_default();

        Ok(Template {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            category: row.get(3)?,
            checklist_items,
            l2_team: row.get(5)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;

    Ok(templates)
}

fn get_template_impl(id: i64) -> AppResult<Template> {
    let mut db_guard = db::get_connection()?;
    let conn = db_guard.as_mut().ok_or(AppError::Db(rusqlite::Error::InvalidQuery))?;

    let mut stmt = conn.prepare(
        "SELECT id, name, description, category, checklist_items, l2_team FROM templates WHERE id = ?"
    )?;

    let template = stmt.query_row([id], |row| {
        let checklist_json: String = row.get(4)?;
        let checklist_items: Vec<ChecklistItem> = serde_json::from_str(&checklist_json)
            .unwrap_or_default();

        Ok(Template {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            category: row.get(3)?,
            checklist_items,
            l2_team: row.get(5)?,
        })
    })?;

    Ok(template)
}
