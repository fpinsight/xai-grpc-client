//! Tokenization API for xAI's Grok models
//!
//! This module provides functionality to tokenize text using Grok models,
//! which is essential for:
//! - Counting tokens for cost estimation
//! - Understanding token boundaries
//! - Debugging prompt construction
//! - Staying within model token limits
//!
//! # Example
//!
//! ```no_run
//! use xai_grpc_client::{GrokClient, TokenizeRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = GrokClient::from_env().await?;
//!
//!     let request = TokenizeRequest::new("grok-2-1212")
//!         .with_text("Hello, world!");
//!
//!     let response = client.tokenize(request).await?;
//!     println!("Token count: {}", response.tokens.len());
//!
//!     for token in response.tokens {
//!         println!("Token: {} (ID: {})", token.string_token, token.token_id);
//!     }
//!
//!     Ok(())
//! }
//! ```

/// A request to tokenize text
///
/// Use the builder pattern to construct requests:
///
/// ```
/// use xai_grpc_client::TokenizeRequest;
///
/// let request = TokenizeRequest::new("grok-2-1212")
///     .with_text("Hello, world!")
///     .with_user("user-123");
/// ```
#[derive(Debug, Clone)]
pub struct TokenizeRequest {
    /// The text to tokenize
    pub text: String,
    /// The model to use for tokenization
    pub model: String,
    /// Optional user identifier for tracking
    pub user: Option<String>,
}

impl TokenizeRequest {
    /// Create a new tokenize request with the specified model
    ///
    /// # Arguments
    ///
    /// * `model` - The model name (e.g., "grok-2-1212")
    ///
    /// # Example
    ///
    /// ```
    /// use xai_grpc_client::TokenizeRequest;
    ///
    /// let request = TokenizeRequest::new("grok-2-1212");
    /// ```
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            text: String::new(),
            model: model.into(),
            user: None,
        }
    }

    /// Set the text to tokenize
    ///
    /// # Arguments
    ///
    /// * `text` - The text to tokenize
    ///
    /// # Example
    ///
    /// ```
    /// use xai_grpc_client::TokenizeRequest;
    ///
    /// let request = TokenizeRequest::new("grok-2-1212")
    ///     .with_text("Hello, world!");
    /// ```
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.text = text.into();
        self
    }

    /// Set the user identifier for tracking
    ///
    /// # Arguments
    ///
    /// * `user` - User identifier
    ///
    /// # Example
    ///
    /// ```
    /// use xai_grpc_client::TokenizeRequest;
    ///
    /// let request = TokenizeRequest::new("grok-2-1212")
    ///     .with_text("Hello!")
    ///     .with_user("user-123");
    /// ```
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }
}

/// A single token from the tokenization response
///
/// Each token contains:
/// - The token ID (vocabulary index)
/// - The string representation
/// - The raw byte representation
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// The token's vocabulary ID
    pub token_id: u32,
    /// The string representation of the token
    pub string_token: String,
    /// The raw bytes of the token
    pub token_bytes: Vec<u8>,
}

/// Response from a tokenization request
///
/// Contains the list of tokens and the model used.
#[derive(Debug, Clone)]
pub struct TokenizeResponse {
    /// The list of tokens
    pub tokens: Vec<Token>,
    /// The model that was used for tokenization
    pub model: String,
}

impl TokenizeResponse {
    /// Get the total number of tokens
    ///
    /// # Example
    ///
    /// ```
    /// # use xai_grpc_client::TokenizeResponse;
    /// # let response = TokenizeResponse { tokens: vec![], model: "grok-2-1212".to_string() };
    /// println!("Token count: {}", response.token_count());
    /// ```
    pub fn token_count(&self) -> usize {
        self.tokens.len()
    }

    /// Get the concatenated text from all tokens
    ///
    /// # Example
    ///
    /// ```
    /// # use xai_grpc_client::TokenizeResponse;
    /// # let response = TokenizeResponse { tokens: vec![], model: "grok-2-1212".to_string() };
    /// println!("Reconstructed text: {}", response.text());
    /// ```
    pub fn text(&self) -> String {
        self.tokens
            .iter()
            .map(|t| t.string_token.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_request_builder() {
        let request = TokenizeRequest::new("grok-2-1212")
            .with_text("Hello, world!")
            .with_user("user-123");

        assert_eq!(request.model, "grok-2-1212");
        assert_eq!(request.text, "Hello, world!");
        assert_eq!(request.user, Some("user-123".to_string()));
    }

    #[test]
    fn test_tokenize_request_minimal() {
        let request = TokenizeRequest::new("grok-2-1212");

        assert_eq!(request.model, "grok-2-1212");
        assert_eq!(request.text, "");
        assert_eq!(request.user, None);
    }

    #[test]
    fn test_tokenize_request_without_user() {
        let request = TokenizeRequest::new("grok-2-1212")
            .with_text("Hello!");

        assert_eq!(request.model, "grok-2-1212");
        assert_eq!(request.text, "Hello!");
        assert_eq!(request.user, None);
    }

    #[test]
    fn test_token_equality() {
        let token1 = Token {
            token_id: 42,
            string_token: "hello".to_string(),
            token_bytes: vec![104, 101, 108, 108, 111],
        };

        let token2 = Token {
            token_id: 42,
            string_token: "hello".to_string(),
            token_bytes: vec![104, 101, 108, 108, 111],
        };

        assert_eq!(token1, token2);
    }

    #[test]
    fn test_tokenize_response_token_count() {
        let response = TokenizeResponse {
            tokens: vec![
                Token {
                    token_id: 1,
                    string_token: "Hello".to_string(),
                    token_bytes: vec![72, 101, 108, 108, 111],
                },
                Token {
                    token_id: 2,
                    string_token: ", ".to_string(),
                    token_bytes: vec![44, 32],
                },
                Token {
                    token_id: 3,
                    string_token: "world".to_string(),
                    token_bytes: vec![119, 111, 114, 108, 100],
                },
            ],
            model: "grok-2-1212".to_string(),
        };

        assert_eq!(response.token_count(), 3);
    }

    #[test]
    fn test_tokenize_response_text() {
        let response = TokenizeResponse {
            tokens: vec![
                Token {
                    token_id: 1,
                    string_token: "Hello".to_string(),
                    token_bytes: vec![72, 101, 108, 108, 111],
                },
                Token {
                    token_id: 2,
                    string_token: ", ".to_string(),
                    token_bytes: vec![44, 32],
                },
                Token {
                    token_id: 3,
                    string_token: "world".to_string(),
                    token_bytes: vec![119, 111, 114, 108, 100],
                },
            ],
            model: "grok-2-1212".to_string(),
        };

        assert_eq!(response.text(), "Hello, world");
    }

    #[test]
    fn test_tokenize_response_empty() {
        let response = TokenizeResponse {
            tokens: vec![],
            model: "grok-2-1212".to_string(),
        };

        assert_eq!(response.token_count(), 0);
        assert_eq!(response.text(), "");
    }
}
