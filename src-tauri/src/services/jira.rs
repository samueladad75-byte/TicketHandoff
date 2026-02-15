use crate::error::{AppError, AppResult};
use crate::models::{JiraComment, JiraTicket, JiraUser};
use crate::services::retry::retry_with_backoff;
use crate::services::ticket_system::TicketSystemClient;
use async_trait::async_trait;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;

pub struct JiraClient {
    base_url: String,
    email: String,
    api_token: String,
    default_client: reqwest::Client,
    upload_client: reqwest::Client,
}

impl JiraClient {
    pub fn new(base_url: String, email: String, api_token: String) -> AppResult<Self> {
        // Standard operations: 10s timeout
        let default_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        // File uploads: 5 minute timeout
        let upload_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(300))
            .build()?;

        Ok(Self {
            base_url,
            email,
            api_token,
            default_client,
            upload_client,
        })
    }

    fn auth_header(&self) -> String {
        let credentials = format!("{}:{}", self.email, self.api_token);
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, credentials.as_bytes());
        format!("Basic {}", encoded)
    }

    pub async fn fetch_issue(&self, key: &str) -> AppResult<JiraTicket> {
        retry_with_backoff(|| self.fetch_issue_impl(key)).await
    }

    async fn fetch_issue_impl(&self, key: &str) -> AppResult<JiraTicket> {
        let url = format!(
            "{}/rest/api/3/issue/{}?fields=summary,description,status,reporter,assignee,comment",
            self.base_url, key
        );

        let response = self
            .default_client
            .get(&url)
            .header(AUTHORIZATION, self.auth_header())
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?;

        let status = response.status();
        if status == 401 {
            return Err(AppError::Jira("Invalid credentials".to_string()));
        } else if status == 404 {
            return Err(AppError::NotFound(format!("Ticket {} not found", key)));
        } else if status == 429 {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("60");
            return Err(AppError::Jira(format!(
                "Rate limited, retry in {} seconds",
                retry_after
            )));
        } else if !status.is_success() {
            return Err(AppError::Jira(format!("Jira server error: {}", status)));
        }

        let jira_response: JiraIssueResponse = response.json().await?;

        Ok(JiraTicket {
            key: jira_response.key,
            summary: jira_response.fields.summary,
            description: jira_response.fields.description,
            status: jira_response.fields.status.name,
            reporter: jira_response.fields.reporter.map(|r| JiraUser {
                display_name: r.display_name,
                email: r.email_address,
            }),
            assignee: jira_response.fields.assignee.map(|a| JiraUser {
                display_name: a.display_name,
                email: a.email_address,
            }),
            comments: jira_response
                .fields
                .comment
                .comments
                .into_iter()
                .map(|c| JiraComment {
                    author: c.author.display_name,
                    body: c.body,
                    created: c.created,
                })
                .collect(),
        })
    }

    pub async fn post_comment(&self, key: &str, body: &str) -> AppResult<()> {
        retry_with_backoff(|| self.post_comment_impl(key, body)).await
    }

    async fn post_comment_impl(&self, key: &str, body: &str) -> AppResult<()> {
        let url = format!("{}/rest/api/3/issue/{}/comment", self.base_url, key);

        // Wrap plain text in minimal ADF structure
        let adf_body = serde_json::json!({
            "body": {
                "type": "doc",
                "version": 1,
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": body
                    }]
                }]
            }
        });

        let response = self
            .default_client
            .post(&url)
            .header(AUTHORIZATION, self.auth_header())
            .header(CONTENT_TYPE, "application/json")
            .json(&adf_body)
            .send()
            .await?;

        let status = response.status();
        if status == 403 {
            return Err(AppError::Jira(format!(
                "No permission to comment on {}. Check your API token permissions.",
                key
            )));
        } else if !status.is_success() {
            return Err(AppError::Jira(format!("Failed to post comment: {}", status)));
        }

        Ok(())
    }

    pub async fn attach_file(&self, key: &str, file_path: &Path) -> AppResult<()> {
        retry_with_backoff(|| self.attach_file_impl(key, file_path)).await
    }

    async fn attach_file_impl(&self, key: &str, file_path: &Path) -> AppResult<()> {
        // Validate file exists and size
        let metadata = tokio::fs::metadata(file_path)
            .await
            .map_err(|_| AppError::File(format!("File not found: {}", file_path.display())))?;

        let size_mb = metadata.len() / (1024 * 1024);
        if size_mb > 100 {
            return Err(AppError::File(format!(
                "File too large ({}MB). Jira limit is 100MB.",
                size_mb
            )));
        }

        let url = format!("{}/rest/api/3/issue/{}/attachments", self.base_url, key);

        // Read file asynchronously (still better than blocking I/O)
        let file_bytes = tokio::fs::read(file_path).await?;

        let file_name = file_path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| AppError::File("Invalid file name".to_string()))?;

        let part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(file_name.to_string())
            .mime_str("application/octet-stream")
            .map_err(|e| AppError::Jira(format!("Failed to create multipart: {}", e)))?;

        let form = reqwest::multipart::Form::new().part("file", part);

        let response = self
            .upload_client // Use upload client with 300s timeout
            .post(&url)
            .header(AUTHORIZATION, self.auth_header())
            .header("X-Atlassian-Token", "no-check") // Required by Jira
            .multipart(form)
            .send()
            .await?;

        let status = response.status();
        if status == 403 {
            return Err(AppError::Jira(format!(
                "No permission to attach files to {}. Check your API token permissions.",
                key
            )));
        } else if status == 413 {
            return Err(AppError::Jira(format!(
                "File rejected by Jira (too large: {}MB). Try compressing it.",
                size_mb
            )));
        } else if !status.is_success() {
            return Err(AppError::Jira(format!("Failed to attach file: {}", status)));
        }

        Ok(())
    }

    pub async fn test_connection(&self) -> AppResult<String> {
        let url = format!("{}/rest/api/3/myself", self.base_url);

        let response = self
            .default_client
            .get(&url)
            .header(AUTHORIZATION, self.auth_header())
            .header(CONTENT_TYPE, "application/json")
            .send()
            .await?;

        let status = response.status();
        if status == 401 {
            return Err(AppError::Jira("Invalid credentials".to_string()));
        } else if !status.is_success() {
            return Err(AppError::Jira(format!("Connection test failed: {}", status)));
        }

        let myself: JiraMyselfResponse = response.json().await?;
        Ok(myself.display_name)
    }
}

// Jira API response structures
#[derive(Debug, Deserialize)]
struct JiraIssueResponse {
    key: String,
    fields: JiraFields,
}

#[derive(Debug, Deserialize)]
struct JiraFields {
    summary: String,
    description: Option<String>,
    status: JiraStatus,
    reporter: Option<JiraUserResponse>,
    assignee: Option<JiraUserResponse>,
    comment: JiraComments,
}

#[derive(Debug, Deserialize)]
struct JiraStatus {
    name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JiraUserResponse {
    display_name: String,
    email_address: Option<String>,
}

#[derive(Debug, Deserialize)]
struct JiraComments {
    comments: Vec<JiraCommentResponse>,
}

#[derive(Debug, Deserialize)]
struct JiraCommentResponse {
    author: JiraUserResponse,
    body: String,
    created: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JiraMyselfResponse {
    display_name: String,
}

#[async_trait]
impl TicketSystemClient for JiraClient {
    async fn fetch_ticket(&self, id: &str) -> AppResult<JiraTicket> {
        self.fetch_issue(id).await
    }

    async fn post_comment(&self, id: &str, body: &str) -> AppResult<()> {
        self.post_comment(id, body).await
    }

    async fn test_connection(&self) -> AppResult<String> {
        self.test_connection().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_header() {
        let client = JiraClient::new(
            "https://test.atlassian.net".to_string(),
            "test@example.com".to_string(),
            "token123".to_string(),
        )
        .unwrap();

        let auth = client.auth_header();
        assert!(auth.starts_with("Basic "));
    }
}
