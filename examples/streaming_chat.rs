use xai_grpc_client::{GrokClient, ChatRequest};
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client from GROK_API_KEY environment variable
    let mut client = GrokClient::from_env().await?;

    // Create a chat request
    let request = ChatRequest::new()
        .user_message("Write a short poem about Rust programming")
        .with_model("grok-2-1212")
        .with_max_tokens(200);

    println!("Streaming response:\n");

    // Stream the response
    let mut stream = client.stream_chat(request).await?;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        print!("{}", chunk.delta);
        std::io::Write::flush(&mut std::io::stdout())?;
    }

    println!("\n\nStream complete!");

    Ok(())
}
