# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.4.1] - 2025-11-30

### Fixed
- 🐛 **Critical: Fixed streaming tool calls not being extracted** (#12)
  - Tool calls are now properly extracted from streaming responses
  - Fixes application hangs when using `stream_chat()` with tools
  - Added `tool_calls: Vec<ToolCall>` field to `ChatChunk`
  - Streaming responses now match non-streaming API completeness

### Added
- ✨ **Streaming log probabilities support**
  - Added `logprobs: Option<LogProbs>` field to `ChatChunk`
  - Token-level probability information now available during streaming
  - Enables real-time confidence analysis of generated tokens
- 📚 **Streaming citations support**
  - Added `citations: Vec<String>` field to `ChatChunk`
  - Citations now accessible in streaming mode (typically in last chunk)
  - Matches non-streaming API citation support
- 🔐 **Encrypted reasoning trace support**
  - Added `use_encrypted_content: bool` field to `ChatRequest`
  - New method: `ChatRequest::with_use_encrypted_content(bool)`
  - Enables xAI's reasoning trace rehydration optimization
  - Faster and cheaper multi-turn reasoning conversations when used with `store_messages` and `previous_response_id`

### Changed
- 📊 **Enhanced `ChatChunk` structure** to match `ChatResponse` completeness
  - Before: Only 4 fields (delta, finish_reason, usage, reasoning_delta)
  - After: 7 fields (added tool_calls, logprobs, citations)
  - Backward compatible - new fields are empty by default

### Developer Notes
- All tests passing (109 tests)
- No breaking changes to existing API
- New fields are backward compatible with default empty values

## [0.4.0] - 2025-11-26

### Added
- ✨ **Agentic Tool Calling Control**: `max_turns` parameter to limit multi-turn tool calling iterations
  - New method: `ChatRequest::with_max_turns(i32)`
  - Prevents runaway agentic loops by setting a maximum number of tool calling rounds
- 📎 **File Attachment Support**: Upload and attach files to chat messages
  - New `ContentPart::File { file_id }` variant for file attachments
  - New method: `ChatRequest::user_with_file(text, file_id)`
  - Works with files uploaded via the Files API
- 🎛️ **Include Options**: Control optional response fields
  - New `IncludeOption` enum exported for public use
  - Options: web search output, X search output, code execution, collections search, document search, MCP tool output, inline citations
  - New methods:
    - `ChatRequest::add_include_option(IncludeOption)` - Add single option
    - `ChatRequest::with_include_options(Vec<IncludeOption>)` - Set all options at once
  - Useful for debugging and accessing tool outputs

### Changed
- 🔄 **Migrated to Git Submodule**: Proto definitions now tracked via `xai-org/xai-proto` submodule
  - Ensures proto files stay in sync with official xAI repository
  - Reduces maintenance burden
  - Added automated workflow to check for proto updates daily
- 📦 **Updated proto path**: Changed from `proto/` to `xai-proto/proto/` in build configuration
- 🚀 **All CI/CD workflows updated** to initialize submodules automatically

### Infrastructure
- 🤖 **Automated Proto Update Checking**: Daily workflow checks for upstream proto changes
  - Automatically creates PRs when updates are available
  - Keeps the client current with latest xAI API features

### Developer Notes
- Implements features from `xai-org/xai-proto` PRs:
  - #18: Tool outputs and inline citations
  - #17: `max_turns` for agentic conversations
  - #16: File attachment support
- All tests passing (98 tests)
- No breaking changes to existing API

### Migration Guide

**Cloning the repository:**

```bash
# New clones need --recursive flag
git clone --recursive https://github.com/fpinsight/xai-grpc-client

# Existing clones need submodule initialization
git submodule update --init --recursive
```

**Using new features:**

```rust
use xai_grpc_client::{ChatRequest, IncludeOption};

// Control agentic tool calling iterations
let request = ChatRequest::new()
    .user_message("Research this topic")
    .add_tool(my_tool)
    .with_max_turns(5);  // Limit to 5 tool calling rounds

// Attach a file to a message
let request = ChatRequest::new()
    .user_with_file("Analyze this document", "file-abc123");

// Include tool outputs in response
let request = ChatRequest::new()
    .user_message("Search for recent news")
    .add_include_option(IncludeOption::WebSearchCallOutput)
    .add_include_option(IncludeOption::InlineCitations);
```

## [0.3.0] - 2025-11-22

### Breaking Changes
- **Removed** `ring-crypto` and `aws-lc-rs-crypto` features
  - These features were removed to avoid conflicts with workspace-level rustls configuration
  - The library no longer manages rustls crypto providers directly
- **Removed** direct `rustls` dependency
  - TLS is now fully delegated to tonic's built-in features
- **Changed** feature flags for TLS root certificate selection:
  - New default: `tls-webpki-roots` (Mozilla's root certificates)
  - Available options: `tls-webpki-roots`, `tls-native-roots`, `tls-roots`

### Added
- ✨ **Flexible channel constructor**: `GrokClient::with_channel(channel, api_key)`
  - Enables "Bring Your Own Channel" pattern for maximum flexibility
  - Supports custom TLS configuration (custom CA certificates, domain validation)
  - Enables proxy support and custom middleware
  - Useful for testing with mock channels
  - See `examples/custom_tls.rs` for usage examples
- 🚀 **Convenience constructor**: `GrokClient::connect(api_key)`
  - Simpler API for programmatic client creation
  - Uses default configuration with provided API key
- 📦 **Re-exported tonic types** for advanced configuration:
  - `Channel`, `ClientTlsConfig`, `Certificate`, `Endpoint`
  - Users no longer need to add tonic as a direct dependency
- 📝 **New example**: `examples/custom_tls.rs`
  - Demonstrates custom TLS configuration
  - Shows how to use custom CA certificates
  - Explains feature flag selection

### Changed
- 🔧 **TLS configuration is now feature-based**:
  - `tls-webpki-roots` (default) - Uses Mozilla's root certificates (recommended for containers)
  - `tls-native-roots` - Uses system native certificate store (recommended for development)
  - `tls-roots` - Enables both root stores simultaneously
- 📚 **Updated README** with comprehensive TLS documentation
  - Added section on custom TLS configuration
  - Documented the new `with_channel()` API
  - Updated migration guide for breaking changes

### Fixed
- 🐛 **Resolved TLS conflicts** with parent workspace rustls configuration
  - Fixes "UnknownIssuer" errors when used in workspaces with custom TLS setups
  - Library no longer installs global `CryptoProvider`
  - Users have full control over TLS configuration via channel setup

### Migration Guide

**If you were using the old crypto provider features:**

```toml
# Old (v0.2.x)
[dependencies]
xai-grpc-client = { version = "0.2", features = ["ring-crypto", "webpki-roots"] }

# New (v0.3.x) - Default
[dependencies]
xai-grpc-client = "0.3"  # Uses tls-webpki-roots by default

# New (v0.3.x) - With native roots
[dependencies]
xai-grpc-client = { version = "0.3", features = ["tls-native-roots"], default-features = false }
```

**If you need custom TLS configuration:**

```rust
use xai_grpc_client::{GrokClient, Channel, ClientTlsConfig};
use secrecy::SecretString;

// Build custom channel with your TLS config
let tls_config = ClientTlsConfig::new().domain_name("api.x.ai");
let channel = Channel::from_static("https://api.x.ai")
    .tls_config(tls_config)?
    .connect()
    .await?;

// Use with_channel instead of new()
let api_key = SecretString::from("your-key".to_string());
let client = GrokClient::with_channel(channel, api_key);
```

## [0.2.1] - 2025-11-18

### Fixed
- Added default crypto provider for rustls to resolve compilation errors
  - The crate now provides `ring` as the default crypto provider
  - Users can opt into `aws-lc-rs` with the `aws-lc-rs-crypto` feature
  - Prevents "exactly one crypto provider must be selected" errors

### Added
- New cargo features for crypto provider selection:
  - `ring-crypto` (default) - Uses the `ring` cryptographic backend
  - `aws-lc-rs-crypto` - Uses the `aws-lc-rs` cryptographic backend
- Documentation for TLS crypto provider selection in README

### Changed
- Made `rustls` an optional dependency to enable feature-based crypto provider selection
- Updated README installation instructions to version 0.2
- Added TLS Crypto Provider section to README with usage examples

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

[unreleased]: https://github.com/fpinsight/xai-grpc-client/compare/v0.4.1...HEAD
[0.4.1]: https://github.com/fpinsight/xai-grpc-client/compare/v0.4.0...v0.4.1
[0.4.0]: https://github.com/fpinsight/xai-grpc-client/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/fpinsight/xai-grpc-client/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/fpinsight/xai-grpc-client/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/fpinsight/xai-grpc-client/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/fpinsight/xai-grpc-client/releases/tag/v0.1.0
