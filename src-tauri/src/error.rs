use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Db(String),

    #[error("Database SQL error: {0}")]
    DbSql(#[from] rusqlite::Error),

    #[error("Jira API error: {0}")]
    Jira(String),

    #[error("Ollama error: {0}")]
    Ollama(String),

    #[error("Template rendering error: {0}")]
    TemplateRender(#[from] handlebars::RenderError),

    #[error("Template error: {0}")]
    TemplateError(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("File error: {0}")]
    File(String),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Keychain error: {0}")]
    Keychain(String),
}

impl From<AppError> for String {
    fn from(err: AppError) -> String {
        err.to_string()
    }
}

impl From<handlebars::TemplateError> for AppError {
    fn from(err: handlebars::TemplateError) -> Self {
        AppError::TemplateError(err.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::File(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
