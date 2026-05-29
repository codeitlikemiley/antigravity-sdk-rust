#![allow(clippy::all, clippy::pedantic, dead_code, unused_variables)]

use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::connection::AnyConnection;
use antigravity_sdk_rust::conversation::Conversation;
use antigravity_sdk_rust::local::LocalConnectionStrategy;
use antigravity_sdk_rust::policy;
use antigravity_sdk_rust::tools::{Tool, ToolRunner};
use antigravity_sdk_rust::types::GeminiConfig;
use serde_json::Value;
use std::sync::Arc;

// 1. Simple Agent Example from README.md
#[allow(dead_code)]
async fn test_simple_agent_example() -> Result<(), anyhow::Error> {
    let agent = Agent::builder().allow_all().build();

    // We do not await this in tests to avoid spawning a subprocess without the binary.
    if false {
        let agent = agent.start().await?;
        let response = agent.chat("Say 'Hello World!'").await?;
        println!("Agent: {}", response.text);
        agent.stop().await?;
    }

    Ok(())
}

// 2. Advanced Usage with Conversation Example from README.md
#[allow(dead_code)]
async fn test_advanced_conversation_example() -> Result<(), anyhow::Error> {
    let tool_runner = ToolRunner::new();
    let strategy = LocalConnectionStrategy::new(
        "localharness".to_string(), // path to harness
        GeminiConfig::default(),
        Default::default(),
        None,
        None,
        vec![],
        vec![],
        Some(tool_runner),
        None,
        "my_conversation_id".to_string(),
        vec![],
    );

    if false {
        let connection = strategy.connect().await?;
        let conversation = Conversation::new(AnyConnection::Local(Arc::new(connection)), None);

        let response = conversation
            .chat_to_completion("What files are here?")
            .await?;
        println!("Agent: {}", response.text);
    }

    Ok(())
}

// 3. Custom Tools Example from README.md
#[allow(dead_code)]
struct WeatherTool;

impl Tool for WeatherTool {
    fn name(&self) -> &str {
        "get_weather"
    }

    fn description(&self) -> &str {
        "Get the current weather for a city."
    }

    fn parameters_json_schema(&self) -> &str {
        r#"{
            "type": "object",
            "properties": {
                "city": {
                    "type": "string"
                }
            },
            "required": ["city"]
        }"#
    }

    async fn call(&self, args: Value) -> Result<Value, anyhow::Error> {
        let city = args.get("city").and_then(|c| c.as_str()).unwrap_or("Tokyo");
        Ok(serde_json::json!({ "weather": format!("It's sunny in {}", city) }))
    }
}

// 4. Hooks and Policies Example from README.md
#[allow(dead_code)]
fn test_hooks_and_policies_example() {
    let _policies = vec![
        policy::deny_all(),         // Block all tools by default
        policy::allow("VIEW_FILE"), // Allow reading/viewing files
    ];
}
