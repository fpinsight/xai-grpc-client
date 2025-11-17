//! Example: Tokenize text to count tokens
//!
//! This example demonstrates how to use the tokenization API to:
//! - Count tokens in text
//! - Understand token boundaries
//! - Estimate costs before making API calls
//! - Debug prompt construction
//!
//! Usage:
//! ```bash
//! export XAI_API_KEY="your-api-key"
//! cargo run --example tokenize
//! ```

use xai_grpc_client::{GrokClient, TokenizeRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client from XAI_API_KEY environment variable
    let mut client = GrokClient::from_env().await?;

    // Example 1: Basic tokenization
    println!("=== Example 1: Basic Tokenization ===\n");

    let text = "Hello, world! How are you today?";
    let request = TokenizeRequest::new("grok-2-1212")
        .with_text(text);

    let response = client.tokenize(request).await?;

    println!("Text: \"{}\"", text);
    println!("Token count: {}", response.token_count());
    println!("\nTokens:");
    for (i, token) in response.tokens.iter().enumerate() {
        println!("  {}: '{}' (ID: {})", i, token.string_token, token.token_id);
    }

    // Example 2: Longer text with multiple sentences
    println!("\n\n=== Example 2: Longer Text ===\n");

    let text = "The quick brown fox jumps over the lazy dog. This is a common English pangram used for testing.";
    let request = TokenizeRequest::new("grok-2-1212")
        .with_text(text);

    let response = client.tokenize(request).await?;

    println!("Text: \"{}\"", text);
    println!("Token count: {}", response.token_count());
    println!("Reconstructed text: \"{}\"", response.text());

    // Example 3: Cost estimation
    println!("\n\n=== Example 3: Cost Estimation ===\n");

    // First, get the model pricing information
    let model = client.get_model("grok-2-1212").await?;
    println!("Model: {} v{}", model.name, model.version);

    // Tokenize a prompt
    let prompt = "Write a detailed analysis of the benefits of Rust programming language, covering safety, performance, and ecosystem.";
    let request = TokenizeRequest::new("grok-2-1212")
        .with_text(prompt);

    let response = client.tokenize(request).await?;
    let prompt_tokens = response.token_count() as u32;

    println!("\nPrompt: \"{}\"", prompt);
    println!("Prompt tokens: {}", prompt_tokens);

    // Estimate cost for different response lengths
    let completion_tokens_scenarios = vec![100, 500, 1000, 5000];

    println!("\nEstimated costs:");
    for completion_tokens in completion_tokens_scenarios {
        let cost = model.calculate_cost(prompt_tokens, completion_tokens, 0);
        println!("  {} completion tokens: ${:.6}", completion_tokens, cost);
    }

    // Example 4: Multi-line text (code example)
    println!("\n\n=== Example 4: Code Tokenization ===\n");

    let code = r#"fn main() {
    println!("Hello, world!");
    let x = 42;
    let y = x * 2;
}"#;

    let request = TokenizeRequest::new("grok-2-1212")
        .with_text(code);

    let response = client.tokenize(request).await?;

    println!("Code snippet:");
    println!("{}", code);
    println!("\nToken count: {}", response.token_count());

    // Example 5: Comparing different texts
    println!("\n\n=== Example 5: Comparing Texts ===\n");

    let texts = vec![
        "Hello!",
        "Hello, world!",
        "Hello, how are you?",
        "Good morning! How are you doing today?",
    ];

    for text in texts {
        let request = TokenizeRequest::new("grok-2-1212")
            .with_text(text);

        let response = client.tokenize(request).await?;
        println!("{:40} → {} tokens", format!("\"{}\"", text), response.token_count());
    }

    // Example 6: Using with user identifier
    println!("\n\n=== Example 6: With User Identifier ===\n");

    let request = TokenizeRequest::new("grok-2-1212")
        .with_text("Track this request for user-123")
        .with_user("user-123");

    let response = client.tokenize(request).await?;
    println!("Token count: {} (tracked for user-123)", response.token_count());

    println!("\n✓ All examples completed successfully!");

    Ok(())
}
