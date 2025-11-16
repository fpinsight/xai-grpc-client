use crate::tools::ToolCall;

#[derive(Clone, Debug)]
pub struct ChatResponse {
    pub request_id: String,
    pub content: String,
    pub finish_reason: FinishReason,
    pub model: String,
    pub usage: TokenUsage,
    pub citations: Vec<String>,
    pub tool_calls: Vec<ToolCall>,
    /// Reasoning trace the model produced before the final answer
    pub reasoning_content: Option<String>,
    /// Log probabilities for the generated tokens (if requested)
    pub logprobs: Option<LogProbs>,
    /// Timestamp when response was created
    pub created: Option<i64>,
    /// Backend configuration fingerprint
    pub system_fingerprint: Option<String>,
}

#[derive(Clone, Debug)]
pub struct LogProbs {
    pub content: Vec<LogProb>,
}

#[derive(Clone, Debug)]
pub struct LogProb {
    pub token: String,
    pub logprob: f32,
    pub bytes: Vec<u8>,
    pub top_logprobs: Vec<TopLogProb>,
}

#[derive(Clone, Debug)]
pub struct TopLogProb {
    pub token: String,
    pub logprob: f32,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct ChatChunk {
    pub delta: String,
    pub finish_reason: Option<FinishReason>,
    pub cumulative_usage: TokenUsage,
    /// Reasoning trace delta (for streaming)
    pub reasoning_delta: Option<String>,
}

#[derive(Clone, Debug, Default)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Clone, Debug)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Error(String),
    Unknown,
}

impl std::fmt::Display for FinishReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stop => write!(f, "stop"),
            Self::Length => write!(f, "length"),
            Self::ToolCalls => write!(f, "tool_calls"),
            Self::ContentFilter => write!(f, "content_filter"),
            Self::Error(e) => write!(f, "error: {e}"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_usage_default() {
        let usage = TokenUsage::default();
        assert_eq!(usage.prompt_tokens, 0);
        assert_eq!(usage.completion_tokens, 0);
        assert_eq!(usage.total_tokens, 0);
    }

    #[test]
    fn test_finish_reason_display() {
        assert_eq!(FinishReason::Stop.to_string(), "stop");
        assert_eq!(FinishReason::Length.to_string(), "length");
        assert_eq!(FinishReason::ToolCalls.to_string(), "tool_calls");
        assert_eq!(FinishReason::ContentFilter.to_string(), "content_filter");
        assert_eq!(
            FinishReason::Error("timeout".to_string()).to_string(),
            "error: timeout"
        );
        assert_eq!(FinishReason::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_chat_response_creation() {
        let response = ChatResponse {
            request_id: "req_123".to_string(),
            content: "Hello, world!".to_string(),
            finish_reason: FinishReason::Stop,
            model: "grok-2".to_string(),
            usage: TokenUsage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            },
            citations: vec!["https://example.com".to_string()],
            tool_calls: vec![],
            reasoning_content: None,
            logprobs: None,
            created: Some(1234567890),
            system_fingerprint: Some("fp_abc123".to_string()),
        };

        assert_eq!(response.request_id, "req_123");
        assert_eq!(response.content, "Hello, world!");
        assert_eq!(response.usage.total_tokens, 15);
        assert_eq!(response.citations.len(), 1);
    }

    #[test]
    fn test_chat_chunk() {
        let chunk = ChatChunk {
            delta: "Hello".to_string(),
            finish_reason: None,
            cumulative_usage: TokenUsage {
                prompt_tokens: 5,
                completion_tokens: 1,
                total_tokens: 6,
            },
            reasoning_delta: None,
        };

        assert_eq!(chunk.delta, "Hello");
        assert!(chunk.finish_reason.is_none());
        assert_eq!(chunk.cumulative_usage.total_tokens, 6);
    }

    #[test]
    fn test_log_probs() {
        let logprobs = LogProbs {
            content: vec![LogProb {
                token: "hello".to_string(),
                logprob: -0.5,
                bytes: vec![104, 101, 108, 108, 111],
                top_logprobs: vec![
                    TopLogProb {
                        token: "hi".to_string(),
                        logprob: -1.0,
                        bytes: vec![104, 105],
                    },
                    TopLogProb {
                        token: "hey".to_string(),
                        logprob: -1.5,
                        bytes: vec![104, 101, 121],
                    },
                ],
            }],
        };

        assert_eq!(logprobs.content.len(), 1);
        assert_eq!(logprobs.content[0].token, "hello");
        assert_eq!(logprobs.content[0].top_logprobs.len(), 2);
    }

    #[test]
    fn test_response_with_reasoning() {
        let response = ChatResponse {
            request_id: "req_456".to_string(),
            content: "The answer is 42".to_string(),
            finish_reason: FinishReason::Stop,
            model: "grok-2".to_string(),
            usage: TokenUsage::default(),
            citations: vec![],
            tool_calls: vec![],
            reasoning_content: Some("First, I considered...".to_string()),
            logprobs: None,
            created: None,
            system_fingerprint: None,
        };

        assert!(response.reasoning_content.is_some());
        assert_eq!(
            response.reasoning_content.unwrap(),
            "First, I considered..."
        );
    }
}
