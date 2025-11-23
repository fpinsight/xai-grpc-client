//! Example: Custom TLS Configuration with "Bring Your Own Channel"
//!
//! This example demonstrates how to use the flexible `with_channel()` constructor
//! to provide custom TLS configuration, including:
//! - Custom domain validation
//! - Custom CA certificates
//! - Native vs WebPKI root stores
//! - Custom timeouts and connection settings
//!
//! Run with:
//! ```bash
//! cargo run --example custom_tls --features tls-native-roots
//! ```
//!
//! Or with webpki roots (default):
//! ```bash
//! cargo run --example custom_tls
//! ```

use secrecy::SecretString;
use std::time::Duration;
use xai_grpc_client::{Channel, ChatRequest, ClientTlsConfig, GrokClient};

// Only needed if using custom CA certificates (see example 2)
#[allow(unused_imports)]
use xai_grpc_client::Certificate;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get API key from environment
    let api_key =
        std::env::var("XAI_API_KEY").expect("XAI_API_KEY environment variable must be set");
    let api_key = SecretString::from(api_key);

    println!("Example 1: Custom TLS with explicit domain validation");
    println!("=====================================================\n");

    // Build a custom TLS configuration with explicit domain validation
    let tls_config = ClientTlsConfig::new().domain_name("api.x.ai"); // Explicitly validate against this domain

    let channel = Channel::from_static("https://api.x.ai")
        .timeout(Duration::from_secs(120))
        .tcp_keepalive(Some(Duration::from_secs(30)))
        .http2_keep_alive_interval(Duration::from_secs(30))
        .keep_alive_timeout(Duration::from_secs(10))
        .tls_config(tls_config)?
        .connect()
        .await?;

    let mut client = GrokClient::with_channel(channel, api_key.clone());

    let request = ChatRequest::new()
        .user_message("Say hello! Keep it brief.")
        .with_model("grok-code-fast-1")
        .with_max_tokens(50);

    let response = client.complete_chat(request).await?;
    println!("Response: {}\n", response.content);

    // Example 2: Using custom CA certificate (uncomment to use)
    // This is useful when you have a custom CA or are in a corporate environment
    println!("Example 2: How to use custom CA certificates (see code)");
    println!("========================================================\n");
    println!("To use a custom CA certificate, uncomment the code below:");
    println!();
    println!("```rust");
    println!("let ca_cert = std::fs::read(\"path/to/ca.pem\")?;");
    println!("let ca = Certificate::from_pem(ca_cert);");
    println!();
    println!("let tls_config = ClientTlsConfig::new()");
    println!("    .ca_certificate(ca)");
    println!("    .domain_name(\"api.x.ai\");");
    println!();
    println!("let channel = Channel::from_static(\"https://api.x.ai\")");
    println!("    .tls_config(tls_config)?");
    println!("    .connect()");
    println!("    .await?;");
    println!();
    println!("let client = GrokClient::with_channel(channel, api_key);");
    println!("```");
    println!();

    // Uncomment this section if you have a custom CA certificate:
    /*
    let ca_cert = std::fs::read("/path/to/your/ca.pem")?;
    let ca = Certificate::from_pem(ca_cert);

    let tls_config = ClientTlsConfig::new()
        .ca_certificate(ca)
        .domain_name("api.x.ai");

    let channel = Channel::from_static("https://api.x.ai")
        .tls_config(tls_config)?
        .connect()
        .await?;

    let mut client = GrokClient::with_channel(channel, api_key);

    let request = ChatRequest::new()
        .user_message("Hello with custom CA!")
        .with_max_tokens(50);

    let response = client.complete_chat(request).await?;
    println!("Response with custom CA: {}\n", response.content);
    */

    println!("Example 3: Feature flag selection");
    println!("===================================\n");
    println!("You can choose which root certificate store to use:");
    println!();
    println!("Default (webpki-roots):");
    println!("  cargo run --example custom_tls");
    println!();
    println!("Native system roots (recommended for development):");
    println!("  cargo run --example custom_tls --features tls-native-roots --no-default-features");
    println!();
    println!("Both root stores (use if unsure):");
    println!("  cargo run --example custom_tls --features tls-roots --no-default-features");
    println!();

    Ok(())
}
