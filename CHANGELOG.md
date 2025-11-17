# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-01-16

### Added
- Model listing API for discovering available models
  - `GrokClient::list_models()` - List all available language models
  - `GrokClient::get_model(name)` - Get detailed information about a specific model
- `models` module with comprehensive model metadata:
  - `LanguageModel` struct with pricing, capabilities, and specifications
  - `Modality` enum for input/output modality types (Text, Image, Embedding)
  - `calculate_cost()` method for estimating request costs
  - `supports_multimodal()` method for checking vision capabilities
- Embedding API for generating vector representations
  - `GrokClient::embed()` - Generate embeddings from text or images
  - `embedding` module with request/response types:
    - `EmbedRequest` builder for creating embedding requests
    - `EmbedInput` enum supporting text and image inputs
    - `EmbedResponse`, `Embedding`, `EmbeddingUsage` types
    - `EmbedEncodingFormat` for Float or Base64 output
    - Base64 embedding decoding support
- New examples:
  - `list_models` - Model discovery and pricing calculation
  - `embeddings` - Text embedding with cosine similarity
- 19 additional unit tests (12 models + 7 embeddings, total: 58 tests)

### Changed
- Updated `build.rs` to compile `chat.proto`, `models.proto`, and `embed.proto`
- Enhanced documentation with detailed pricing unit explanations
- Updated README with models and embeddings features
- Added `base64` dependency for embedding decoding

## [0.1.0] - 2024-11-16

### Added
- Initial release of xai-grpc-client
- Async chat completions with tokio and tonic
- Streaming support for real-time responses
- Tool calling with 7 tool types:
  - Function tools
  - Web search
  - X (Twitter) search
  - MCP (Model Context Protocol) tools
  - Collections search
  - Document search
  - Code execution
- Multimodal support (text + images)
- Advanced features:
  - Log probabilities
  - Reasoning effort control
  - Response format (JSON, JSON schema)
  - Deferred completions
  - Stored completions
  - Sampling parameters (temperature, top_p, penalties)
- CompletionOptions for trait-based abstraction
- Comprehensive error handling with retry logic
- API key security with secrecy crate
- 39 unit tests covering core functionality
- Full documentation with examples

### Security
- API keys stored using `secrecy::Secret` to prevent accidental exposure
- TLS support for secure gRPC connections

[unreleased]: https://github.com/fpinsight/xai-grpc-client/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/fpinsight/xai-grpc-client/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/fpinsight/xai-grpc-client/releases/tag/v0.1.0
