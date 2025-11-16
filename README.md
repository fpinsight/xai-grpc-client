# xai-grpc-client

[![CI](https://github.com/fpinsight/xai-grpc-client/workflows/CI/badge.svg)](https://github.com/fpinsight/xai-grpc-client/actions)
[![Crates.io](https://img.shields.io/crates/v/xai-grpc-client.svg)](https://crates.io/crates/xai-grpc-client)
[![Documentation](https://docs.rs/xai-grpc-client/badge.svg)](https://docs.rs/xai-grpc-client)
[![License](https://img.shields.io/crates/l/xai-grpc-client.svg)](https://github.com/fpinsight/xai-grpc-client#license)
[![Rust](https://img.shields.io/badge/rust-1.82%2B-blue.svg?maxAge=3600)](https://github.com/fpinsight/xai-grpc-client)

Unofficial Rust client for [xAI's Grok API](https://docs.x.ai/) with full gRPC support.

## Features

- 🚀 **Async/await API** - Built on tokio and tonic for high performance
- 🔒 **Type-safe** - Strongly typed request builders with compile-time guarantees
- 📡 **Streaming support** - Real-time response streaming with tokio-stream
- 🔧 **Tool calling** - Function calling with 7 tool types (function, web search, X search, MCP, etc.)
- 🖼️ **Multimodal** - Text and image inputs for vision capabilities
- 🧠 **Advanced features** - Log probabilities, reasoning traces, deferred completions
- 📋 **Model discovery** - List available models with pricing and capabilities
- 🔐 **Secure by default** - Uses `secrecy` crate to protect API keys in memory
- ✅ **Comprehensive** - ~95% coverage of Grok API proto features
- 🧪 **Well-tested** - 51 unit tests covering all core modules

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
xai-grpc-client = "0.1"
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
```

## Quick Start

```rust
use xai_grpc_client::{GrokClient, ChatRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client from GROK_API_KEY environment variable
    let mut client = GrokClient::from_env().await?;

    // Create a simple chat request
    let request = ChatRequest::new()
        .user_message("What is the meaning of life?")
        .with_model("grok-2-1212")
        .with_max_tokens(100);

    // Get response
    let response = client.complete_chat(request).await?;
    println!("{}", response.content);

    Ok(())
}
```

Set your API key:
```bash
export GROK_API_KEY="your-api-key-here"
```

## Examples

### Streaming Chat

Stream responses in real-time:

```rust
use xai_grpc_client::{GrokClient, ChatRequest};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GrokClient::from_env().await?;

    let request = ChatRequest::new()
        .user_message("Write a short poem about Rust");

    let mut stream = client.stream_chat(request).await?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        print!("{}", chunk.delta);
    }

    Ok(())
}
```

### Tool Calling (Function Calling)

Enable the model to call functions:

```rust
use xai_grpc_client::{GrokClient, ChatRequest, FunctionTool, Tool, ToolChoice};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GrokClient::from_env().await?;

    // Define a function tool
    let get_weather = FunctionTool::new(
        "get_weather",
        "Get the current weather in a location"
    )
    .with_parameters(json!({
        "type": "object",
        "properties": {
            "location": {
                "type": "string",
                "description": "City name"
            },
            "unit": {
                "type": "string",
                "enum": ["celsius", "fahrenheit"]
            }
        },
        "required": ["location"]
    }));

    let request = ChatRequest::new()
        .user_message("What's the weather in Tokyo?")
        .add_tool(Tool::Function(get_weather))
        .with_tool_choice(ToolChoice::Auto);

    let response = client.complete_chat(request).await?;

    // Check if model called the tool
    if !response.tool_calls.is_empty() {
        for tool_call in &response.tool_calls {
            println!("Function: {}", tool_call.function.name);
            println!("Arguments: {}", tool_call.function.arguments);
        }
    }

    Ok(())
}
```

### Multimodal (Vision)

Send images with your prompts:

```rust
use xai_grpc_client::{GrokClient, ChatRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GrokClient::from_env().await?;

    let request = ChatRequest::new()
        .user_with_image(
            "What's in this image?",
            "https://example.com/image.jpg"
        )
        .with_model("grok-2-vision-1212");

    let response = client.complete_chat(request).await?;
    println!("{}", response.content);

    Ok(())
}
```

### Web Search

Enable web search for up-to-date information:

```rust
use xai_grpc_client::{GrokClient, ChatRequest, Tool, WebSearchTool};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GrokClient::from_env().await?;

    let request = ChatRequest::new()
        .user_message("What are the latest developments in AI?")
        .add_tool(Tool::WebSearch(WebSearchTool::new()));

    let response = client.complete_chat(request).await?;

    println!("Response: {}", response.content);

    if !response.citations.is_empty() {
        println!("\nSources:");
        for citation in &response.citations {
            println!("  - {}", citation);
        }
    }

    Ok(())
}
```

### Model Listing

List available models and get pricing information:

```rust
use xai_grpc_client::GrokClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GrokClient::from_env().await?;

    // List all available models
    let models = client.list_models().await?;
    for model in models {
        println!("{}: {} (max {} tokens)",
            model.name, model.version, model.max_prompt_length);

        // Calculate cost for a request
        let cost = model.calculate_cost(10000, 1000, 0);
        println!("  Example cost: ${:.4}", cost);
    }

    // Get specific model details
    let model = client.get_model("grok-2-1212").await?;
    println!("Model: {} v{}", model.name, model.version);

    Ok(())
}
```

### Advanced: Deferred Completions

For long-running tasks, start a deferred completion and poll for results:

```rust
use xai_grpc_client::{GrokClient, ChatRequest};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GrokClient::from_env().await?;

    let request = ChatRequest::new()
        .user_message("Write a detailed analysis of quantum computing")
        .with_reasoning_effort(ReasoningEffort::High);

    // Start deferred completion
    let request_id = client.start_deferred(request).await?;
    println!("Started deferred completion: {}", request_id);

    // Wait for completion with polling
    let response = client.wait_for_deferred(
        request_id,
        Duration::from_secs(2),  // poll interval
        Duration::from_secs(300) // timeout
    ).await?;

    println!("{}", response.content);

    Ok(())
}
```

### CompletionOptions (for trait abstraction)

Use `CompletionOptions` to create reusable configurations:

```rust
use xai_grpc_client::{GrokClient, ChatRequest, CompletionOptions, Message, MessageContent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GrokClient::from_env().await?;

    // Define reusable options
    let options = CompletionOptions::new()
        .with_model("grok-2-1212")
        .with_temperature(0.7)
        .with_max_tokens(500);

    // Use with different message sets
    let messages = vec![
        Message::System("You are a helpful coding assistant".to_string()),
        Message::User(MessageContent::Text("Explain Rust ownership".into()))
    ];

    let request = ChatRequest::from_messages_with_options(messages, options);
    let response = client.complete_chat(request).await?;

    println!("{}", response.content);

    Ok(())
}
```

## API Coverage

This library covers approximately 95% of the Grok API features:

**Implemented:**
- ✅ Chat completions (streaming & non-streaming)
- ✅ Tool calling (7 tool types)
- ✅ Multimodal inputs (text + images)
- ✅ Advanced sampling (temperature, top_p, penalties)
- ✅ Reasoning effort control
- ✅ Log probabilities
- ✅ Response format (JSON, JSON schema)
- ✅ Web search integration
- ✅ X (Twitter) search
- ✅ Deferred completions
- ✅ Stored completions
- ✅ Stop sequences
- ✅ Seed for reproducibility

**Not yet implemented:**
- ⏳ Embeddings API
- ⏳ Image generation API
- ⏳ Model listing API

## Error Handling

The library provides comprehensive error handling with retry logic:

```rust
use xai_grpc_client::{GrokClient, ChatRequest, GrokError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = GrokClient::from_env().await?;

    let request = ChatRequest::new()
        .user_message("Hello!");

    match client.complete_chat(request).await {
        Ok(response) => println!("{}", response.content),
        Err(GrokError::RateLimit { retry_after_secs }) => {
            println!("Rate limited. Retry after {} seconds", retry_after_secs);
        }
        Err(GrokError::Auth(msg)) => {
            println!("Authentication error: {}", msg);
        }
        Err(e) if e.is_retryable() => {
            println!("Retryable error: {}", e);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }

    Ok(())
}
```

## Configuration

### From Environment Variable

```rust
let client = GrokClient::from_env().await?;
```

Requires `GROK_API_KEY` environment variable.

### Manual Configuration

```rust
use xai_grpc_client::{GrokClient, GrokConfig};
use secrecy::SecretString;
use std::time::Duration;

let config = GrokConfig {
    endpoint: "https://api.x.ai".to_string(),
    api_key: SecretString::from("your-api-key".to_string()),
    default_model: "grok-2-1212".to_string(),
    timeout: Duration::from_secs(120),
};

let client = GrokClient::new(config).await?;
```

## Available Models

- `grok-2-1212` - Latest Grok 2 (December 2024)
- `grok-2-vision-1212` - Grok 2 with vision capabilities
- `grok-beta` - Beta model with experimental features

Check [xAI's documentation](https://docs.x.ai/docs/models) for the latest model list.

## Testing

Run the test suite:

```bash
# Unit tests (no API key required)
cargo test --lib

# Integration tests (requires GROK_API_KEY)
cargo test --test integration_test

# Run specific test
cargo test test_chat_request_builder
```

The library includes 39 comprehensive unit tests covering:
- Request building and validation
- Response parsing
- Error handling and retry logic
- Tool configuration
- Multimodal messages

## Examples

See the [examples/](examples/) directory for more complete examples:

```bash
# Simple chat
cargo run --example simple_chat

# Streaming
cargo run --example streaming_chat

# Tool calling
cargo run --example tool_calling

# Multimodal
cargo run --example multimodal
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

```bash
git clone https://github.com/fpinsight/xai-grpc-client
cd xai-grpc-client
cargo build
cargo test
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Disclaimer

This is an unofficial client library and is not affiliated with or endorsed by xAI. Use at your own risk.

## Links

- [xAI's Official Documentation](https://docs.x.ai/)
- [Grok API Documentation](https://docs.x.ai/docs)
- [crates.io](https://crates.io/crates/xai-grpc-client)
- [Documentation](https://docs.rs/xai-grpc-client)
- [Repository](https://github.com/fpinsight/xai-grpc-client)

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for release history.
