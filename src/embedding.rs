//! Embedding API for generating vector representations.
//!
//! This module provides access to xAI's embedding models, allowing you to:
//! - Generate embeddings from text strings
//! - Generate embeddings from images
//! - Support for both text-only and multimodal embedding models
//!
//! # Examples
//!
//! ## Embedding text
//!
//! ```no_run
//! use xai_grpc_client::{GrokClient, EmbedRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = GrokClient::from_env().await?;
//!
//!     let request = EmbedRequest::new("embed-large-v1")
//!         .add_text("Hello, world!")
//!         .add_text("How are you?");
//!
//!     let response = client.embed(request).await?;
//!
//!     for embedding in response.embeddings {
//!         println!("Embedding {} has {} dimensions",
//!             embedding.index, embedding.vector.len());
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Embedding images
//!
//! ```no_run
//! use xai_grpc_client::{GrokClient, EmbedRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = GrokClient::from_env().await?;
//!
//!     let request = EmbedRequest::new("embed-vision-v1")
//!         .add_image("https://example.com/image.jpg");
//!
//!     let response = client.embed(request).await?;
//!     println!("Generated {} embeddings", response.embeddings.len());
//!     Ok(())
//! }
//! ```

use crate::{proto, request::ImageDetail};

/// Request for generating embeddings.
///
/// Supports embedding text strings, images, or a mix of both depending on
/// the model capabilities. You can embed up to 128 inputs in a single request.
#[derive(Clone, Debug)]
pub struct EmbedRequest {
    /// Inputs to embed (text or images).
    pub inputs: Vec<EmbedInput>,
    /// Model name or alias to use.
    pub model: String,
    /// Encoding format for the embeddings (Float or Base64).
    pub encoding_format: EmbedEncodingFormat,
    /// Optional user identifier for tracking.
    pub user: Option<String>,
}

impl EmbedRequest {
    /// Create a new embedding request with the specified model.
    ///
    /// # Examples
    ///
    /// ```
    /// use xai_grpc_client::EmbedRequest;
    ///
    /// let request = EmbedRequest::new("embed-large-v1");
    /// ```
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            inputs: Vec::new(),
            model: model.into(),
            encoding_format: EmbedEncodingFormat::Float,
            user: None,
        }
    }

    /// Add a text string to embed.
    ///
    /// # Examples
    ///
    /// ```
    /// use xai_grpc_client::EmbedRequest;
    ///
    /// let request = EmbedRequest::new("embed-large-v1")
    ///     .add_text("Hello, world!");
    /// ```
    pub fn add_text(mut self, text: impl Into<String>) -> Self {
        self.inputs.push(EmbedInput::Text(text.into()));
        self
    }

    /// Add an image URL to embed.
    ///
    /// # Examples
    ///
    /// ```
    /// use xai_grpc_client::EmbedRequest;
    ///
    /// let request = EmbedRequest::new("embed-vision-v1")
    ///     .add_image("https://example.com/image.jpg");
    /// ```
    pub fn add_image(self, url: impl Into<String>) -> Self {
        self.add_image_with_detail(url, ImageDetail::Auto)
    }

    /// Add an image URL with specific detail level.
    ///
    /// # Examples
    ///
    /// ```
    /// use xai_grpc_client::{EmbedRequest, ImageDetail};
    ///
    /// let request = EmbedRequest::new("embed-vision-v1")
    ///     .add_image_with_detail("https://example.com/image.jpg", ImageDetail::High);
    /// ```
    pub fn add_image_with_detail(mut self, url: impl Into<String>, detail: ImageDetail) -> Self {
        self.inputs.push(EmbedInput::Image {
            url: url.into(),
            detail,
        });
        self
    }

    /// Set the encoding format for embeddings.
    ///
    /// # Examples
    ///
    /// ```
    /// use xai_grpc_client::{EmbedRequest, EmbedEncodingFormat};
    ///
    /// let request = EmbedRequest::new("embed-large-v1")
    ///     .with_encoding_format(EmbedEncodingFormat::Base64);
    /// ```
    pub fn with_encoding_format(mut self, format: EmbedEncodingFormat) -> Self {
        self.encoding_format = format;
        self
    }

    /// Set the user identifier for tracking.
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }
}

/// Input to be embedded (text or image).
#[derive(Clone, Debug)]
pub enum EmbedInput {
    /// Text string to embed.
    Text(String),
    /// Image URL to embed with optional detail level.
    Image {
        /// URL of the image.
        url: String,
        /// Detail level for processing.
        detail: ImageDetail,
    },
}

/// Encoding format for embedding vectors.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EmbedEncodingFormat {
    /// Return embeddings as arrays of floats.
    Float,
    /// Return embeddings as base64-encoded strings.
    Base64,
}

/// Response from an embedding request.
#[derive(Clone, Debug)]
pub struct EmbedResponse {
    /// Request identifier.
    pub id: String,
    /// Generated embeddings (one per input).
    pub embeddings: Vec<Embedding>,
    /// Token usage statistics.
    pub usage: EmbeddingUsage,
    /// Model name used (may differ from request if alias was used).
    pub model: String,
    /// Backend configuration fingerprint.
    pub system_fingerprint: String,
}

/// A single embedding vector.
#[derive(Clone, Debug)]
pub struct Embedding {
    /// Index of the input that generated this embedding.
    pub index: usize,
    /// The embedding vector.
    pub vector: Vec<f32>,
}

/// Usage statistics for an embedding request.
#[derive(Clone, Debug, Default)]
pub struct EmbeddingUsage {
    /// Number of text embeddings generated.
    pub num_text_embeddings: u32,
    /// Number of image embeddings generated.
    pub num_image_embeddings: u32,
}

impl From<proto::EmbeddingUsage> for EmbeddingUsage {
    fn from(proto: proto::EmbeddingUsage) -> Self {
        Self {
            num_text_embeddings: proto.num_text_embeddings as u32,
            num_image_embeddings: proto.num_image_embeddings as u32,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_request_builder() {
        let request = EmbedRequest::new("embed-large-v1")
            .add_text("Hello")
            .add_text("World");

        assert_eq!(request.model, "embed-large-v1");
        assert_eq!(request.inputs.len(), 2);
        assert!(matches!(request.inputs[0], EmbedInput::Text(_)));
    }

    #[test]
    fn test_embed_request_with_images() {
        let request = EmbedRequest::new("embed-vision-v1")
            .add_image("https://example.com/img1.jpg")
            .add_image_with_detail("https://example.com/img2.jpg", ImageDetail::High);

        assert_eq!(request.inputs.len(), 2);
        assert!(matches!(request.inputs[0], EmbedInput::Image { .. }));
        assert!(matches!(request.inputs[1], EmbedInput::Image { .. }));
    }

    #[test]
    fn test_embed_request_mixed() {
        let request = EmbedRequest::new("embed-multimodal-v1")
            .add_text("Description")
            .add_image("https://example.com/img.jpg");

        assert_eq!(request.inputs.len(), 2);
    }

    #[test]
    fn test_encoding_format() {
        let request =
            EmbedRequest::new("embed-large-v1").with_encoding_format(EmbedEncodingFormat::Base64);

        assert_eq!(request.encoding_format, EmbedEncodingFormat::Base64);
    }

    #[test]
    fn test_with_user() {
        let request = EmbedRequest::new("embed-large-v1").with_user("user123");

        assert_eq!(request.user, Some("user123".to_string()));
    }

    #[test]
    fn test_embedding_usage_default() {
        let usage = EmbeddingUsage::default();
        assert_eq!(usage.num_text_embeddings, 0);
        assert_eq!(usage.num_image_embeddings, 0);
    }

    #[test]
    fn test_embedding_usage_from_proto() {
        let proto = proto::EmbeddingUsage {
            num_text_embeddings: 5,
            num_image_embeddings: 2,
        };

        let usage: EmbeddingUsage = proto.into();
        assert_eq!(usage.num_text_embeddings, 5);
        assert_eq!(usage.num_image_embeddings, 2);
    }
}
