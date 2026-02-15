use crate::error::AppError;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

/// Retry an async operation with exponential backoff
///
/// Attempts: 1 (immediate), 2 (100ms), 3 (200ms), 4 (400ms)
/// Max delay capped at 10s with jitter
pub async fn retry_with_backoff<F, Fut, T>(mut operation: F) -> Result<T, AppError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    let mut attempt = 0;
    let max_attempts = 4;

    loop {
        attempt += 1;

        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= max_attempts => {
                log::error!("Operation failed after {} attempts: {}", max_attempts, e);
                return Err(e);
            }
            Err(e) if !is_retryable(&e) => {
                log::warn!("Non-retryable error, failing immediately: {}", e);
                return Err(e);
            }
            Err(e) => {
                let delay_ms = calculate_backoff(attempt);
                log::warn!(
                    "Attempt {}/{} failed: {}. Retrying in {}ms",
                    attempt,
                    max_attempts,
                    e,
                    delay_ms
                );
                sleep(Duration::from_millis(delay_ms)).await;
            }
        }
    }
}

/// Calculate exponential backoff with jitter
fn calculate_backoff(attempt: u32) -> u64 {
    let base_delay = 100u64;
    let exponential = base_delay * 2u64.pow(attempt.saturating_sub(1));
    let capped = exponential.min(10_000); // Cap at 10s

    // Add jitter (±25%)
    let jitter_range = capped / 4;
    let jitter = (rand::random::<u64>() % jitter_range).saturating_sub(jitter_range / 2);
    capped.saturating_add(jitter)
}

/// Determine if an error is retryable
fn is_retryable(error: &AppError) -> bool {
    match error {
        // HTTP errors
        AppError::Http(e) => {
            // Retry on timeout, connection errors, or 5xx server errors
            e.is_timeout()
                || e.is_connect()
                || e.status()
                    .map(|s| s.is_server_error() || s.as_u16() == 429)
                    .unwrap_or(false)
        }
        // Jira API errors
        AppError::Jira(msg) => {
            msg.contains("429") // Rate limit
                || msg.contains("503") // Service unavailable
                || msg.contains("502") // Bad gateway
                || msg.contains("504") // Gateway timeout
                || msg.contains("timeout")
                || msg.contains("connection")
        }
        // Ollama errors
        AppError::Ollama(msg) => {
            msg.contains("connection")
                || msg.contains("timeout")
                || msg.contains("unavailable")
        }
        // Don't retry these
        AppError::Db(_)
        | AppError::DbSql(_)
        | AppError::Validation(_)
        | AppError::NotFound(_)
        | AppError::TemplateRender(_)
        | AppError::TemplateError(_)
        | AppError::File(_)
        | AppError::Keychain(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_calculation() {
        // Attempt 1: ~100ms
        let backoff1 = calculate_backoff(1);
        assert!(backoff1 >= 75 && backoff1 <= 125);

        // Attempt 2: ~200ms
        let backoff2 = calculate_backoff(2);
        assert!(backoff2 >= 150 && backoff2 <= 250);

        // Attempt 3: ~400ms
        let backoff3 = calculate_backoff(3);
        assert!(backoff3 >= 300 && backoff3 <= 500);

        // Very high attempt: capped at 10s
        let backoff_high = calculate_backoff(20);
        assert!(backoff_high <= 12_500); // 10s + max jitter
    }

    #[test]
    fn test_retryable_errors() {
        // Retryable
        assert!(is_retryable(&AppError::Jira("429 Too Many Requests".into())));
        assert!(is_retryable(&AppError::Jira("503 Service Unavailable".into())));
        assert!(is_retryable(&AppError::Ollama("connection refused".into())));

        // Not retryable
        assert!(!is_retryable(&AppError::Validation("bad input".into())));
        assert!(!is_retryable(&AppError::NotFound("not found".into())));
        assert!(!is_retryable(&AppError::Jira("401 Unauthorized".into())));
    }
}
