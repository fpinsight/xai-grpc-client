use super::config::GrokClient;
use crate::{
    error::{GrokError, Result},
    proto::{self, GetCompletionsRequest},
    request::{
        ChatRequest, ContentPart, ImageDetail, Message, MessageContent, ReasoningEffort, SearchMode,
    },
    response::{ChatChunk, ChatResponse, FinishReason, LogProb, LogProbs, TokenUsage, TopLogProb},
    tools::ToolCall,
};

impl GrokClient {
    /// Convert ChatRequest to protobuf GetCompletionsRequest
    pub(super) fn to_proto_request(&self, request: &ChatRequest) -> Result<GetCompletionsRequest> {
        let messages = request
            .messages()
            .iter()
            .map(|msg| self.message_to_proto(msg))
            .collect();

        let mut proto_req = GetCompletionsRequest {
            messages,
            model: request
                .model()
                .unwrap_or(&self.config.default_model)
                .to_string(),
            max_tokens: request.max_tokens().map(|t| t as i32),
            temperature: request.temperature(),
            top_p: request.top_p(),
            stop: request.stop_sequences().to_vec(),
            seed: request.seed(),
            ..Default::default()
        };

        // Add tools if specified
        if let Some(tools) = request.tools() {
            proto_req.tools = tools.iter().map(|tool| tool.to_proto()).collect();
        }

        // Add tool_choice if specified
        if let Some(tool_choice) = request.tool_choice() {
            proto_req.tool_choice = Some(tool_choice.to_proto());
        }

        // Add reasoning effort if specified
        if let Some(effort) = request.reasoning_effort() {
            proto_req.reasoning_effort = Some(match effort {
                ReasoningEffort::Low => 1,    // proto::ReasoningEffort::Low
                ReasoningEffort::Medium => 2, // proto::ReasoningEffort::Medium
                ReasoningEffort::High => 3,   // proto::ReasoningEffort::High
            });
        }

        // Add search config if specified
        if let Some(search) = request.search_config() {
            let search_mode = match search.mode {
                SearchMode::Off => 0,  // proto::SearchMode::Off
                SearchMode::On => 1,   // proto::SearchMode::On
                SearchMode::Auto => 2, // proto::SearchMode::Auto
            };

            proto_req.search_parameters = Some(proto::SearchParameters {
                mode: search_mode,
                max_search_results: search.max_results.map(|v| v as i32),
                ..Default::default()
            });
        }

        // Add response format if specified
        if let Some(format) = request.response_format() {
            use crate::request::ResponseFormat;
            let (format_type, schema) = match format {
                ResponseFormat::Text => (proto::FormatType::Text as i32, None),
                ResponseFormat::JsonObject => (proto::FormatType::JsonObject as i32, None),
                ResponseFormat::JsonSchema(schema) => (
                    proto::FormatType::JsonSchema as i32,
                    Some(schema.to_string()),
                ),
            };

            proto_req.response_format = Some(proto::ResponseFormat {
                format_type,
                schema,
            });
        }

        // Add new fields
        if let Some(user) = request.user() {
            proto_req.user = user.to_string();
        }

        proto_req.logprobs = request.logprobs();
        proto_req.top_logprobs = request.top_logprobs();

        if let Some(penalty) = request.frequency_penalty() {
            proto_req.frequency_penalty = Some(penalty);
        }

        if let Some(penalty) = request.presence_penalty() {
            proto_req.presence_penalty = Some(penalty);
        }

        if let Some(parallel) = request.parallel_tool_calls() {
            proto_req.parallel_tool_calls = Some(parallel);
        }

        if let Some(prev_id) = request.previous_response_id() {
            proto_req.previous_response_id = Some(prev_id.to_string());
        }

        proto_req.store_messages = request.store_messages();

        Ok(proto_req)
    }

    fn message_to_proto(&self, message: &Message) -> proto::Message {
        let (role, content_vec) = match message {
            Message::System(text) => (
                proto::MessageRole::RoleSystem,
                vec![proto::Content {
                    content: Some(proto::content::Content::Text(text.clone())),
                }],
            ),
            Message::User(content) => {
                let parts = self.message_content_to_proto(content);
                (proto::MessageRole::RoleUser, parts)
            }
            Message::Assistant(text) => (
                proto::MessageRole::RoleAssistant,
                vec![proto::Content {
                    content: Some(proto::content::Content::Text(text.clone())),
                }],
            ),
        };

        proto::Message {
            role: role as i32,
            content: content_vec,
            ..Default::default()
        }
    }

    fn message_content_to_proto(&self, content: &MessageContent) -> Vec<proto::Content> {
        match content {
            MessageContent::Text(text) => vec![proto::Content {
                content: Some(proto::content::Content::Text(text.clone())),
            }],
            MessageContent::MultiModal(parts) => parts
                .iter()
                .map(|part| match part {
                    ContentPart::Text(text) => proto::Content {
                        content: Some(proto::content::Content::Text(text.clone())),
                    },
                    ContentPart::ImageUrl { url, detail } => {
                        let detail_value = match detail {
                            Some(ImageDetail::Auto) => proto::ImageDetail::DetailAuto as i32,
                            Some(ImageDetail::Low) => proto::ImageDetail::DetailLow as i32,
                            Some(ImageDetail::High) => proto::ImageDetail::DetailHigh as i32,
                            None => proto::ImageDetail::DetailAuto as i32,
                        };

                        proto::Content {
                            content: Some(proto::content::Content::ImageUrl(
                                proto::ImageUrlContent {
                                    image_url: url.clone(),
                                    detail: detail_value,
                                },
                            )),
                        }
                    }
                })
                .collect(),
        }
    }

    pub(super) fn proto_to_response(
        &self,
        proto: proto::GetChatCompletionResponse,
    ) -> Result<ChatResponse> {
        let output = proto
            .outputs
            .first()
            .ok_or_else(|| GrokError::InvalidRequest("Response has no outputs".to_string()))?;

        let message = output
            .message
            .as_ref()
            .ok_or_else(|| GrokError::InvalidRequest("Output has no message".to_string()))?;

        let content = message.content.clone();

        // Extract reasoning content if present (convert empty string to None)
        let reasoning_content = if message.reasoning_content.is_empty() {
            None
        } else {
            Some(message.reasoning_content.clone())
        };

        // Extract tool calls from message
        let tool_calls: Vec<ToolCall> = message
            .tool_calls
            .iter()
            .filter_map(|tc| ToolCall::from_proto(tc.clone()))
            .collect();

        let finish_reason = Self::parse_finish_reason_static(output.finish_reason);

        let usage = proto
            .usage
            .map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens as u32,
                completion_tokens: u.completion_tokens as u32,
                total_tokens: u.total_tokens as u32,
            })
            .unwrap_or_default();

        // Parse logprobs if present (from output, not message)
        let logprobs = output.logprobs.as_ref().map(|lp| LogProbs {
            content: lp
                .content
                .iter()
                .map(|log_prob| LogProb {
                    token: log_prob.token.clone(),
                    logprob: log_prob.logprob,
                    bytes: log_prob.bytes.clone(),
                    top_logprobs: log_prob
                        .top_logprobs
                        .iter()
                        .map(|top| TopLogProb {
                            token: top.token.clone(),
                            logprob: top.logprob,
                            bytes: top.bytes.clone(),
                        })
                        .collect(),
                })
                .collect(),
        });

        // Parse timestamp if present
        let created = proto.created.map(|ts| ts.seconds);

        // Extract system fingerprint
        let system_fingerprint = if proto.system_fingerprint.is_empty() {
            None
        } else {
            Some(proto.system_fingerprint)
        };

        Ok(ChatResponse {
            request_id: proto.id,
            content,
            finish_reason,
            model: proto.model,
            usage,
            citations: proto.citations,
            tool_calls,
            reasoning_content,
            logprobs,
            created,
            system_fingerprint,
        })
    }

    pub(super) fn proto_chunk_to_chunk(chunk: proto::GetChatCompletionChunk) -> Result<ChatChunk> {
        let output = chunk.outputs.first();

        // Extract delta text from chunk
        let delta = output
            .and_then(|output| output.delta.as_ref().map(|d| d.content.clone()))
            .unwrap_or_default();

        // Extract reasoning delta if present
        let reasoning_delta = output
            .and_then(|output| output.delta.as_ref())
            .map(|delta| delta.reasoning_content.clone())
            .filter(|s| !s.is_empty()); // Filter out empty strings

        // Check finish_reason - only set if it's not REASON_INVALID (0)
        let finish_reason = output
            .map(|output| output.finish_reason)
            .filter(|&reason| reason != 0) // Filter out REASON_INVALID
            .map(Self::parse_finish_reason_static);

        // Usage is in the chunk itself, not cumulative in streaming
        let cumulative_usage = chunk
            .usage
            .map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens as u32,
                completion_tokens: u.completion_tokens as u32,
                total_tokens: u.total_tokens as u32,
            })
            .unwrap_or_default();

        Ok(ChatChunk {
            delta,
            finish_reason,
            cumulative_usage,
            reasoning_delta,
        })
    }

    pub(super) fn parse_finish_reason_static(reason: i32) -> FinishReason {
        // Map proto FinishReason enum to our FinishReason
        match reason {
            3 => FinishReason::Stop,                                    // REASON_STOP
            1 | 2 => FinishReason::Length, // REASON_MAX_LEN or REASON_MAX_CONTEXT
            4 => FinishReason::ToolCalls,  // REASON_TOOL_CALLS
            5 => FinishReason::Error("Time limit reached".to_string()), // REASON_TIME_LIMIT
            _ => FinishReason::Unknown,
        }
    }
}
