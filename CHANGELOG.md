# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-11-17

### Added
- Model listing API for discovering available models
  - `GrokClient::list_models()` - List all available language models
  - `GrokClient::get_model(name)` - Get detailed information about a specific model
  - `GrokClient::list_embedding_models()` - List embedding models
  - `GrokClient::get_embedding_model(name)` - Get embedding model details
  - `GrokClient::list_image_generation_models()` - List image generation models
  - `GrokClient::get_image_generation_model(name)` - Get image model details
- `models` module with comprehensive model metadata:
  - `LanguageModel` struct with pricing, capabilities, and specifications
  - `EmbeddingModel` struct with pricing and capabilities
  - `ImageGenerationModel` struct with pricing per image
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
- Tokenization API for counting tokens and cost estimation
  - `GrokClient::tokenize()` - Tokenize text to count tokens
  - `tokenize` module with `TokenizeRequest`, `TokenizeResponse`, and `Token` types
  - Useful for understanding token boundaries and estimating API costs
- Auth API for API key management
  - `GrokClient::get_api_key_info()` - Get API key status and permissions
  - `api_key` module with `ApiKeyInfo` struct
  - `ApiKeyInfo::is_active()` - Check if key is active
  - `ApiKeyInfo::status_string()` - Get human-readable status
  - Includes ACL permissions, team info, and blocking status
- Sample API for simple text completion (alternative to Chat)
  - `GrokClient::sample_text()` - Simple text sampling
  - `GrokClient::sample_text_streaming()` - Streaming text sampling
  - `sample` module with `SampleRequest`, `SampleResponse`, `SampleChoice`
- Image Generation API
  - `GrokClient::generate_image()` - Generate images from text prompts
  - `image` module with `ImageGenerationRequest`, `ImageGenerationResponse`, `GeneratedImage`
  - Supports text-to-image and image-to-image generation
  - Base64 and URL output formats
- Documents Search API for RAG
  - `GrokClient::search_documents()` - Search documents in collections
  - `documents` module with `DocumentSearchRequest`, `DocumentSearchResponse`, `SearchMatch`
  - L2 distance and cosine similarity ranking metrics
- New examples:
  - `list_models` - Model discovery and pricing calculation
  - `embeddings` - Text embedding with cosine similarity
  - `tokenize` - Token counting and cost estimation
- 59 additional unit tests (models: 12, embeddings: 7, tokenize: 7, api_key: 12, sample: 6, image: 10, documents: 8, total: 98 tests)

### Changed
- Updated `build.rs` to compile all proto files: `chat.proto`, `models.proto`, `embed.proto`, `tokenize.proto`, `auth.proto`, `sample.proto`, `image.proto`, `documents.proto`
- Enhanced documentation with detailed pricing unit explanations
- Enhanced README with comprehensive API coverage breakdown showing 100% coverage
- Added detailed feature coverage section showing all implemented services
- Updated examples section in README with tokenization and API key info
- Added `base64` dependency for embedding decoding

### API Coverage
This release achieves **100% (19/19)** API coverage - complete implementation of all xAI Grok API services! 🎉
- ✅ Chat Service: 100% (6/6 RPCs)
- ✅ Models Service: 100% (6/6 RPCs)
- ✅ Embeddings Service: 100% (1/1 RPCs)
- ✅ Tokenize Service: 100% (1/1 RPCs)
- ✅ Auth Service: 100% (1/1 RPCs)
- ✅ Sample Service: 100% (2/2 RPCs)
- ✅ Image Service: 100% (1/1 RPCs)
- ✅ Documents Service: 100% (1/1 RPCs)

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
