//! Sample API for raw text sampling
//!
//! The Sample service provides a simpler alternative to the Chat service for basic text completion.
//! It's useful for straightforward text generation without the conversation structure.
//!
//! **Note**: For most use cases, the Chat API (`GrokClient::complete_chat`) is recommended
//! as it provides more features and better conversation management.

use crate::proto;

/// Request for text sampling
#[derive(Debug, Clone)]
pub struct SampleRequest {
    /// Text prompts to sample from
    pub prompts: Vec<String>,
    /// Model name
    pub model: String,
    /// Number of completions (1-128)
    pub n: Option<i32>,
    /// Maximum tokens to generate
    pub max_tokens: Option<i32>,
    /// Random seed for determinism
    pub seed: Option<i32>,
    /// Stop sequences
    pub stop: Vec<String>,
    /// Temperature (0-2)
    pub temperature: Option<f32>,
    /// Top-p sampling
    pub top_p: Option<f32>,
    /// Frequency penalty (-2 to 2)
    pub frequency_penalty: Option<f32>,
    /// Presence penalty (-2 to 2)
    pub presence_penalty: Option<f32>,
    /// Return log probabilities
    pub logprobs: bool,
    /// Number of top logprobs (0-8)
    pub top_logprobs: Option<i32>,
    /// User identifier
    pub user: Option<String>,
}

impl SampleRequest {
    /// Create a new sample request
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            prompts: Vec::new(),
            model: model.into(),
            n: None,
            max_tokens: None,
            seed: None,
            stop: Vec::new(),
            temperature: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            logprobs: false,
            top_logprobs: None,
            user: None,
        }
    }

    /// Add a prompt
    pub fn add_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompts.push(prompt.into());
        self
    }

    /// Set number of completions
    pub fn with_n(mut self, n: i32) -> Self {
        self.n = Some(n);
        self
    }

    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: i32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Set temperature
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature);
        self
    }
}

/// Response from sampling
#[derive(Debug, Clone)]
pub struct SampleResponse {
    /// Request ID
    pub id: String,
    /// Generated completions
    pub choices: Vec<SampleChoice>,
    /// Model used
    pub model: String,
    /// Token usage
    pub total_tokens: i32,
}

/// A single completion choice
#[derive(Debug, Clone)]
pub struct SampleChoice {
    /// Index of this choice
    pub index: i32,
    /// Generated text
    pub text: String,
    /// Finish reason
    pub finish_reason: String,
}

impl From<proto::SampleTextResponse> for SampleResponse {
    fn from(proto: proto::SampleTextResponse) -> Self {
        Self {
            id: proto.id,
            choices: proto.choices.into_iter().map(Into::into).collect(),
            model: proto.model,
            total_tokens: proto.usage.map(|u| u.total_tokens).unwrap_or(0),
        }
    }
}

impl From<proto::SampleChoice> for SampleChoice {
    fn from(proto: proto::SampleChoice) -> Self {
        let finish_reason = match proto::FinishReason::try_from(proto.finish_reason) {
            Ok(proto::FinishReason::ReasonStop) => "stop",
            Ok(proto::FinishReason::ReasonMaxLen) => "length",
            Ok(proto::FinishReason::ReasonMaxContext) => "max_context",
            Ok(proto::FinishReason::ReasonToolCalls) => "tool_calls",
            Ok(proto::FinishReason::ReasonTimeLimit) => "time_limit",
            _ => "unknown",
        };

        Self {
            index: proto.index,
            text: proto.text,
            finish_reason: finish_reason.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_request_builder() {
        let request = SampleRequest::new("grok-2-1212")
            .add_prompt("Hello, world!")
            .add_prompt("How are you?")
            .with_n(3)
            .with_max_tokens(100)
            .with_temperature(0.8);

        assert_eq!(request.model, "grok-2-1212");
        assert_eq!(request.prompts.len(), 2);
        assert_eq!(request.prompts[0], "Hello, world!");
        assert_eq!(request.prompts[1], "How are you?");
        assert_eq!(request.n, Some(3));
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.8));
    }

    #[test]
    fn test_sample_request_minimal() {
        let request = SampleRequest::new("grok-beta");

        assert_eq!(request.model, "grok-beta");
        assert_eq!(request.prompts.len(), 0);
        assert_eq!(request.n, None);
        assert_eq!(request.max_tokens, None);
        assert_eq!(request.temperature, None);
        assert!(!request.logprobs);
    }

    #[test]
    fn test_sample_choice_from_proto() {
        let proto_choice = proto::SampleChoice {
            finish_reason: proto::FinishReason::ReasonStop as i32,
            index: 0,
            text: "Hello there!".to_string(),
        };

        let choice: SampleChoice = proto_choice.into();
        assert_eq!(choice.index, 0);
        assert_eq!(choice.text, "Hello there!");
        assert_eq!(choice.finish_reason, "stop");
    }

    #[test]
    fn test_sample_choice_finish_reasons() {
        let test_cases = vec![
            (proto::FinishReason::ReasonStop, "stop"),
            (proto::FinishReason::ReasonMaxLen, "length"),
            (proto::FinishReason::ReasonMaxContext, "max_context"),
            (proto::FinishReason::ReasonToolCalls, "tool_calls"),
            (proto::FinishReason::ReasonTimeLimit, "time_limit"),
            (proto::FinishReason::ReasonInvalid, "unknown"),
        ];

        for (proto_reason, expected_str) in test_cases {
            let proto_choice = proto::SampleChoice {
                finish_reason: proto_reason as i32,
                index: 0,
                text: "test".to_string(),
            };

            let choice: SampleChoice = proto_choice.into();
            assert_eq!(choice.finish_reason, expected_str);
        }
    }

    #[test]
    fn test_sample_response_from_proto() {
        let proto_response = proto::SampleTextResponse {
            id: "req-123".to_string(),
            choices: vec![
                proto::SampleChoice {
                    finish_reason: proto::FinishReason::ReasonStop as i32,
                    index: 0,
                    text: "First choice".to_string(),
                },
                proto::SampleChoice {
                    finish_reason: proto::FinishReason::ReasonMaxLen as i32,
                    index: 1,
                    text: "Second choice".to_string(),
                },
            ],
            created: None,
            model: "grok-2-1212".to_string(),
            system_fingerprint: "fp_test".to_string(),
            usage: Some(proto::SamplingUsage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
                cached_prompt_text_tokens: 0,
                num_sources_used: 0,
                prompt_image_tokens: 0,
                reasoning_tokens: 0,
                prompt_text_tokens: 10,
                server_side_tools_used: vec![],
            }),
        };

        let response: SampleResponse = proto_response.into();
        assert_eq!(response.id, "req-123");
        assert_eq!(response.model, "grok-2-1212");
        assert_eq!(response.total_tokens, 30);
        assert_eq!(response.choices.len(), 2);
        assert_eq!(response.choices[0].text, "First choice");
        assert_eq!(response.choices[1].text, "Second choice");
    }

    #[test]
    fn test_sample_request_clone() {
        let request = SampleRequest::new("grok-2-1212")
            .add_prompt("Test")
            .with_max_tokens(100);

        let cloned = request.clone();
        assert_eq!(cloned.model, request.model);
        assert_eq!(cloned.prompts, request.prompts);
        assert_eq!(cloned.max_tokens, request.max_tokens);
    }
}
