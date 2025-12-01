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
use base64::Engine;

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
        proto_req.use_encrypted_content = request.use_encrypted_content();

        // Add max_turns if specified
        if let Some(max_turns) = request.max_turns() {
            proto_req.max_turns = Some(max_turns);
        }

        // Add include options
        if !request.include_options().is_empty() {
            proto_req.include = request
                .include_options()
                .iter()
                .map(|opt| *opt as i32)
                .collect();
        }

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
                    ContentPart::File { file_id } => proto::Content {
                        content: Some(proto::content::Content::File(proto::FileContent {
                            file_id: file_id.clone(),
                        })),
                    },
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

        // Extract tool calls from delta
        let tool_calls = output
            .and_then(|output| output.delta.as_ref())
            .map(|delta| {
                delta
                    .tool_calls
                    .iter()
                    .filter_map(|tc| ToolCall::from_proto(tc.clone()))
                    .collect()
            })
            .unwrap_or_default();

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

        // Extract logprobs if present
        let logprobs = output.and_then(|output| {
            output.logprobs.as_ref().map(|lp| LogProbs {
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
            })
        });

        // Extract citations (typically only in the last chunk)
        let citations = chunk.citations;

        Ok(ChatChunk {
            delta,
            finish_reason,
            cumulative_usage,
            reasoning_delta,
            tool_calls,
            logprobs,
            citations,
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

    /// Convert EmbedRequest to protobuf EmbedRequest
    pub(super) fn embed_request_to_proto(
        &self,
        request: &crate::embedding::EmbedRequest,
    ) -> proto::EmbedRequest {
        use crate::embedding::{EmbedEncodingFormat, EmbedInput};

        let input = request
            .inputs
            .iter()
            .map(|input| match input {
                EmbedInput::Text(text) => proto::EmbedInput {
                    input: Some(proto::embed_input::Input::String(text.clone())),
                },
                EmbedInput::Image { url, detail } => proto::EmbedInput {
                    input: Some(proto::embed_input::Input::ImageUrl(
                        proto::ImageUrlContent {
                            image_url: url.clone(),
                            detail: match detail {
                                ImageDetail::Auto => proto::ImageDetail::DetailAuto as i32,
                                ImageDetail::Low => proto::ImageDetail::DetailLow as i32,
                                ImageDetail::High => proto::ImageDetail::DetailHigh as i32,
                            },
                        },
                    )),
                },
            })
            .collect();

        proto::EmbedRequest {
            input,
            model: request.model.clone(),
            encoding_format: match request.encoding_format {
                EmbedEncodingFormat::Float => proto::EmbedEncodingFormat::FormatFloat as i32,
                EmbedEncodingFormat::Base64 => proto::EmbedEncodingFormat::FormatBase64 as i32,
            },
            user: request.user.clone().unwrap_or_default(),
        }
    }

    /// Convert protobuf EmbedResponse to EmbedResponse
    pub(super) fn proto_to_embed_response(
        response: proto::EmbedResponse,
    ) -> Result<crate::embedding::EmbedResponse> {
        let embeddings = response
            .embeddings
            .into_iter()
            .map(|emb| {
                // Take the first feature vector, failing if missing
                let fv =
                    emb.embeddings.into_iter().next().ok_or_else(|| {
                        GrokError::InvalidRequest("missing embedding vector".into())
                    })?;

                let vector = if !fv.float_array.is_empty() {
                    fv.float_array
                } else if !fv.base64_array.is_empty() {
                    Self::decode_base64_embedding(&fv.base64_array)?
                } else {
                    return Err(GrokError::InvalidRequest(
                        "embedding had neither float nor base64 array".into(),
                    ));
                };

                Ok(crate::embedding::Embedding {
                    index: emb.index as usize,
                    vector,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(crate::embedding::EmbedResponse {
            id: response.id,
            embeddings,
            usage: response.usage.map(Into::into).unwrap_or_default(),
            model: response.model,
            system_fingerprint: response.system_fingerprint,
        })
    }

    fn decode_base64_embedding(base64_str: &str) -> Result<Vec<f32>> {
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(base64_str)
            .map_err(|e| GrokError::InvalidRequest(format!("invalid base64 embedding: {e}")))?;

        if decoded.len() % 4 != 0 {
            return Err(GrokError::InvalidRequest(
                "embedding byte length not divisible by 4".into(),
            ));
        }

        // Convert bytes to f32 array (little-endian)
        let floats: Vec<f32> = decoded
            .chunks_exact(4)
            .map(|chunk| {
                let bytes: [u8; 4] = chunk
                    .try_into()
                    .expect("chunk size already validated as divisible by 4");
                f32::from_le_bytes(bytes)
            })
            .collect();

        Ok(floats)
    }
}
