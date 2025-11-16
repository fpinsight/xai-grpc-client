use xai_grpc_client::{ChatRequest, GrokClient};

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

    println!("Response: {}", response.content);
    println!("\nToken usage:");
    println!("  Prompt: {}", response.usage.prompt_tokens);
    println!("  Completion: {}", response.usage.completion_tokens);
    println!("  Total: {}", response.usage.total_tokens);

    Ok(())
}
