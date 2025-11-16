use xai_grpc_client::GrokClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client from XAI_API_KEY environment variable
    let mut client = GrokClient::from_env().await?;

    // List all available models
    println!("Available Models:\n");
    let models = client.list_models().await?;

    for model in &models {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("Name: {}", model.name);
        println!("Version: {}", model.version);

        if !model.aliases.is_empty() {
            println!("Aliases: {}", model.aliases.join(", "));
        }

        println!("Max Context: {} tokens", model.max_prompt_length);
        println!("Multimodal: {}", if model.supports_multimodal() { "Yes" } else { "No" });

        // Display pricing
        println!("\nPricing:");
        println!("  Prompt: ${:.4}/1M tokens",
            model.prompt_text_token_price as f64 / 100.0 / 1_000_000.0);
        println!("  Completion: ${:.4}/1M tokens",
            model.completion_text_token_price as f64 / 100.0 / 1_000_000.0);

        if model.cached_prompt_token_price > 0 {
            println!("  Cached prompt: ${:.4}/100M tokens",
                model.cached_prompt_token_price as f64);
        }

        if model.search_price > 0 {
            println!("  Search: ${:.4}/1M searches",
                model.search_price as f64 / 100.0 / 1_000_000.0);
        }

        // Calculate example cost
        let cost = model.calculate_cost(10000, 1000, 0);
        println!("\nExample: 10K prompt + 1K completion = ${:.4}", cost);
        println!();
    }

    // Get specific model details
    println!("\n\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("Detailed info for grok-2-1212:");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let model = client.get_model("grok-2-1212").await?;
    println!("Name: {}", model.name);
    println!("Version: {}", model.version);
    println!("System Fingerprint: {}", model.system_fingerprint);
    println!("Max Context Length: {} tokens", model.max_prompt_length);
    println!("\nSupported Modalities:");
    println!("  Input: {:?}", model.input_modalities);
    println!("  Output: {:?}", model.output_modalities);

    Ok(())
}
