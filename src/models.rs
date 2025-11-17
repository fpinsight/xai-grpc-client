//! Model listing and information API.
//!
//! This module provides access to xAI's model listing API, allowing you to:
//! - List all available language models with [`GrokClient::list_models`](crate::GrokClient::list_models)
//! - Get detailed information about specific models with [`GrokClient::get_model`](crate::GrokClient::get_model)
//! - Check pricing, context lengths, and capabilities
//!
//! # Examples
//!
//! ## Listing all models
//!
//! ```no_run
//! use xai_grpc_client::GrokClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = GrokClient::from_env().await?;
//!     let models = client.list_models().await?;
//!
//!     for model in models {
//!         println!("{}: {} tokens", model.name, model.max_prompt_length);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Getting specific model information
//!
//! ```no_run
//! use xai_grpc_client::GrokClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = GrokClient::from_env().await?;
//!     let model = client.get_model("grok-2-1212").await?;
//!
//!     println!("Model: {}", model.name);
//!     println!("Version: {}", model.version);
//!     println!("Max context: {} tokens", model.max_prompt_length);
//!     println!("Multimodal: {}", model.supports_multimodal());
//!
//!     // Calculate cost for a typical request
//!     let cost = model.calculate_cost(10_000, 1_000, 0);
//!     println!("Cost for 10K prompt + 1K completion: ${:.4}", cost);
//!
//!     Ok(())
//! }
//! ```

use crate::proto;

/// Information about a language model.
///
///This struct contains comprehensive metadata about an xAI language model,
/// including its capabilities, pricing, and technical specifications.
///
/// # Pricing Units
///
/// The pricing fields use specific units to represent fractional cents:
/// - `prompt_text_token_price`: 1/100 USD cents per 1M tokens (e.g., 500 = $0.05 per 1M tokens)
/// - `prompt_image_token_price`: 1/100 USD cents per 1M tokens
/// - `completion_text_token_price`: 1/100 USD cents per 1M tokens
/// - `cached_prompt_token_price`: USD cents per 100M tokens (e.g., 50 = $0.50 per 100M tokens)
/// - `search_price`: 1/100 USD cents per 1M searches
///
/// Use [`calculate_cost`](LanguageModel::calculate_cost) to convert these to USD amounts.
///
/// # Examples
///
/// ```no_run
/// # use xai_grpc_client::GrokClient;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let mut client = GrokClient::from_env().await?;
/// let model = client.get_model("grok-2-1212").await?;
///
/// // Check capabilities
/// if model.supports_multimodal() {
///     println!("{} supports images!", model.name);
/// }
///
/// // Calculate costs
/// let cost = model.calculate_cost(50_000, 5_000, 0);
/// println!("50K prompt + 5K completion costs: ${:.4}", cost);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct LanguageModel {
    /// The model name used in API requests (e.g., "grok-2-1212").
    pub name: String,

    /// Alternative names that can be used for this model (e.g., ["grok-2-latest"]).
    ///
    /// Aliases provide convenient shortcuts for referring to models without
    /// needing to know the specific version number.
    pub aliases: Vec<String>,

    /// Version number of the model (e.g., "2.0").
    pub version: String,

    /// Supported input modalities.
    ///
    /// Common combinations:
    /// - `[Text]` - Text-only model
    /// - `[Text, Image]` - Multimodal model supporting vision
    pub input_modalities: Vec<Modality>,

    /// Supported output modalities.
    ///
    /// Most models output `[Text]`, but some specialized models may
    /// support image generation or embeddings.
    pub output_modalities: Vec<Modality>,

    /// Price per million prompt text tokens in 1/100 USD cents.
    ///
    /// Example: 500 = $0.05 per 1M tokens = $0.00005 per token
    pub prompt_text_token_price: i64,

    /// Price per million prompt image tokens in 1/100 USD cents.
    ///
    /// Only applicable for multimodal models that accept images.
    pub prompt_image_token_price: i64,

    /// Price per 100 million cached prompt tokens in USD cents.
    ///
    /// Example: 50 = $0.50 per 100M tokens
    ///
    /// Cached tokens are significantly cheaper as they're reused from
    /// previous requests with the same prefix.
    pub cached_prompt_token_price: i64,

    /// Price per million completion text tokens in 1/100 USD cents.
    ///
    /// Example: 1500 = $0.15 per 1M tokens = $0.00015 per token
    pub completion_text_token_price: i64,

    /// Price per million searches in 1/100 USD cents.
    ///
    /// Only applicable when using web search or X search tools.
    pub search_price: i64,

    /// Maximum context length in tokens (prompt + completion).
    ///
    /// This represents the total number of tokens the model can process
    /// in a single request, including both input and output.
    pub max_prompt_length: i32,

    /// Backend configuration fingerprint.
    ///
    /// This identifier tracks the specific backend configuration used by
    /// the model, useful for debugging and reproducibility.
    pub system_fingerprint: String,
}

/// Modality supported by a model for input or output.
///
/// Models can support different combinations of modalities:
/// - Text-only models: `input_modalities: [Text]`, `output_modalities: [Text]`
/// - Vision models: `input_modalities: [Text, Image]`, `output_modalities: [Text]`
/// - Embedding models: `input_modalities: [Text]`, `output_modalities: [Embedding]`
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Modality {
    /// Text input/output - supported by all language models.
    Text,

    /// Image input/output - supported by multimodal vision models.
    ///
    /// Models with `Image` in `input_modalities` can process image URLs
    /// alongside text prompts.
    Image,

    /// Embedding input/output - vector representations.
    ///
    /// Used by embedding models that convert text or images into
    /// high-dimensional vector representations for semantic search.
    Embedding,
}

/// Information about an embedding model.
///
/// Embedding models convert text or images into high-dimensional vector
/// representations that can be used for semantic search, clustering, and
/// similarity comparisons.
///
/// # Pricing Units
///
/// - `prompt_text_token_price`: 1/100 USD cents per 1M tokens
/// - `prompt_image_token_price`: 1/100 USD cents per 1M tokens
///
/// # Examples
///
/// ```no_run
/// # use xai_grpc_client::GrokClient;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let mut client = GrokClient::from_env().await?;
/// let model = client.get_embedding_model("embed-large-v1").await?;
///
/// println!("Model: {}", model.name);
/// println!("Version: {}", model.version);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct EmbeddingModel {
    /// The model name used in API requests (e.g., "embed-large-v1").
    pub name: String,

    /// Alternative names that can be used for this model.
    pub aliases: Vec<String>,

    /// Version number of the model.
    pub version: String,

    /// Supported input modalities (typically Text and optionally Image).
    pub input_modalities: Vec<Modality>,

    /// Supported output modalities (always includes Embedding).
    pub output_modalities: Vec<Modality>,

    /// Price per million text prompt tokens in 1/100 USD cents.
    pub prompt_text_token_price: i64,

    /// Price per million image prompt tokens in 1/100 USD cents.
    pub prompt_image_token_price: i64,

    /// Backend configuration fingerprint.
    pub system_fingerprint: String,
}

/// Information about an image generation model.
///
/// Image generation models create images from text prompts.
///
/// # Pricing Units
///
/// - `image_price`: USD cents per image
///
/// # Examples
///
/// ```no_run
/// # use xai_grpc_client::GrokClient;
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// # let mut client = GrokClient::from_env().await?;
/// let model = client.get_image_generation_model("image-gen-1").await?;
///
/// println!("Model: {}", model.name);
/// println!("Cost per image: ${:.2}", model.image_price as f64 / 100.0);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, Debug)]
pub struct ImageGenerationModel {
    /// The model name used in API requests.
    pub name: String,

    /// Alternative names that can be used for this model.
    pub aliases: Vec<String>,

    /// Version number of the model.
    pub version: String,

    /// Supported input modalities (typically Text).
    pub input_modalities: Vec<Modality>,

    /// Supported output modalities (typically Image).
    pub output_modalities: Vec<Modality>,

    /// Price per image in USD cents.
    ///
    /// Example: 200 = $2.00 per image
    pub image_price: i64,

    /// Maximum length of the prompt/input in tokens.
    pub max_prompt_length: i32,

    /// Backend configuration fingerprint.
    pub system_fingerprint: String,
}

impl From<proto::LanguageModel> for LanguageModel {
    fn from(proto: proto::LanguageModel) -> Self {
        Self {
            name: proto.name,
            aliases: proto.aliases,
            version: proto.version,
            input_modalities: proto
                .input_modalities
                .into_iter()
                .filter_map(|m| proto::Modality::try_from(m).ok())
                .map(Modality::from)
                .collect(),
            output_modalities: proto
                .output_modalities
                .into_iter()
                .filter_map(|m| proto::Modality::try_from(m).ok())
                .map(Modality::from)
                .collect(),
            prompt_text_token_price: proto.prompt_text_token_price,
            prompt_image_token_price: proto.prompt_image_token_price,
            cached_prompt_token_price: proto.cached_prompt_token_price,
            completion_text_token_price: proto.completion_text_token_price,
            search_price: proto.search_price,
            max_prompt_length: proto.max_prompt_length,
            system_fingerprint: proto.system_fingerprint,
        }
    }
}

impl From<proto::Modality> for Modality {
    fn from(proto: proto::Modality) -> Self {
        match proto {
            proto::Modality::Text => Modality::Text,
            proto::Modality::Image => Modality::Image,
            proto::Modality::Embedding => Modality::Embedding,
            proto::Modality::InvalidModality => Modality::Text, // Default fallback
        }
    }
}

impl LanguageModel {
    /// Calculate the cost (in USD) for a given number of prompt and completion tokens.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use xai_grpc_client::models::LanguageModel;
    /// # let model = LanguageModel {
    /// #     name: "grok-2".to_string(),
    /// #     aliases: vec![],
    /// #     version: "1.0".to_string(),
    /// #     input_modalities: vec![],
    /// #     output_modalities: vec![],
    /// #     prompt_text_token_price: 500,
    /// #     prompt_image_token_price: 0,
    /// #     cached_prompt_token_price: 0,
    /// #     completion_text_token_price: 1500,
    /// #     search_price: 0,
    /// #     max_prompt_length: 131072,
    /// #     system_fingerprint: "".to_string(),
    /// # };
    /// let cost = model.calculate_cost(1000, 500, 0);
    /// println!("Cost: ${:.4}", cost);
    /// ```
    pub fn calculate_cost(
        &self,
        prompt_tokens: u32,
        completion_tokens: u32,
        cached_tokens: u32,
    ) -> f64 {
        let prompt_cost =
            (prompt_tokens as f64 * self.prompt_text_token_price as f64) / 1_000_000.0 / 100.0;
        let cached_cost =
            (cached_tokens as f64 * self.cached_prompt_token_price as f64) / 100_000_000.0;
        let completion_cost = (completion_tokens as f64 * self.completion_text_token_price as f64)
            / 1_000_000.0
            / 100.0;

        prompt_cost + cached_cost + completion_cost
    }

    /// Check if the model supports multimodal input (text + images).
    ///
    /// Returns `true` if the model accepts both text and image inputs,
    /// allowing you to send image URLs alongside text prompts.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use xai_grpc_client::GrokClient;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// # let mut client = GrokClient::from_env().await?;
    /// let model = client.get_model("grok-2-vision-1212").await?;
    ///
    /// if model.supports_multimodal() {
    ///     println!("{} can process images!", model.name);
    /// } else {
    ///     println!("{} is text-only", model.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn supports_multimodal(&self) -> bool {
        self.input_modalities.contains(&Modality::Text)
            && self.input_modalities.contains(&Modality::Image)
    }
}

impl From<proto::EmbeddingModel> for EmbeddingModel {
    fn from(proto: proto::EmbeddingModel) -> Self {
        Self {
            name: proto.name,
            aliases: proto.aliases,
            version: proto.version,
            input_modalities: proto
                .input_modalities
                .into_iter()
                .filter_map(|m| proto::Modality::try_from(m).ok())
                .map(Modality::from)
                .collect(),
            output_modalities: proto
                .output_modalities
                .into_iter()
                .filter_map(|m| proto::Modality::try_from(m).ok())
                .map(Modality::from)
                .collect(),
            prompt_text_token_price: proto.prompt_text_token_price,
            prompt_image_token_price: proto.prompt_image_token_price,
            system_fingerprint: proto.system_fingerprint,
        }
    }
}

impl From<proto::ImageGenerationModel> for ImageGenerationModel {
    fn from(proto: proto::ImageGenerationModel) -> Self {
        Self {
            name: proto.name,
            aliases: proto.aliases,
            version: proto.version,
            input_modalities: proto
                .input_modalities
                .into_iter()
                .filter_map(|m| proto::Modality::try_from(m).ok())
                .map(Modality::from)
                .collect(),
            output_modalities: proto
                .output_modalities
                .into_iter()
                .filter_map(|m| proto::Modality::try_from(m).ok())
                .map(Modality::from)
                .collect(),
            image_price: proto.image_price,
            max_prompt_length: proto.max_prompt_length,
            system_fingerprint: proto.system_fingerprint,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_model() -> LanguageModel {
        LanguageModel {
            name: "grok-2".to_string(),
            aliases: vec!["grok-latest".to_string()],
            version: "1.0".to_string(),
            input_modalities: vec![Modality::Text],
            output_modalities: vec![Modality::Text],
            prompt_text_token_price: 500, // $0.005 per 1M tokens
            prompt_image_token_price: 0,
            cached_prompt_token_price: 50,     // $0.50 per 100M tokens
            completion_text_token_price: 1500, // $0.015 per 1M tokens
            search_price: 0,
            max_prompt_length: 131072,
            system_fingerprint: "fp_test".to_string(),
        }
    }

    #[test]
    fn test_calculate_cost_basic() {
        let model = create_test_model();

        // price is in 1/100 USD cents per 1M tokens
        // 500 = $0.05 per 1M tokens = $0.00005 per 1K tokens
        // 1500 = $0.15 per 1M tokens = $0.00015 per 1K tokens
        // 1000 prompt * 0.00005 + 500 completion * 0.00015 = 0.005 + 0.0075 = 0.0125
        let cost = model.calculate_cost(1000, 500, 0);
        assert!(
            (cost - 0.0125).abs() < 0.0001,
            "Expected ~$0.0125, got ${cost}"
        );
    }

    #[test]
    fn test_calculate_cost_with_cached() {
        let model = create_test_model();

        // cached_prompt_token_price is in USD cents per 100M tokens
        // Formula: (cached_tokens * cached_prompt_token_price) / 100_000_000
        // = (10000 * 50) / 100_000_000 = 500000 / 100000000 = $0.005
        // Total: $0.005 (prompt) + $0.0075 (completion) + $0.005 (cached) = $0.0175
        let cost = model.calculate_cost(1000, 500, 10000);
        assert!(
            (cost - 0.0175).abs() < 0.0001,
            "Expected ~$0.0175, got ${cost}"
        );
    }

    #[test]
    fn test_calculate_cost_large_numbers() {
        let model = create_test_model();

        // 1M prompt + 100K completion
        // = 1M * 0.00005 + 100K * 0.00015 = $5.0 + $1.5 = $6.50
        let cost = model.calculate_cost(1_000_000, 100_000, 0);
        assert!((cost - 6.5).abs() < 0.01, "Expected ~$6.50, got ${cost}");
    }

    #[test]
    fn test_calculate_cost_zero() {
        let model = create_test_model();
        let cost = model.calculate_cost(0, 0, 0);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_supports_multimodal_text_only() {
        let text_only = LanguageModel {
            input_modalities: vec![Modality::Text],
            output_modalities: vec![Modality::Text],
            ..create_test_model()
        };

        assert!(!text_only.supports_multimodal());
    }

    #[test]
    fn test_supports_multimodal_vision() {
        let multimodal = LanguageModel {
            input_modalities: vec![Modality::Text, Modality::Image],
            output_modalities: vec![Modality::Text],
            ..create_test_model()
        };

        assert!(multimodal.supports_multimodal());
    }

    #[test]
    fn test_supports_multimodal_image_only() {
        let image_only = LanguageModel {
            input_modalities: vec![Modality::Image],
            output_modalities: vec![Modality::Image],
            ..create_test_model()
        };

        assert!(!image_only.supports_multimodal());
    }

    #[test]
    fn test_modality_from_proto() {
        assert_eq!(Modality::from(proto::Modality::Text), Modality::Text);
        assert_eq!(Modality::from(proto::Modality::Image), Modality::Image);
        assert_eq!(
            Modality::from(proto::Modality::Embedding),
            Modality::Embedding
        );
        // Invalid should default to Text
        assert_eq!(
            Modality::from(proto::Modality::InvalidModality),
            Modality::Text
        );
    }

    #[test]
    fn test_language_model_clone() {
        let model = create_test_model();
        let cloned = model.clone();

        assert_eq!(model.name, cloned.name);
        assert_eq!(model.version, cloned.version);
        assert_eq!(model.max_prompt_length, cloned.max_prompt_length);
    }

    #[test]
    fn test_language_model_debug() {
        let model = create_test_model();
        let debug_str = format!("{model:?}");
        assert!(debug_str.contains("grok-2"));
        assert!(debug_str.contains("1.0"));
    }

    #[test]
    fn test_language_model_aliases() {
        let model = create_test_model();
        assert_eq!(model.aliases.len(), 1);
        assert_eq!(model.aliases[0], "grok-latest");
    }

    #[test]
    fn test_language_model_from_proto() {
        let proto_model = proto::LanguageModel {
            name: "test-model".to_string(),
            aliases: vec!["test-alias".to_string()],
            version: "2.0".to_string(),
            input_modalities: vec![proto::Modality::Text as i32, proto::Modality::Image as i32],
            output_modalities: vec![proto::Modality::Text as i32],
            prompt_text_token_price: 1000,
            prompt_image_token_price: 2000,
            cached_prompt_token_price: 100,
            completion_text_token_price: 3000,
            search_price: 500,
            created: None,
            max_prompt_length: 32768,
            system_fingerprint: "fp_test_123".to_string(),
        };

        let model: LanguageModel = proto_model.into();

        assert_eq!(model.name, "test-model");
        assert_eq!(model.aliases, vec!["test-alias"]);
        assert_eq!(model.version, "2.0");
        assert_eq!(model.input_modalities.len(), 2);
        assert!(model.input_modalities.contains(&Modality::Text));
        assert!(model.input_modalities.contains(&Modality::Image));
        assert_eq!(model.prompt_text_token_price, 1000);
        assert_eq!(model.prompt_image_token_price, 2000);
        assert_eq!(model.cached_prompt_token_price, 100);
        assert_eq!(model.completion_text_token_price, 3000);
        assert_eq!(model.search_price, 500);
        assert_eq!(model.max_prompt_length, 32768);
        assert_eq!(model.system_fingerprint, "fp_test_123");
    }
}
