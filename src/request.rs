//! Request types and builders for the Grok API.
//!
//! This module provides ergonomic builders for constructing chat completion requests
//! with support for multimodal inputs, tool calling, advanced sampling parameters,
//! and more.

use crate::proto::IncludeOption;
use crate::tools::{Tool, ToolChoice};
use serde_json::Value as JsonValue;

/// Configuration options for chat completions.
///
/// Used to configure requests with model selection, sampling parameters,
/// and tool calling settings. This type can be reused across multiple requests.
///
/// # Examples
///
/// ```
/// use xai_grpc_client::CompletionOptions;
///
/// let options = CompletionOptions::new()
///     .with_model("grok-2-1212")
///     .with_temperature(0.7)
///     .with_max_tokens(500);
/// ```
#[derive(Default, Clone, Debug)]
pub struct CompletionOptions {
    /// Model to use for completion.
    pub model: Option<String>,
    /// Sampling temperature (0.0-2.0).
    pub temperature: Option<f32>,
    /// Maximum tokens to generate.
    pub max_tokens: Option<u32>,
    /// Nucleus sampling probability.
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop_sequences: Vec<String>,
    pub tools: Option<Vec<Tool>>,
    pub tool_choice: Option<ToolChoice>,
    pub response_format: Option<ResponseFormat>,
}

impl CompletionOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }
}

/// Builder for constructing chat completion requests.
///
/// Provides a fluent API for building requests with various options like
/// model selection, sampling parameters, tool calling, multimodal inputs, and more.
///
/// # Examples
///
/// ```
/// use xai_grpc_client::ChatRequest;
///
/// let request = ChatRequest::new()
///     .user_message("What is the meaning of life?")
///     .with_model("grok-2-1212")
///     .with_max_tokens(100)
///     .with_temperature(0.7);
/// ```
#[derive(Default, Clone, Debug)]
pub struct ChatRequest {
    messages: Vec<Message>,
    model: Option<String>,
    max_tokens: Option<u32>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    stop: Vec<String>,
    reasoning_effort: Option<ReasoningEffort>,
    search: Option<SearchConfig>,
    seed: Option<i32>,
    response_format: Option<ResponseFormat>,
    tools: Option<Vec<Tool>>,
    tool_choice: Option<ToolChoice>,
    // New fields
    user: Option<String>,
    logprobs: bool,
    top_logprobs: Option<i32>,
    frequency_penalty: Option<f32>,
    presence_penalty: Option<f32>,
    parallel_tool_calls: Option<bool>,
    previous_response_id: Option<String>,
    store_messages: bool,
    use_encrypted_content: bool,
    max_turns: Option<i32>,
    include: Vec<IncludeOption>,
}

/// A message in a chat conversation.
///
/// Messages can be from the system (instructions), user (input), assistant (AI response),
/// or tool (result from a tool execution).
#[derive(Clone, Debug)]
pub enum Message {
    /// System message providing instructions or context to the model.
    System(String),
    /// User message containing the user's input (text or multimodal).
    User(MessageContent),
    /// Assistant message containing the AI's previous response.
    Assistant(String),
    /// Tool result message containing the output from a tool execution.
    ///
    /// # Important: Message Order
    ///
    /// The xAI gRPC API matches tool results to tool calls by **message order**, not by ID.
    /// Tool results must be provided in the **same order** as the tool calls were received
    /// from the model. The `tool_call_id` is accepted for API compatibility with other
    /// providers (e.g., OpenAI) but is not used by xAI's gRPC API.
    Tool {
        /// The ID of the tool call this is responding to.
        /// Note: Accepted for compatibility but not used by xAI's gRPC API.
        tool_call_id: String,
        /// The content/result of the tool execution.
        content: String,
    },
}

/// Content of a user message, which can be text-only or multimodal.
#[derive(Clone, Debug)]
pub enum MessageContent {
    /// Plain text message.
    Text(String),
    /// Multimodal message with text and/or images.
    MultiModal(Vec<ContentPart>),
}

/// A part of a multimodal message (text, image, or file attachment).
#[derive(Clone, Debug)]
pub enum ContentPart {
    /// Text content.
    Text(String),
    /// Image URL with optional detail level.
    ImageUrl {
        /// URL of the image.
        url: String,
        /// Level of detail for image processing.
        detail: Option<ImageDetail>,
    },
    /// File attachment by file ID.
    File {
        /// File ID from the Files API.
        file_id: String,
    },
}

/// Level of detail for image processing in vision models.
#[derive(Clone, Debug)]
pub enum ImageDetail {
    /// Automatic detail level.
    Auto,
    /// Low detail (faster, cheaper).
    Low,
    /// High detail (slower, more accurate).
    High,
}

impl From<String> for MessageContent {
    fn from(text: String) -> Self {
        MessageContent::Text(text)
    }
}

impl From<&str> for MessageContent {
    fn from(text: &str) -> Self {
        MessageContent::Text(text.to_string())
    }
}

/// Level of reasoning effort for the model.
///
/// Higher reasoning effort may produce better results for complex tasks
/// but will take longer and cost more tokens.
#[derive(Clone, Debug)]
pub enum ReasoningEffort {
    /// Minimal reasoning effort.
    Low,
    /// Moderate reasoning effort (balanced).
    Medium,
    /// Maximum reasoning effort for complex tasks.
    High,
}

/// Configuration for web search augmentation.
///
/// Allows the model to search the web for up-to-date information.
#[derive(Clone, Debug)]
pub struct SearchConfig {
    /// Search mode (default or advanced).
    pub mode: SearchMode,
    /// Sources to search (web, news, etc.).
    pub sources: Vec<SearchSource>,
    /// Maximum number of search results to return.
    pub max_results: Option<u32>,
}

/// Search mode for web search augmentation.
#[derive(Clone, Debug)]
pub enum SearchMode {
    /// Search disabled.
    Off,
    /// Search enabled.
    On,
    /// Automatic search when needed.
    Auto,
}

/// Source for search results.
#[derive(Clone, Debug)]
pub enum SearchSource {
    /// General web search.
    Web,
    /// X (Twitter) search.
    X,
    /// News articles.
    News,
}

/// Format for the model's response.
#[derive(Clone, Debug)]
pub enum ResponseFormat {
    /// Plain text response (default).
    Text,
    /// Any valid JSON object.
    JsonObject,
    /// Response must conform to provided JSON schema.
    JsonSchema(JsonValue),
}

impl ChatRequest {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn user_message(mut self, content: impl Into<MessageContent>) -> Self {
        self.messages.push(Message::User(content.into()));
        self
    }

    pub fn system_message(mut self, content: impl Into<String>) -> Self {
        self.messages.push(Message::System(content.into()));
        self
    }

    pub fn assistant_message(mut self, content: impl Into<String>) -> Self {
        self.messages.push(Message::Assistant(content.into()));
        self
    }

    /// Add a tool result message to the conversation.
    ///
    /// Tool result messages are sent after executing a client-side tool to provide
    /// the result back to the model. This enables multi-turn tool calling workflows.
    ///
    /// # Important: Message Order
    ///
    /// **Tool results must be provided in the same order as the tool calls were received.**
    /// The xAI gRPC API matches tool results to tool calls by message order, not by ID.
    /// If the model made multiple tool calls, you must provide the results in the exact
    /// same sequence.
    ///
    /// # Arguments
    ///
    /// * `tool_call_id` - The ID of the tool call this result corresponds to (from the model's tool_calls).
    ///   Note: Accepted for API compatibility but not used by xAI's gRPC API.
    /// * `content` - The result content from the tool execution (typically JSON)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use xai_grpc_client::ChatRequest;
    ///
    /// // Single tool call
    /// let request = ChatRequest::new()
    ///     .user_message("What's the weather?")
    ///     .tool_result("call_123", r#"{"temperature": 72, "condition": "sunny"}"#);
    ///
    /// // Multiple tool calls - results must be in the same order as calls
    /// let request = ChatRequest::new()
    ///     .user_message("What's the weather in Tokyo and London?")
    ///     .tool_result("call_1", r#"{"city": "Tokyo", "temp": 22}"#)      // First call result
    ///     .tool_result("call_2", r#"{"city": "London", "temp": 15}"#);   // Second call result
    /// ```
    pub fn tool_result(
        mut self,
        tool_call_id: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        self.messages.push(Message::Tool {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
        });
        self
    }

    pub fn user_multimodal(mut self, parts: Vec<ContentPart>) -> Self {
        self.messages
            .push(Message::User(MessageContent::MultiModal(parts)));
        self
    }

    pub fn user_with_image(
        mut self,
        text: impl Into<String>,
        image_url: impl Into<String>,
    ) -> Self {
        self.messages
            .push(Message::User(MessageContent::MultiModal(vec![
                ContentPart::Text(text.into()),
                ContentPart::ImageUrl {
                    url: image_url.into(),
                    detail: None,
                },
            ])));
        self
    }

    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp.clamp(0.0, 2.0));
        self
    }

    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p.clamp(0.0, 1.0));
        self
    }

    pub fn with_max_tokens(mut self, tokens: u32) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    pub fn with_reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.reasoning_effort = Some(effort);
        self
    }

    pub fn with_web_search(mut self) -> Self {
        self.search = Some(SearchConfig {
            mode: SearchMode::Auto,
            sources: vec![SearchSource::Web],
            max_results: Some(5),
        });
        self
    }

    pub fn with_json_output(mut self) -> Self {
        self.response_format = Some(ResponseFormat::JsonObject);
        self
    }

    pub fn with_json_schema(mut self, schema: JsonValue) -> Self {
        self.response_format = Some(ResponseFormat::JsonSchema(schema));
        self
    }

    // Deprecated alias for backwards compatibility
    pub fn with_structured_output(self, schema: JsonValue) -> Self {
        self.with_json_schema(schema)
    }

    pub fn with_seed(mut self, seed: i32) -> Self {
        self.seed = Some(seed);
        self
    }

    pub fn add_stop_sequence(mut self, seq: impl Into<String>) -> Self {
        self.stop.push(seq.into());
        self
    }

    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    pub fn add_tool(mut self, tool: Tool) -> Self {
        if let Some(ref mut tools) = self.tools {
            tools.push(tool);
        } else {
            self.tools = Some(vec![tool]);
        }
        self
    }

    pub fn with_tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    pub fn with_logprobs(mut self, top_logprobs: Option<i32>) -> Self {
        self.logprobs = true;
        self.top_logprobs = top_logprobs;
        self
    }

    pub fn with_frequency_penalty(mut self, penalty: f32) -> Self {
        self.frequency_penalty = Some(penalty);
        self
    }

    pub fn with_presence_penalty(mut self, penalty: f32) -> Self {
        self.presence_penalty = Some(penalty);
        self
    }

    pub fn with_parallel_tool_calls(mut self, enabled: bool) -> Self {
        self.parallel_tool_calls = Some(enabled);
        self
    }

    pub fn with_previous_response_id(mut self, id: impl Into<String>) -> Self {
        self.previous_response_id = Some(id.into());
        self
    }

    pub fn with_store_messages(mut self, store: bool) -> Self {
        self.store_messages = store;
        self
    }

    pub fn with_use_encrypted_content(mut self, use_encrypted: bool) -> Self {
        self.use_encrypted_content = use_encrypted;
        self
    }

    /// Set the maximum number of agentic tool calling turns.
    /// Useful for controlling how many iterations the model can take when using tools.
    ///
    /// # Panics
    /// Panics if `max_turns` is less than 1.
    pub fn with_max_turns(mut self, max_turns: i32) -> Self {
        assert!(
            max_turns >= 1,
            "max_turns must be at least 1, got {max_turns}"
        );
        self.max_turns = Some(max_turns);
        self
    }

    /// Add an include option to control what optional fields are returned in the response.
    /// Can be called multiple times to include multiple options.
    pub fn add_include_option(mut self, option: IncludeOption) -> Self {
        self.include.push(option);
        self
    }

    /// Set all include options at once.
    pub fn with_include_options(mut self, options: Vec<IncludeOption>) -> Self {
        self.include = options;
        self
    }

    /// Convenience method to attach a file to the message.
    pub fn user_with_file(mut self, text: impl Into<String>, file_id: impl Into<String>) -> Self {
        self.messages
            .push(Message::User(MessageContent::MultiModal(vec![
                ContentPart::Text(text.into()),
                ContentPart::File {
                    file_id: file_id.into(),
                },
            ])));
        self
    }

    // Getters for conversion
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    pub fn model(&self) -> Option<&str> {
        self.model.as_deref()
    }

    pub fn max_tokens(&self) -> Option<u32> {
        self.max_tokens
    }

    pub fn temperature(&self) -> Option<f32> {
        self.temperature
    }

    pub fn top_p(&self) -> Option<f32> {
        self.top_p
    }

    pub fn stop_sequences(&self) -> &[String] {
        &self.stop
    }

    pub fn reasoning_effort(&self) -> Option<&ReasoningEffort> {
        self.reasoning_effort.as_ref()
    }

    pub fn search_config(&self) -> Option<&SearchConfig> {
        self.search.as_ref()
    }

    pub fn seed(&self) -> Option<i32> {
        self.seed
    }

    pub fn response_format(&self) -> Option<&ResponseFormat> {
        self.response_format.as_ref()
    }

    pub fn tools(&self) -> Option<&[Tool]> {
        self.tools.as_deref()
    }

    pub fn tool_choice(&self) -> Option<&ToolChoice> {
        self.tool_choice.as_ref()
    }

    pub fn user(&self) -> Option<&str> {
        self.user.as_deref()
    }

    pub fn logprobs(&self) -> bool {
        self.logprobs
    }

    pub fn top_logprobs(&self) -> Option<i32> {
        self.top_logprobs
    }

    pub fn frequency_penalty(&self) -> Option<f32> {
        self.frequency_penalty
    }

    pub fn presence_penalty(&self) -> Option<f32> {
        self.presence_penalty
    }

    pub fn parallel_tool_calls(&self) -> Option<bool> {
        self.parallel_tool_calls
    }

    pub fn previous_response_id(&self) -> Option<&str> {
        self.previous_response_id.as_deref()
    }

    pub fn store_messages(&self) -> bool {
        self.store_messages
    }

    pub fn use_encrypted_content(&self) -> bool {
        self.use_encrypted_content
    }

    pub fn max_turns(&self) -> Option<i32> {
        self.max_turns
    }

    pub fn include_options(&self) -> &[IncludeOption] {
        &self.include
    }

    /// Create a ChatRequest from a list of messages with optional configuration
    pub fn from_messages(messages: Vec<Message>) -> Self {
        Self {
            messages,
            ..Default::default()
        }
    }

    /// Create a ChatRequest from messages and apply CompletionOptions
    /// This is the primary method used by LLMProvider trait implementations
    pub fn from_messages_with_options(messages: Vec<Message>, options: CompletionOptions) -> Self {
        Self {
            messages,
            model: options.model,
            temperature: options.temperature,
            max_tokens: options.max_tokens,
            top_p: options.top_p,
            frequency_penalty: options.frequency_penalty,
            presence_penalty: options.presence_penalty,
            stop: options.stop_sequences,
            tools: options.tools,
            tool_choice: options.tool_choice,
            response_format: options.response_format,
            ..Default::default()
        }
    }
}

impl SearchConfig {
    pub fn web() -> Self {
        Self {
            mode: SearchMode::Auto,
            sources: vec![SearchSource::Web],
            max_results: Some(5),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chat_request_builder() {
        let request = ChatRequest::new()
            .user_message("Hello, world!")
            .with_model("grok-2")
            .with_temperature(0.7)
            .with_max_tokens(100);

        assert_eq!(request.messages().len(), 1);
        assert_eq!(request.model(), Some("grok-2"));
        assert_eq!(request.temperature(), Some(0.7));
        assert_eq!(request.max_tokens(), Some(100));
    }

    #[test]
    fn test_multimodal_message() {
        let request = ChatRequest::new().user_multimodal(vec![
            ContentPart::Text("Describe this image".to_string()),
            ContentPart::ImageUrl {
                url: "https://example.com/image.jpg".to_string(),
                detail: Some(ImageDetail::High),
            },
        ]);

        assert_eq!(request.messages().len(), 1);
        match &request.messages()[0] {
            Message::User(MessageContent::MultiModal(parts)) => {
                assert_eq!(parts.len(), 2);
            }
            _ => panic!("Expected multimodal user message"),
        }
    }

    #[test]
    fn test_from_messages() {
        let messages = vec![
            Message::System("You are a helpful assistant".to_string()),
            Message::User(MessageContent::Text("Hello".to_string())),
        ];

        let request = ChatRequest::from_messages(messages);
        assert_eq!(request.messages().len(), 2);
    }

    #[test]
    fn test_from_messages_with_options() {
        let messages = vec![Message::User(MessageContent::Text("Test".to_string()))];
        let options = CompletionOptions::new()
            .with_model("grok-2")
            .with_temperature(0.8)
            .with_max_tokens(200);

        let request = ChatRequest::from_messages_with_options(messages, options);

        assert_eq!(request.messages().len(), 1);
        assert_eq!(request.model(), Some("grok-2"));
        assert_eq!(request.temperature(), Some(0.8));
        assert_eq!(request.max_tokens(), Some(200));
    }

    #[test]
    fn test_sampling_parameters() {
        let request = ChatRequest::new()
            .user_message("Test")
            .with_frequency_penalty(0.5)
            .with_presence_penalty(0.3)
            .with_top_p(0.9);

        assert_eq!(request.frequency_penalty(), Some(0.5));
        assert_eq!(request.presence_penalty(), Some(0.3));
        assert_eq!(request.top_p(), Some(0.9));
    }

    #[test]
    fn test_stop_sequences() {
        let request = ChatRequest::new()
            .user_message("Test")
            .add_stop_sequence("STOP")
            .add_stop_sequence("END");

        assert_eq!(request.stop_sequences(), &["STOP", "END"]);
    }

    #[test]
    fn test_logprobs() {
        let request = ChatRequest::new()
            .user_message("Test")
            .with_logprobs(Some(5));

        assert!(request.logprobs());
        assert_eq!(request.top_logprobs(), Some(5));
    }

    #[test]
    fn test_stored_messages() {
        let request = ChatRequest::new()
            .user_message("Test")
            .with_store_messages(true)
            .with_previous_response_id("resp_123");

        assert!(request.store_messages());
        assert_eq!(request.previous_response_id(), Some("resp_123"));
    }

    #[test]
    fn test_search_config() {
        let config = SearchConfig::web();
        assert!(matches!(config.mode, SearchMode::Auto));
        assert_eq!(config.sources.len(), 1);
        assert_eq!(config.max_results, Some(5));
    }

    #[test]
    fn test_reasoning_effort() {
        let request = ChatRequest::new()
            .user_message("Complex problem")
            .with_reasoning_effort(ReasoningEffort::High);

        assert!(matches!(
            request.reasoning_effort(),
            Some(ReasoningEffort::High)
        ));
    }

    #[test]
    fn test_json_output() {
        let request = ChatRequest::new()
            .user_message("Generate JSON")
            .with_json_output();

        assert!(matches!(
            request.response_format(),
            Some(ResponseFormat::JsonObject)
        ));
    }

    // Tests for v0.4.0 features
    #[test]
    fn test_max_turns() {
        let request = ChatRequest::new()
            .user_message("Research this topic")
            .with_max_turns(5);

        assert_eq!(request.max_turns(), Some(5));
    }

    #[test]
    fn test_max_turns_single_turn() {
        let request = ChatRequest::new()
            .user_message("Single turn")
            .with_max_turns(1);

        assert_eq!(request.max_turns(), Some(1));
    }

    #[test]
    #[should_panic(expected = "max_turns must be at least 1")]
    fn test_max_turns_validation_zero() {
        ChatRequest::new().user_message("Test").with_max_turns(0);
    }

    #[test]
    #[should_panic(expected = "max_turns must be at least 1")]
    fn test_max_turns_validation_negative() {
        ChatRequest::new().user_message("Test").with_max_turns(-1);
    }

    #[test]
    fn test_include_options_single() {
        let request = ChatRequest::new()
            .user_message("Test")
            .add_include_option(IncludeOption::WebSearchCallOutput);

        assert_eq!(request.include_options().len(), 1);
    }

    #[test]
    fn test_include_options_multiple() {
        let request = ChatRequest::new()
            .user_message("Test")
            .add_include_option(IncludeOption::WebSearchCallOutput)
            .add_include_option(IncludeOption::InlineCitations)
            .add_include_option(IncludeOption::XSearchCallOutput);

        assert_eq!(request.include_options().len(), 3);
    }

    #[test]
    fn test_with_include_options() {
        let options = vec![
            IncludeOption::WebSearchCallOutput,
            IncludeOption::CodeExecutionCallOutput,
            IncludeOption::InlineCitations,
        ];

        let request = ChatRequest::new()
            .user_message("Test")
            .with_include_options(options);

        assert_eq!(request.include_options().len(), 3);
    }

    #[test]
    fn test_user_with_file() {
        let request = ChatRequest::new().user_with_file("Analyze this document", "file-abc123");

        assert_eq!(request.messages().len(), 1);
        match &request.messages()[0] {
            Message::User(MessageContent::MultiModal(parts)) => {
                assert_eq!(parts.len(), 2);
                match &parts[0] {
                    ContentPart::Text(text) => assert_eq!(text, "Analyze this document"),
                    _ => panic!("Expected text part"),
                }
                match &parts[1] {
                    ContentPart::File { file_id } => assert_eq!(file_id, "file-abc123"),
                    _ => panic!("Expected file part"),
                }
            }
            _ => panic!("Expected multimodal user message"),
        }
    }

    #[test]
    fn test_file_content_part() {
        let file_part = ContentPart::File {
            file_id: "file-xyz789".to_string(),
        };

        match file_part {
            ContentPart::File { file_id } => assert_eq!(file_id, "file-xyz789"),
            _ => panic!("Expected file content part"),
        }
    }

    #[test]
    fn test_multimodal_with_file_and_image() {
        let request = ChatRequest::new().user_multimodal(vec![
            ContentPart::Text("Compare these".to_string()),
            ContentPart::ImageUrl {
                url: "https://example.com/image1.jpg".to_string(),
                detail: Some(ImageDetail::High),
            },
            ContentPart::File {
                file_id: "file-doc123".to_string(),
            },
        ]);

        assert_eq!(request.messages().len(), 1);
        match &request.messages()[0] {
            Message::User(MessageContent::MultiModal(parts)) => {
                assert_eq!(parts.len(), 3);
            }
            _ => panic!("Expected multimodal user message"),
        }
    }

    #[test]
    fn test_combined_new_features() {
        let request = ChatRequest::new()
            .user_message("Research and analyze")
            .with_max_turns(10)
            .add_include_option(IncludeOption::WebSearchCallOutput)
            .add_include_option(IncludeOption::InlineCitations);

        assert_eq!(request.max_turns(), Some(10));
        assert_eq!(request.include_options().len(), 2);
    }

    #[test]
    fn test_tool_result_message() {
        let request = ChatRequest::new()
            .user_message("What's the weather?")
            .tool_result(
                "call_abc123",
                r#"{"temperature": 72, "condition": "sunny"}"#,
            );

        assert_eq!(request.messages().len(), 2);
        match &request.messages()[1] {
            Message::Tool {
                tool_call_id,
                content,
            } => {
                assert_eq!(tool_call_id, "call_abc123");
                assert_eq!(content, r#"{"temperature": 72, "condition": "sunny"}"#);
            }
            _ => panic!("Expected tool result message"),
        }
    }

    #[test]
    fn test_multi_turn_tool_calling() {
        let request = ChatRequest::new()
            .user_message("Use the calculator to add 5 and 3")
            .assistant_message("I'll use the calculator tool.")
            .tool_result("call_1", r#"{"result": 8}"#)
            .assistant_message("The sum of 5 and 3 is 8.");

        assert_eq!(request.messages().len(), 4);

        // Verify the sequence
        assert!(matches!(request.messages()[0], Message::User(_)));
        assert!(matches!(request.messages()[1], Message::Assistant(_)));
        assert!(matches!(request.messages()[2], Message::Tool { .. }));
        assert!(matches!(request.messages()[3], Message::Assistant(_)));
    }

    #[test]
    fn test_tool_result_with_from_messages() {
        let messages = vec![
            Message::User(MessageContent::Text("Calculate 10 * 5".to_string())),
            Message::Tool {
                tool_call_id: "call_xyz".to_string(),
                content: r#"{"result": 50}"#.to_string(),
            },
        ];

        let request = ChatRequest::from_messages(messages);
        assert_eq!(request.messages().len(), 2);
    }
}
