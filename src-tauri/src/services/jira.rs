use crate::error::{AppError, AppResult};
use crate::models::{JiraComment, JiraTicket, JiraUser};
use crate::services::ticket_system::TicketSystemClient;
use async_trait::async_trait;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde::Deserialize;
use std::time::Duration;

pub struct JiraClient {
    base_url: String,
    email: String,
    api_token: String,
    client: reqwest::Client,
}

impl JiraClient {
    pub fn new(base_url: String, email: String, api_token: String) -> AppResult<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()?;

        Ok(Self {
            base_url,
            email,
            api_token,
            client,
        })
    }

    fn auth_header(&self) -> String {
        let credentials = format!("{}:{}", self.email, self.api_token);
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, credentials.as_bytes());
        format!("Basic {}", encoded)
    }

    pub async fn fetch_issue(&self, key: &str) -> AppResult<JiraTicket> {
        let url = format!(
            "{}/rest/api/3/issue/{}?fields=summary,description,status,reporter,assignee,comment",
            self.base_url, key
        );

        let response = self
            .client
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
            .client
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

    pub async fn test_connection(&self) -> AppResult<String> {
        let url = format!("{}/rest/api/3/myself", self.base_url);

        let response = self
            .client
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
