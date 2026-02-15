use crate::error::AppResult;
use crate::models::JiraTicket;
use async_trait::async_trait;

#[async_trait]
pub trait TicketSystemClient {
    async fn fetch_ticket(&self, id: &str) -> AppResult<JiraTicket>;
    async fn post_comment(&self, id: &str, body: &str) -> AppResult<()>;
    async fn test_connection(&self) -> AppResult<String>;
}
