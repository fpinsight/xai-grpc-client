use serde_json::json;
use xai_grpc_client::{ChatRequest, FunctionTool, GrokClient, Tool, ToolChoice};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize client
    let mut client = GrokClient::from_env().await?;

    // Define a weather tool
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

    // Step 1: Send initial request with tool
    println!("=== Step 1: Initial request ===\n");
    let request = ChatRequest::new()
        .user_message("What's the weather in Tokyo?")
        .with_model("grok-2-1212")
        .add_tool(Tool::Function(get_weather))
        .with_tool_choice(ToolChoice::Auto);

    let response = client.complete_chat(request).await?;

    // Step 2: Check if model called the tool
    if response.tool_calls.is_empty() {
        println!("Model responded directly: {}", response.content);
        return Ok(());
    }

    println!("Model called tool:");
    for tool_call in &response.tool_calls {
        println!("  Function: {}", tool_call.function.name);
        println!("  Arguments: {}", tool_call.function.arguments);
        println!("  Call ID: {}", tool_call.id);
    }

    // Step 3: Execute the tool (simulated)
    // NOTE: In production, this is where you'd dispatch to actual tool implementations
    // (e.g., call a weather API, database query, etc.)
    println!("\n=== Step 2: Execute tool ===\n");
    let tool_result = json!({
        "temperature": 22,
        "condition": "Partly cloudy",
        "humidity": 65,
        "wind_speed": 12
    });
    println!("Tool executed, result: {}", tool_result);

    // Step 4: Send tool result back to model
    println!("\n=== Step 3: Send tool result back ===\n");

    // IMPORTANT: Tool results must be provided in the same order as tool calls were received.
    // xAI's gRPC API matches results to calls by message order, not by ID.
    // NOTE: This example handles a single tool call scenario. For multiple tool calls,
    // iterate through all calls and provide results in the same order.
    let follow_up_request = ChatRequest::new()
        .user_message("What's the weather in Tokyo?")
        .assistant_message(&response.content) // Include the assistant's tool call message
        // Safe: we verified tool_calls is non-empty at line 38
        .tool_result(&response.tool_calls[0].id, tool_result.to_string())
        .with_model("grok-2-1212");

    let final_response = client.complete_chat(follow_up_request).await?;

    println!("Model's final response:");
    println!("{}", final_response.content);

    Ok(())
}
