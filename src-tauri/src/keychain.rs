use crate::error::{AppError, AppResult};
use security_framework::passwords::{delete_generic_password, get_generic_password, set_generic_password};

const SERVICE_NAME: &str = "com.tickethandoff.jira";

/// Save Jira credentials to macOS Keychain
pub fn save_jira_credentials(base_url: &str, email: &str, token: &str) -> AppResult<()> {
    // Encode base_url and token together, using email as account identifier
    let password = format!("{}||{}", base_url, token);

    set_generic_password(SERVICE_NAME, email, password.as_bytes())
        .map_err(|e| AppError::Keychain(format!("Failed to save credentials: {}", e)))?;

    Ok(())
}

/// Retrieve Jira credentials from macOS Keychain
pub fn get_jira_credentials(email: &str) -> AppResult<(String, String)> {
    let password_bytes = get_generic_password(SERVICE_NAME, email)
        .map_err(|e| AppError::Keychain(format!("Failed to retrieve credentials: {}", e)))?;

    let password = String::from_utf8(password_bytes)
        .map_err(|e| AppError::Keychain(format!("Invalid credential data: {}", e)))?;

    let parts: Vec<&str> = password.split("||").collect();
    if parts.len() != 2 {
        return Err(AppError::Keychain("Corrupted credential data".into()));
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Delete Jira credentials from macOS Keychain
pub fn delete_jira_credentials(email: &str) -> AppResult<()> {
    delete_generic_password(SERVICE_NAME, email)
        .map_err(|e| AppError::Keychain(format!("Failed to delete credentials: {}", e)))?;

    Ok(())
}

/// Check if credentials exist in keychain
pub fn credentials_exist(email: &str) -> bool {
    get_generic_password(SERVICE_NAME, email).is_ok()
}
