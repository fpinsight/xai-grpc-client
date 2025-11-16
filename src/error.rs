//! Error types for the xAI gRPC client.
//!
//! This module defines all errors that can occur when using the Grok API client,
//! including network errors, authentication failures, rate limiting, and invalid requests.

use thiserror::Error;

/// Errors that can occur when using the Grok API client.
///
/// This enum covers all possible error conditions including transport failures,
/// API errors, rate limiting, and configuration issues.
///
/// # Examples
///
/// ```no_run
/// use xai_grpc_client::{GrokClient, GrokError};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = GrokClient::from_env().await;
///
/// match client {
///     Ok(_) => println!("Connected successfully"),
///     Err(GrokError::Auth(msg)) => println!("Authentication failed: {}", msg),
///     Err(GrokError::RateLimit { retry_after_secs }) => {
///         println!("Rate limited, retry after {} seconds", retry_after_secs)
///     }
///     Err(e) => println!("Error: {}", e),
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Error, Debug)]
pub enum GrokError {
    /// gRPC transport layer error (network issues, connection failures).
    #[error("gRPC transport error: {0}")]
    Transport(#[from] tonic::transport::Error),

    /// gRPC status error returned by the server.
    #[error("gRPC status error: {0}")]
    Status(#[from] tonic::Status),

    /// Rate limit exceeded. The client should wait before retrying.
    #[error("Rate limit exceeded, retry after {retry_after_secs} seconds")]
    RateLimit {
        /// Number of seconds to wait before retrying.
        retry_after_secs: u64,
    },

    /// Authentication failed due to invalid or missing API key.
    #[error("Authentication failed: {0}")]
    Auth(String),

    /// Invalid request parameters or malformed request.
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Configuration error (e.g., missing required settings).
    #[error("Configuration error: {0}")]
    Config(String),

    /// Required environment variable not set.
    #[error("Environment variable not set: {0}")]
    EnvVar(#[from] std::env::VarError),

    /// Invalid metadata/header value.
    #[error("Invalid header value: {0}")]
    InvalidHeaderValue(#[from] tonic::metadata::errors::InvalidMetadataValue),
}

/// Result type alias using [`GrokError`].
///
/// This is a convenience alias for `Result<T, GrokError>`.
pub type Result<T> = std::result::Result<T, GrokError>;

impl GrokError {
    /// Returns true if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Transport(_) => true,
            Self::RateLimit { .. } => true, // Retryable after delay
            Self::Status(status) => matches!(
                status.code(),
                tonic::Code::Unavailable
                    | tonic::Code::DeadlineExceeded
                    | tonic::Code::ResourceExhausted
            ),
            _ => false,
        }
    }

    /// Get retry delay in seconds if this is a rate limit error
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Self::RateLimit { retry_after_secs } => Some(*retry_after_secs),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_retryable_rate_limit_first() {
        // Test rate limit first since transport error construction is complex
        let error = GrokError::RateLimit {
            retry_after_secs: 60,
        };
        assert!(error.is_retryable());
        assert_eq!(error.retry_after(), Some(60));
    }

    #[test]
    fn test_is_retryable_rate_limit() {
        let error = GrokError::RateLimit {
            retry_after_secs: 60,
        };
        assert!(error.is_retryable());
        assert_eq!(error.retry_after(), Some(60));
    }

    #[test]
    fn test_is_retryable_status_unavailable() {
        let status = tonic::Status::unavailable("service unavailable");
        let error = GrokError::Status(status);
        assert!(error.is_retryable());
    }

    #[test]
    fn test_is_retryable_status_deadline() {
        let status = tonic::Status::deadline_exceeded("timeout");
        let error = GrokError::Status(status);
        assert!(error.is_retryable());
    }

    #[test]
    fn test_is_retryable_status_resource_exhausted() {
        let status = tonic::Status::resource_exhausted("quota exceeded");
        let error = GrokError::Status(status);
        assert!(error.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_auth() {
        let error = GrokError::Auth("invalid api key".to_string());
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_invalid_request() {
        let error = GrokError::InvalidRequest("bad parameters".to_string());
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_config() {
        let error = GrokError::Config("missing config".to_string());
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_is_not_retryable_status_permission_denied() {
        let status = tonic::Status::permission_denied("forbidden");
        let error = GrokError::Status(status);
        assert!(!error.is_retryable());
    }

    #[test]
    fn test_retry_after_none_for_non_rate_limit() {
        let error = GrokError::Config("test".to_string());
        assert_eq!(error.retry_after(), None);
    }

    #[test]
    fn test_error_display() {
        let error = GrokError::Auth("test auth error".to_string());
        assert_eq!(error.to_string(), "Authentication failed: test auth error");

        let error = GrokError::RateLimit {
            retry_after_secs: 120,
        };
        assert_eq!(
            error.to_string(),
            "Rate limit exceeded, retry after 120 seconds"
        );
    }
}
