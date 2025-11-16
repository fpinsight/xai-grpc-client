use serde_json::json;
use xai_grpc_client::{ChatRequest, FunctionTool, GrokClient, Tool, ToolChoice};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let mut client = GrokClient::from_env().await?;

    // Define a function tool
    let get_weather = FunctionTool::new("get_weather", "Get the current weather in a location")
        .with_parameters(json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "City name"
                },
                "unit": {
                    "type": "string",
                    "enum": ["celsius", "fahrenheit"],
                    "description": "Temperature unit"
                }
            },
            "required": ["location"]
        }));

    // Create request with tool
    let request = ChatRequest::new()
        .user_message("What's the weather in Tokyo?")
        .with_model("grok-2-1212")
        .add_tool(Tool::Function(get_weather))
        .with_tool_choice(ToolChoice::Auto);

    println!("Sending request with tool...\n");

    let response = client.complete_chat(request).await?;

    // Check if model called the tool
    if !response.tool_calls.is_empty() {
        println!("Tool was called!");
        for tool_call in &response.tool_calls {
            println!("\nFunction: {}", tool_call.function.name);
            println!("Arguments: {}", tool_call.function.arguments);
        }
    } else {
        println!("Response: {}", response.content);
    }

    Ok(())
}
