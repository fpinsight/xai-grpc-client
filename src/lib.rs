//! # xai-grpc-client
//!
//! Unofficial Rust client for xAI's Grok API with gRPC support.
//!
//! ## Features
//!
//! - **Async/await API** - Built on tokio and tonic
//! - **Type-safe requests** - Strongly typed request builders
//! - **Streaming support** - Real-time response streaming
//! - **Tool calling** - Function calling with 7 tool types
//! - **Multimodal** - Text and image inputs
//! - **Advanced features** - Log probabilities, reasoning traces, deferred completions
//! - **Secure by default** - Uses `secrecy` crate for API keys
//!
//! ## Quick Start
//!
//! ```no_run
//! use xai_grpc_client::{GrokClient, ChatRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize client from GROK_API_KEY environment variable
//!     let mut client = GrokClient::from_env().await?;
//!
//!     // Create a simple chat request
//!     let request = ChatRequest::new()
//!         .user_message("What is the meaning of life?")
//!         .with_model("grok-2-1212")
//!         .with_max_tokens(100);
//!
//!     // Get response
//!     let response = client.complete_chat(request).await?;
//!     println!("{}", response.content);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Streaming Example
//!
//! ```no_run
//! use xai_grpc_client::{GrokClient, ChatRequest};
//! use tokio_stream::StreamExt;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = GrokClient::from_env().await?;
//!
//!     let request = ChatRequest::new()
//!         .user_message("Write a short poem");
//!
//!     let mut stream = client.stream_chat(request).await?;
//!
//!     while let Some(chunk) = stream.next().await {
//!         let chunk = chunk?;
//!         print!("{}", chunk.delta);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Tool Calling Example
//!
//! ```no_run
//! use xai_grpc_client::{GrokClient, ChatRequest, FunctionTool, Tool, ToolChoice};
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = GrokClient::from_env().await?;
//!
//!     let get_weather = FunctionTool::new(
//!         "get_weather",
//!         "Get the current weather in a location"
//!     )
//!     .with_parameters(json!({
//!         "type": "object",
//!         "properties": {
//!             "location": {
//!                 "type": "string",
//!                 "description": "City name"
//!             }
//!         },
//!         "required": ["location"]
//!     }));
//!
//!     let request = ChatRequest::new()
//!         .user_message("What's the weather in Tokyo?")
//!         .add_tool(Tool::Function(get_weather))
//!         .with_tool_choice(ToolChoice::Auto);
//!
//!     let response = client.complete_chat(request).await?;
//!
//!     if !response.tool_calls.is_empty() {
//!         println!("Tool called: {:?}", response.tool_calls[0]);
//!     }
//!
//!     Ok(())
//! }
//! ```

// Include generated protobuf code once at module level
#[path = "generated/xai_api.rs"]
#[allow(warnings, missing_docs)]
pub(crate) mod proto;

#[allow(missing_docs)]
mod auth;

/// Client implementation for connecting to the xAI Grok API.
pub mod client;

/// Error types for the client.
mod error;

/// Request types and builders for chat completions.
pub mod request;

/// Response types for chat completions.
pub mod response;

/// Tool calling support (function calling, web search, etc.).
pub mod tools;

/// Model listing and information API.
pub mod models;

/// Embedding API for vector representations.
pub mod embedding;

/// Tokenization API for counting tokens.
pub mod tokenize;

/// API key information and status.
pub mod api_key;

/// Sample API for raw text sampling.
pub mod sample;

/// Image generation API.
pub mod image;

/// Documents search API for RAG.
pub mod documents;

// Re-exports for convenient access
pub use api_key::ApiKeyInfo;
pub use client::{GrokClient, GrokConfig};
pub use documents::{DocumentSearchRequest, DocumentSearchResponse, RankingMetric, SearchMatch};
pub use embedding::{
    EmbedEncodingFormat, EmbedInput, EmbedRequest, EmbedResponse, Embedding, EmbeddingUsage,
};
pub use error::{GrokError, Result};
pub use image::{GeneratedImage, ImageFormat, ImageGenerationRequest, ImageGenerationResponse};
pub use models::{EmbeddingModel, ImageGenerationModel, LanguageModel, Modality};
pub use proto::IncludeOption;
pub use request::{
    ChatRequest, CompletionOptions, ContentPart, ImageDetail, Message, MessageContent,
    ReasoningEffort, ResponseFormat, SearchConfig, SearchMode, SearchSource,
};
pub use response::{
    ChatChunk, ChatResponse, FinishReason, LogProb, LogProbs, TokenUsage, TopLogProb,
};
pub use sample::{SampleChoice, SampleRequest, SampleResponse};
pub use tokenize::{Token, TokenizeRequest, TokenizeResponse};
pub use tools::{
    CollectionsSearchTool, DocumentSearchTool, FunctionCall, FunctionTool, McpTool, Tool, ToolCall,
    ToolCallKind, ToolCallStatusKind, ToolChoice, WebSearchTool, XSearchTool,
};

// Re-export tonic types for users who need custom channel configuration
// This allows users to configure TLS, timeouts, and other transport options
// without adding tonic as a direct dependency
pub use tonic::transport::{Certificate, Channel, ClientTlsConfig, Endpoint};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        ChatRequest, ChatResponse, GrokClient, GrokConfig, Message, MessageContent,
        Result as GrokResult, Tool, ToolChoice,
    };
}
