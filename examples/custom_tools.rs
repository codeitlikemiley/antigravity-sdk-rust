use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::policy;
use antigravity_sdk_rust::tool_context::ToolContext;
use antigravity_sdk_rust::tools::Tool;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing_subscriber::EnvFilter;

struct LookupSkuTool;

impl Tool for LookupSkuTool {
    fn name(&self) -> &'static str {
        "lookup_fruit_sku"
    }

    fn description(&self) -> &'static str {
        "Looks up the SKU for a given fruit."
    }

    fn parameters_json_schema(&self) -> &'static str {
        r#"{
            "type": "object",
            "properties": {
                "fruit_name": {
                    "type": "string"
                }
            },
            "required": ["fruit_name"]
        }"#
    }

    async fn call(&self, args: Value) -> Result<Value, anyhow::Error> {
        let fruit_name = args
            .get("fruit_name")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing fruit_name"))?
            .to_lowercase();

        let sku = match fruit_name.trim_end_matches('s') {
            "apple" => "SKU-APP-123",
            "banana" => "SKU-BAN-456",
            "orange" => "SKU-ORA-789",
            _ => "SKU-GEN-000",
        };

        Ok(Value::String(format!(
            "SKU for {} is {}. Order ID for restocking: ORD-{}-NEW",
            fruit_name, sku, sku
        )))
    }
}

struct RecordFruitTool {
    inventory: Arc<Mutex<HashMap<String, u32>>>,
}

impl Tool for RecordFruitTool {
    fn name(&self) -> &'static str {
        "record_fruit"
    }

    fn description(&self) -> &'static str {
        "Records the count of fruits by SKU."
    }

    fn parameters_json_schema(&self) -> &'static str {
        r#"{
            "type": "object",
            "properties": {
                "sku": {
                    "type": "string"
                },
                "count": {
                    "type": "integer"
                }
            },
            "required": ["sku", "count"]
        }"#
    }

    async fn call(&self, args: Value) -> Result<Value, anyhow::Error> {
        let sku = args
            .get("sku")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow::anyhow!("Missing sku"))?
            .to_string();

        let count = args
            .get("count")
            .and_then(Value::as_u64)
            .ok_or_else(|| anyhow::anyhow!("Missing count"))? as u32;

        let mut inv = self
            .inventory
            .lock()
            .map_err(|e| anyhow::anyhow!("Mutex lock poisoned: {}", e))?;
        let entry = inv.entry(sku.clone()).or_insert(0);
        *entry += count;
        let total = *entry;
        drop(inv);

        Ok(Value::String(format!(
            "Recorded {} units for {}. Total count is now {}.",
            count, sku, total
        )))
    }
}

/// A tool that keeps its state in the session rather than in the process.
///
/// `needs_context()` is the opt-in; `call_with_context` then receives the
/// session-scoped [`ToolContext`]. `update_state` is a read-modify-write under
/// one lock, so two of these running concurrently cannot lose a count.
struct CallCounterTool;

impl Tool for CallCounterTool {
    fn name(&self) -> &'static str {
        "count_requests"
    }

    fn description(&self) -> &'static str {
        "Reports how many times it has been called during this session."
    }

    fn parameters_json_schema(&self) -> &'static str {
        r#"{"type": "object", "properties": {}}"#
    }

    async fn call(&self, _args: Value) -> Result<Value, anyhow::Error> {
        // Never reached: a needs_context tool called without a context is an
        // error result, not a silent fallback to this path.
        Err(anyhow::anyhow!("count_requests requires a ToolContext"))
    }

    fn needs_context(&self) -> bool {
        true
    }

    async fn call_with_context(
        &self,
        _args: Value,
        context: &ToolContext,
    ) -> Result<Value, anyhow::Error> {
        context.update_state::<u32, _>("calls", |current| Some(current.unwrap_or(0) + 1));
        let calls: u32 = context.get_state("calls").unwrap_or(0);
        let conversation = context.conversation_id().unwrap_or_default();
        Ok(Value::String(format!(
            "called {calls} time(s) in conversation {conversation}"
        )))
    }
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    // Load environment variables from .env file if present
    dotenvy::dotenv().ok();

    // Check if the user specified a binary path or check the environment variable
    let harness_path = std::env::var("ANTIGRAVITY_HARNESS_PATH").ok();
    let api_key = std::env::var("GEMINI_API_KEY").ok();

    let mut builder = Agent::builder();
    if let Some(path) = harness_path {
        builder = builder.binary_path(path);
    }
    if let Some(key) = api_key {
        builder = builder.api_key(key);
    }

    // Initialize shared mutable state for stateful tool
    let inventory = Arc::new(Mutex::new(HashMap::new()));

    let agent = builder
        .default_model("gemini-3.5-flash")
        .system_instructions(antigravity_sdk_rust::types::SystemInstructions::Custom(
            antigravity_sdk_rust::types::CustomSystemInstructions {
                text: "You keep track of fruit inventory. To record fruits, you MUST first look up the \
                       fruit's SKU using lookup_fruit_sku, and then use that SKU with record_fruit."
                        .to_string(),
            },
        ))
        .tools(vec![
            Arc::new(LookupSkuTool),
            Arc::new(RecordFruitTool {
                inventory: inventory.clone(),
            }),
            Arc::new(CallCounterTool),
        ])
        .policies(vec![
            policy::deny_all(),
            policy::allow("lookup_fruit_sku"),
            policy::allow("record_fruit"),
            policy::allow("count_requests"),
        ])
        .build();

    println!("Starting agent...");
    let agent = agent.start().await?;

    println!("  === Custom Tools Demo ===");

    // Turn 1: Lookup fruit SKU
    let prompt1 = "What is the SKU for apples? We need to order more.";
    println!("\n  User: {}", prompt1);
    let response1 = agent.chat(prompt1).await?;
    println!("  Agent: {}", response1.text);

    // Stateful interaction: record fruits across multiple turns
    println!("\n  === Stateful Tool (Fruit Counter) Demo ===");

    let turns = vec![
        "I have 5 apples.",
        "And I just got 3 bananas.",
        "Oh, and another 2 apples.",
    ];

    for user_input in turns {
        println!("\n  User: {}", user_input);
        let response = agent.chat(user_input).await?;
        println!("  Agent: {}", response.text);
    }

    // Context-aware tool: its state lives in the session, not in this process.
    println!("\n  === Context-Aware Tool Demo ===");
    for _ in 0..2 {
        let prompt = "Call count_requests and tell me exactly what it returned.";
        println!("\n  User: {}", prompt);
        let response = agent.chat(prompt).await?;
        println!("  Agent: {}", response.text);
    }

    agent.stop().await?;
    Ok(())
}
