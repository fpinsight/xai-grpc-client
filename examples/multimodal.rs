use xai_grpc_client::{ChatRequest, GrokClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let mut client = GrokClient::from_env().await?;

    // Example with a publicly accessible image URL
    let image_url = "https://upload.wikimedia.org/wikipedia/commons/thumb/d/d5/Rust_programming_language_black_logo.svg/1200px-Rust_programming_language_black_logo.svg.png";

    // Create multimodal request
    let request = ChatRequest::new()
        .user_with_image("What's in this image?", image_url)
        .with_model("grok-2-vision-1212")
        .with_max_tokens(200);

    println!("Analyzing image...\n");

    let response = client.complete_chat(request).await?;

    println!("Response: {}", response.content);
    println!("\nToken usage: {}", response.usage.total_tokens);

    Ok(())
}
