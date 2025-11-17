use xai_grpc_client::{EmbedRequest, GrokClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client from XAI_API_KEY environment variable
    let mut client = GrokClient::from_env().await?;

    println!("=== Text Embeddings Example ===\n");

    // Create an embedding request with multiple text inputs
    let request = EmbedRequest::new("embed-large-v1")
        .add_text("The quick brown fox jumps over the lazy dog")
        .add_text("Machine learning is a subset of artificial intelligence")
        .add_text("Rust is a systems programming language");

    let response = client.embed(request).await?;

    println!("Request ID: {}", response.id);
    println!("Model: {}", response.model);
    println!("System Fingerprint: {}\n", response.system_fingerprint);

    // Display embeddings
    for embedding in &response.embeddings {
        println!("Embedding {}:", embedding.index);
        println!("  Dimensions: {}", embedding.vector.len());
        println!(
            "  First 5 values: {:?}",
            &embedding.vector[..5.min(embedding.vector.len())]
        );
        println!();
    }

    // Display usage statistics
    println!("Usage Statistics:");
    println!("  Text embeddings: {}", response.usage.num_text_embeddings);
    println!(
        "  Image embeddings: {}",
        response.usage.num_image_embeddings
    );
    println!();

    // Calculate cosine similarity between first two embeddings
    if response.embeddings.len() >= 2 {
        let similarity = cosine_similarity(
            &response.embeddings[0].vector,
            &response.embeddings[1].vector,
        );
        println!("Cosine similarity between embeddings 0 and 1: {similarity:.4}");
    }

    println!("\n=== Multimodal Embeddings Example ===\n");

    // Example with both text and images (if you have a multimodal model)
    let _multimodal_request = EmbedRequest::new("embed-vision-v1")
        .add_text("A cat sitting on a mat")
        .add_image("https://example.com/cat.jpg");

    // Note: This will fail if the URL doesn't exist or the model doesn't support it
    // Uncomment to test with a real image URL and model
    /*
    let multimodal_response = client.embed(multimodal_request).await?;
    println!("Generated {} multimodal embeddings", multimodal_response.embeddings.len());
    println!("  Text: {}", multimodal_response.usage.num_text_embeddings);
    println!("  Images: {}", multimodal_response.usage.num_image_embeddings);
    */

    println!("Example completed successfully!");
    Ok(())
}

/// Calculate cosine similarity between two vectors
fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() {
        return 0.0;
    }

    let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let magnitude_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let magnitude_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

    if magnitude_a == 0.0 || magnitude_b == 0.0 {
        return 0.0;
    }

    dot_product / (magnitude_a * magnitude_b)
}
