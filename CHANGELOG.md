# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[unreleased]: https://github.com/fpinsight/xai-grpc-client/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/fpinsight/xai-grpc-client/releases/tag/v0.1.0
