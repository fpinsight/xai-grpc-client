//! Tool calling support for Grok API
//!
//! This module provides ergonomic Rust types for working with Grok's tool calling capabilities,
//! including function calling, web search, X search, code execution, and more.

use serde_json::Value;
use std::collections::HashMap;

// Use shared proto module
use crate::proto::{
    self, CodeExecution as ProtoCodeExecution, CollectionsSearch as ProtoCollectionsSearch,
    DocumentSearch as ProtoDocumentSearch, Function as ProtoFunction,
    FunctionCall as ProtoFunctionCall, Mcp as ProtoMcp, ToolCall as ProtoToolCall,
    ToolCallStatus, ToolCallType, ToolChoice as ProtoToolChoice, ToolMode,
    WebSearch as ProtoWebSearch, XSearch as ProtoXSearch,
};

/// Ergonomic wrapper for tool definitions
#[derive(Clone, Debug)]
pub enum Tool {
    /// Client-side function calling (like OpenAI)
    Function(FunctionTool),
    /// Server-side web search with domain filters
    WebSearch(WebSearchTool),
    /// Search X posts with engagement thresholds
    XSearch(XSearchTool),
    /// Server-side code execution
    CodeExecution,
    /// Search custom data collections
    CollectionsSearch(CollectionsSearchTool),
    /// Model Context Protocol integration
    Mcp(McpTool),
    /// Document retrieval
    DocumentSearch(DocumentSearchTool),
}

impl Tool {
    /// Convert to protobuf representation
    pub fn to_proto(&self) -> proto::Tool {
        let tool = match self {
            Tool::Function(f) => proto::tool::Tool::Function(f.to_proto()),
            Tool::WebSearch(w) => proto::tool::Tool::WebSearch(w.to_proto()),
            Tool::XSearch(x) => proto::tool::Tool::XSearch(x.to_proto()),
            Tool::CodeExecution => proto::tool::Tool::CodeExecution(ProtoCodeExecution {}),
            Tool::CollectionsSearch(c) => proto::tool::Tool::CollectionsSearch(c.to_proto()),
            Tool::Mcp(m) => proto::tool::Tool::Mcp(m.to_proto()),
            Tool::DocumentSearch(d) => proto::tool::Tool::DocumentSearch(d.to_proto()),
        };

        proto::Tool { tool: Some(tool) }
    }
}

/// Client-side function tool definition
#[derive(Clone, Debug)]
pub struct FunctionTool {
    /// Name of the function
    pub name: String,
    /// Description of what the function does
    pub description: String,
    /// JSON Schema describing the function parameters
    pub parameters: Value,
}

impl FunctionTool {
    /// Create a new function tool
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {},
            }),
        }
    }

    /// Set the parameters JSON schema
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = parameters;
        self
    }

    fn to_proto(&self) -> ProtoFunction {
        ProtoFunction {
            name: self.name.clone(),
            description: self.description.clone(),
            strict: false,
            parameters: self.parameters.to_string(),
        }
    }
}

/// Web search tool configuration
#[derive(Clone, Debug, Default)]
pub struct WebSearchTool {
    /// Domains to exclude from results (max 5)
    pub excluded_domains: Vec<String>,
    /// Domains to restrict results to (max 5)
    pub allowed_domains: Vec<String>,
    /// Enable image understanding in search results
    pub enable_image_understanding: Option<bool>,
}

impl WebSearchTool {
    /// Create a new web search tool
    pub fn new() -> Self {
        Self::default()
    }

    /// Exclude specific domains from search results
    pub fn with_excluded_domains(mut self, domains: Vec<String>) -> Self {
        self.excluded_domains = domains;
        self
    }

    /// Restrict search to specific domains only
    pub fn with_allowed_domains(mut self, domains: Vec<String>) -> Self {
        self.allowed_domains = domains;
        self
    }

    /// Enable image understanding in search results
    pub fn with_image_understanding(mut self, enable: bool) -> Self {
        self.enable_image_understanding = Some(enable);
        self
    }

    fn to_proto(&self) -> ProtoWebSearch {
        ProtoWebSearch {
            excluded_domains: self.excluded_domains.clone(),
            allowed_domains: self.allowed_domains.clone(),
            enable_image_understanding: self.enable_image_understanding,
        }
    }
}

/// X (Twitter) search tool configuration
#[derive(Clone, Debug, Default)]
pub struct XSearchTool {
    /// Start date for search results (ISO-8601)
    pub from_date: Option<prost_types::Timestamp>,
    /// End date for search results (ISO-8601)
    pub to_date: Option<prost_types::Timestamp>,
    /// Allowed X handles
    pub allowed_x_handles: Vec<String>,
    /// Excluded X handles
    pub excluded_x_handles: Vec<String>,
    /// Enable image understanding
    pub enable_image_understanding: Option<bool>,
    /// Enable video understanding
    pub enable_video_understanding: Option<bool>,
}

impl XSearchTool {
    /// Create a new X search tool
    pub fn new() -> Self {
        Self::default()
    }

    /// Set date range for search results
    pub fn with_date_range(
        mut self,
        from: Option<prost_types::Timestamp>,
        to: Option<prost_types::Timestamp>,
    ) -> Self {
        self.from_date = from;
        self.to_date = to;
        self
    }

    /// Set allowed X handles
    pub fn with_allowed_handles(mut self, handles: Vec<String>) -> Self {
        self.allowed_x_handles = handles;
        self
    }

    /// Set excluded X handles
    pub fn with_excluded_handles(mut self, handles: Vec<String>) -> Self {
        self.excluded_x_handles = handles;
        self
    }

    /// Enable media understanding
    pub fn with_media_understanding(mut self, images: bool, videos: bool) -> Self {
        self.enable_image_understanding = Some(images);
        self.enable_video_understanding = Some(videos);
        self
    }

    fn to_proto(&self) -> ProtoXSearch {
        ProtoXSearch {
            from_date: self.from_date.clone(),
            to_date: self.to_date.clone(),
            allowed_x_handles: self.allowed_x_handles.clone(),
            excluded_x_handles: self.excluded_x_handles.clone(),
            enable_image_understanding: self.enable_image_understanding,
            enable_video_understanding: self.enable_video_understanding,
        }
    }
}

/// Collections search tool configuration
#[derive(Clone, Debug)]
pub struct CollectionsSearchTool {
    /// Collection IDs to search (max 10)
    pub collection_ids: Vec<String>,
    /// Number of chunks to return per collection
    pub limit: Option<i32>,
}

impl CollectionsSearchTool {
    /// Create a new collections search tool
    pub fn new(collection_ids: Vec<String>) -> Self {
        Self {
            collection_ids,
            limit: None,
        }
    }

    /// Set the limit of chunks to return
    pub fn with_limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    fn to_proto(&self) -> ProtoCollectionsSearch {
        ProtoCollectionsSearch {
            collection_ids: self.collection_ids.clone(),
            limit: self.limit,
        }
    }
}

/// Model Context Protocol server configuration
#[derive(Clone, Debug)]
pub struct McpTool {
    /// Label for the MCP server
    pub server_label: String,
    /// Description of the server
    pub server_description: String,
    /// Server URL
    pub server_url: String,
    /// Allowed tool names
    pub allowed_tool_names: Vec<String>,
    /// Authorization token
    pub authorization: Option<String>,
    /// Extra headers to send
    pub extra_headers: HashMap<String, String>,
}

impl McpTool {
    /// Create a new MCP tool
    pub fn new(server_url: impl Into<String>) -> Self {
        Self {
            server_label: String::new(),
            server_description: String::new(),
            server_url: server_url.into(),
            allowed_tool_names: Vec::new(),
            authorization: None,
            extra_headers: HashMap::new(),
        }
    }

    /// Set a label for the MCP server
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.server_label = label.into();
        self
    }

    /// Set a description for the MCP server
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.server_description = description.into();
        self
    }

    /// Set allowed tool names
    pub fn with_allowed_tools(mut self, tools: Vec<String>) -> Self {
        self.allowed_tool_names = tools;
        self
    }

    /// Set authorization token
    pub fn with_authorization(mut self, token: impl Into<String>) -> Self {
        self.authorization = Some(token.into());
        self
    }

    /// Add extra headers
    pub fn with_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.extra_headers = headers;
        self
    }

    fn to_proto(&self) -> ProtoMcp {
        ProtoMcp {
            server_label: self.server_label.clone(),
            server_description: self.server_description.clone(),
            server_url: self.server_url.clone(),
            allowed_tool_names: self.allowed_tool_names.clone(),
            authorization: self.authorization.clone(),
            extra_headers: self.extra_headers.clone(),
        }
    }
}

/// Document search tool configuration
#[derive(Clone, Debug, Default)]
pub struct DocumentSearchTool {
    /// Number of files to limit search to
    pub limit: Option<i32>,
}

impl DocumentSearchTool {
    /// Create a new document search tool
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the limit of files to search
    pub fn with_limit(mut self, limit: i32) -> Self {
        self.limit = Some(limit);
        self
    }

    fn to_proto(&self) -> ProtoDocumentSearch {
        ProtoDocumentSearch { limit: self.limit }
    }
}

/// Tool choice configuration - controls which tools the model can use
#[derive(Clone, Debug)]
pub enum ToolChoice {
    /// Let the model decide whether to use tools
    Auto,
    /// Require the model to use a tool
    Required,
    /// Force the model to call a specific function
    Function(String),
}

impl ToolChoice {
    /// Convert to protobuf representation
    pub fn to_proto(&self) -> ProtoToolChoice {
        let tool_choice = match self {
            ToolChoice::Auto => proto::tool_choice::ToolChoice::Mode(ToolMode::Auto as i32),
            ToolChoice::Required => {
                proto::tool_choice::ToolChoice::Mode(ToolMode::Required as i32)
            }
            ToolChoice::Function(name) => {
                proto::tool_choice::ToolChoice::FunctionName(name.clone())
            }
        };

        ProtoToolChoice {
            tool_choice: Some(tool_choice),
        }
    }
}

/// Tool call from the model (in responses)
#[derive(Clone, Debug)]
pub struct ToolCall {
    /// Unique identifier for this tool call
    pub id: String,
    /// Type of tool call (client-side or server-side)
    pub call_type: ToolCallKind,
    /// Status of the tool call
    pub status: ToolCallStatusKind,
    /// Error message if the call failed
    pub error_message: Option<String>,
    /// The actual function call details
    pub function: FunctionCall,
}

impl ToolCall {
    /// Parse from protobuf representation
    pub fn from_proto(proto: ProtoToolCall) -> Option<Self> {
        let function = match proto.tool? {
            proto::tool_call::Tool::Function(f) => FunctionCall {
                name: f.name,
                arguments: f.arguments,
            },
        };

        Some(Self {
            id: proto.id,
            call_type: ToolCallKind::from_proto(proto.r#type),
            status: ToolCallStatusKind::from_proto(proto.status),
            error_message: proto.error_message,
            function,
        })
    }

    /// Convert to protobuf representation
    pub fn to_proto(&self) -> ProtoToolCall {
        ProtoToolCall {
            id: self.id.clone(),
            r#type: self.call_type.to_proto() as i32,
            status: self.status.to_proto() as i32,
            error_message: self.error_message.clone(),
            tool: Some(proto::tool_call::Tool::Function(ProtoFunctionCall {
                name: self.function.name.clone(),
                arguments: self.function.arguments.clone(),
            })),
        }
    }
}

/// Type of tool call
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolCallKind {
    /// Client-side function (maps to OpenAI's function_call)
    ClientSideTool,
    /// Server-side web search
    WebSearchTool,
    /// Server-side X search
    XSearchTool,
    /// Server-side code execution
    CodeExecutionTool,
    /// Server-side collections search
    CollectionsSearchTool,
    /// Server-side MCP tool
    McpTool,
    /// Unknown or invalid type
    Unknown,
}

impl ToolCallKind {
    fn from_proto(value: i32) -> Self {
        match value {
            x if x == ToolCallType::ClientSideTool as i32 => ToolCallKind::ClientSideTool,
            x if x == ToolCallType::WebSearchTool as i32 => ToolCallKind::WebSearchTool,
            x if x == ToolCallType::XSearchTool as i32 => ToolCallKind::XSearchTool,
            x if x == ToolCallType::CodeExecutionTool as i32 => ToolCallKind::CodeExecutionTool,
            x if x == ToolCallType::CollectionsSearchTool as i32 => {
                ToolCallKind::CollectionsSearchTool
            }
            x if x == ToolCallType::McpTool as i32 => ToolCallKind::McpTool,
            _ => ToolCallKind::Unknown,
        }
    }

    fn to_proto(&self) -> ToolCallType {
        match self {
            ToolCallKind::ClientSideTool => ToolCallType::ClientSideTool,
            ToolCallKind::WebSearchTool => ToolCallType::WebSearchTool,
            ToolCallKind::XSearchTool => ToolCallType::XSearchTool,
            ToolCallKind::CodeExecutionTool => ToolCallType::CodeExecutionTool,
            ToolCallKind::CollectionsSearchTool => ToolCallType::CollectionsSearchTool,
            ToolCallKind::McpTool => ToolCallType::McpTool,
            ToolCallKind::Unknown => ToolCallType::Invalid,
        }
    }
}

/// Status of a tool call
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ToolCallStatusKind {
    /// Tool call is in progress
    InProgress,
    /// Tool call completed successfully
    Completed,
    /// Tool call incomplete
    Incomplete,
    /// Tool call failed
    Failed,
}

impl ToolCallStatusKind {
    fn from_proto(value: i32) -> Self {
        match value {
            x if x == ToolCallStatus::InProgress as i32 => ToolCallStatusKind::InProgress,
            x if x == ToolCallStatus::Completed as i32 => ToolCallStatusKind::Completed,
            x if x == ToolCallStatus::Incomplete as i32 => ToolCallStatusKind::Incomplete,
            x if x == ToolCallStatus::Failed as i32 => ToolCallStatusKind::Failed,
            _ => ToolCallStatusKind::InProgress, // default
        }
    }

    fn to_proto(&self) -> ToolCallStatus {
        match self {
            ToolCallStatusKind::InProgress => ToolCallStatus::InProgress,
            ToolCallStatusKind::Completed => ToolCallStatus::Completed,
            ToolCallStatusKind::Incomplete => ToolCallStatus::Incomplete,
            ToolCallStatusKind::Failed => ToolCallStatus::Failed,
        }
    }
}

/// Function call details
#[derive(Clone, Debug)]
pub struct FunctionCall {
    /// Name of the function to call
    pub name: String,
    /// Arguments as JSON string
    pub arguments: String,
}

impl FunctionCall {
    /// Parse arguments as JSON
    pub fn parse_arguments<T: serde::de::DeserializeOwned>(&self) -> serde_json::Result<T> {
        serde_json::from_str(&self.arguments)
    }

    /// Get arguments as a JSON value
    pub fn arguments_json(&self) -> serde_json::Result<Value> {
        serde_json::from_str(&self.arguments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_function_tool_creation() {
        let tool = FunctionTool::new("get_weather", "Get current weather");
        assert_eq!(tool.name, "get_weather");
        assert_eq!(tool.description, "Get current weather");
    }

    #[test]
    fn test_function_tool_with_parameters() {
        let params = json!({
            "type": "object",
            "properties": {
                "location": {"type": "string"}
            }
        });

        let tool = FunctionTool::new("get_weather", "Get weather")
            .with_parameters(params.clone());

        assert_eq!(tool.parameters, params);
    }

    #[test]
    fn test_web_search_tool() {
        let tool = WebSearchTool::new().with_excluded_domains(vec!["spam.com".to_string()]);
        assert_eq!(tool.excluded_domains.len(), 1);
    }

    #[test]
    fn test_x_search_tool() {
        let tool = XSearchTool::new().with_allowed_handles(vec!["@rustlang".to_string()]);
        assert_eq!(tool.allowed_x_handles.len(), 1);
    }

    #[test]
    fn test_tool_choice_auto() {
        let choice = ToolChoice::Auto;
        assert!(matches!(choice, ToolChoice::Auto));
    }

    #[test]
    fn test_tool_choice_required() {
        let choice = ToolChoice::Required;
        assert!(matches!(choice, ToolChoice::Required));
    }

    #[test]
    fn test_tool_choice_function() {
        let choice = ToolChoice::Function("my_function".to_string());
        match choice {
            ToolChoice::Function(name) => assert_eq!(name, "my_function"),
            _ => panic!("Expected Function variant"),
        }
    }

    #[test]
    fn test_function_call_parse_arguments() {
        let call = FunctionCall {
            name: "test_fn".to_string(),
            arguments: r#"{"param": "value"}"#.to_string(),
        };

        let json = call.arguments_json().unwrap();
        assert_eq!(json["param"], "value");
    }

    #[test]
    fn test_mcp_tool() {
        let tool = McpTool::new("https://example.com/mcp")
            .with_label("My MCP Server");
        assert_eq!(tool.server_url, "https://example.com/mcp");
        assert_eq!(tool.server_label, "My MCP Server");
    }

    #[test]
    fn test_collections_search_tool() {
        let tool = CollectionsSearchTool::new(vec!["coll_1".to_string()]).with_limit(10);

        assert_eq!(tool.collection_ids.len(), 1);
        assert_eq!(tool.limit, Some(10));
    }

    #[test]
    fn test_document_search_tool() {
        let tool = DocumentSearchTool::new().with_limit(20);

        assert_eq!(tool.limit, Some(20));
    }
}
