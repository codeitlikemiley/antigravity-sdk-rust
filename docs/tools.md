# Tools

In-process tool execution for the Antigravity Rust SDK.

## Overview

Tools are Rust functions exposed to the Gemini model for invocation. The SDK provides a trait-based system with JSON schema validation, automatic registration, and optional session-scoped context.

## Tool Trait

Define a tool by implementing the `Tool` trait:

```rust,no_run
use antigravity_sdk_rust::tools::Tool;
use serde_json::Value;

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
                "city": { "type": "string", "description": "City name" }
            },
            "required": ["city"]
        }"#
    }

    async fn call(&self, args: Value) -> Result<Value, anyhow::Error> {
        let city = args.get("city").and_then(|c| c.as_str()).unwrap_or("Unknown");
        Ok(serde_json::json!({
            "temperature": 22,
            "condition": "sunny",
            "city": city
        }))
    }
}
```

### Full Trait Signature

```rust,no_run
pub trait Tool: Send + Sync {
    /// Unique name used in tool calls from the model.
    fn name(&self) -> &str;

    /// Human-readable description of what the tool does.
    fn description(&self) -> &str;

    /// JSON Schema string describing the expected arguments.
    fn parameters_json_schema(&self) -> &str;

    /// Execute the tool with the given arguments.
    async fn call(&self, args: Value) -> Result<Value, anyhow::Error>;

    /// Whether this tool requires a `ToolContext`. Default: `false`.
    fn needs_context(&self) -> bool { false }

    /// Execute with session context. Default: delegates to `call()`.
    async fn call_with_context(
        &self,
        args: Value,
        ctx: &ToolContext,
    ) -> Result<Value, anyhow::Error> {
        self.call(args).await
    }
}
```

## DynTool (Object-Safe)

The `Tool` trait uses `async fn`, which isn't object-safe. The SDK provides `DynTool` — an object-safe wrapper trait that uses `BoxFuture` — with an automatic blanket implementation:

```rust,no_run
// Any T: Tool automatically implements DynTool
// You never need to implement DynTool manually
let tool: Arc<dyn DynTool> = Arc::new(WeatherTool);
```

## ToolRunner

`ToolRunner` manages tool registration and dispatch:

```rust,no_run
use antigravity_sdk_rust::tools::ToolRunner;
use std::sync::Arc;

let runner = ToolRunner::new();

// Register tools
runner.register(Arc::new(WeatherTool)).await;

// Execute a single tool by name
let result = runner.execute("get_weather", serde_json::json!({"city": "Tokyo"})).await?;

// Batch-execute multiple tool calls
let results = runner.process_tool_calls(&tool_calls).await;
```

Tools are stored behind `Arc<RwLock<Vec<Arc<dyn DynTool>>>>` for concurrent access.

## Context-Aware Tools

Tools can opt-in to receiving a `ToolContext` for session state and agent communication:

```rust,no_run
use antigravity_sdk_rust::tools::Tool;
use antigravity_sdk_rust::tool_context::ToolContext;
use serde_json::Value;

struct CounterTool;

impl Tool for CounterTool {
    fn name(&self) -> &str { "counter" }
    fn description(&self) -> &str { "Increment and return a counter" }
    fn parameters_json_schema(&self) -> &str { r#"{"type":"object"}"# }

    fn needs_context(&self) -> bool { true } // Opt-in

    async fn call(&self, _args: Value) -> Result<Value, anyhow::Error> {
        Ok(Value::Null) // Fallback when no context
    }

    async fn call_with_context(
        &self,
        _args: Value,
        ctx: &ToolContext,
    ) -> Result<Value, anyhow::Error> {
        let count: i32 = ctx.get_state("count").unwrap_or(0);
        ctx.set_state("count", count + 1);
        Ok(serde_json::json!({ "count": count + 1 }))
    }
}
```

### ToolContext API

```rust,no_run
pub struct ToolContext {
    // Methods:
    fn conversation_id(&self) -> &str;
    fn is_idle(&self) -> bool;
    async fn send(&self, message: &str) -> Result<()>;
    fn get_state<T: DeserializeOwned>(&self, key: &str) -> Option<T>;
    fn set_state<T: Serialize>(&self, key: &str, value: T);
}
```

> **Note**: Tool state is independent of Hook state. They use separate stores.

## Built-in Tools

The SDK provides these built-in tools (managed by the harness):

| Tool | Description |
|------|-------------|
| `CreateFile` | Create a new file |
| `EditFile` | Edit an existing file |
| `FindFile` | Search for files by name |
| `ListDir` | List directory contents |
| `RunCommand` | Execute shell commands |
| `SearchDir` | Search file contents |
| `ViewFile` | Read file contents |
| `StartSubagent` | Launch sub-agents |
| `GenerateImage` | Generate images |
| `Finish` | Signal task completion |
| `GrepSearch` | Grep-based search |

### Read-Only Tools

`BuiltinTools::read_only()` returns: `FindFile`, `ListDir`, `SearchDir`, `ViewFile`, `GrepSearch`.

## Agent Builder Integration

Register tools via the builder:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use std::sync::Arc;

let agent = Agent::builder()
    .tool(Arc::new(WeatherTool))       // single tool
    .tools(vec![                        // multiple tools
        Arc::new(WeatherTool),
        Arc::new(CounterTool),
    ])
    .allow_all()
    .build();
```

## Shared State Pattern

For tools that need shared mutable state, use `Arc<Mutex<T>>`:

```rust,no_run
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

struct InventoryTool {
    db: Arc<Mutex<HashMap<String, i32>>>,
}

impl Tool for InventoryTool {
    fn name(&self) -> &str { "check_inventory" }
    fn description(&self) -> &str { "Check item inventory" }
    fn parameters_json_schema(&self) -> &str { r#"{"type":"object","properties":{"item":{"type":"string"}}}"# }

    async fn call(&self, args: Value) -> Result<Value, anyhow::Error> {
        let item = args["item"].as_str().unwrap_or("");
        let db = self.db.lock().unwrap();
        let count = db.get(item).copied().unwrap_or(0);
        Ok(serde_json::json!({ "item": item, "count": count }))
    }
}
```

## Google Search Fallback

If the model invokes `google_search` or `web_search` and no custom tool handles it, the SDK runs a built-in DuckDuckGo scraper:
- **Native**: Spawns `python3` subprocess to scrape results
- **WASM**: Returns empty result (not available)

## Python SDK Comparison

| Python | Rust |
|--------|------|
| `@tool` decorator or `ToolWithSchema` | `impl Tool for T` trait |
| `ToolRunner.register()` | `ToolRunner::register()` |
| `ToolRunner.execute()` | `ToolRunner::execute()` |
| `ToolContext` with `get_state`/`set_state` | `ToolContext` with `get_state`/`set_state` |
| Sync/async auto-detection | All tools are async |
