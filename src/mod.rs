// Include generated protobuf code once at module level
#[path = "../generated/xai_api.rs"]
#[allow(warnings)]
pub(crate) mod proto;

mod auth;
pub mod client;
mod error;
pub mod request;
pub mod response;
pub mod tools;

pub use client::{GrokClient, GrokConfig};
pub use error::{GrokError, Result};
pub use request::{
    ChatRequest, ContentPart, ImageDetail, Message, MessageContent, ReasoningEffort, SearchConfig,
    CompletionOptions, SearchMode, SearchSource, ResponseFormat,
};
pub use response::{ChatResponse, ChatChunk, FinishReason, TokenUsage, LogProbs, LogProb, TopLogProb};
pub use tools::{
    CollectionsSearchTool, DocumentSearchTool, FunctionCall, FunctionTool, McpTool, Tool,
    ToolCall, ToolCallKind, ToolCallStatusKind, ToolChoice, WebSearchTool, XSearchTool,
};
