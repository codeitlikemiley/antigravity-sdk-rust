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
        // Never reached: a `needs_context` tool called without a context is an
        // error result, not a silent fallback.
        unreachable!()
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
    fn conversation_id(&self) -> Option<String>;
    fn is_idle(&self) -> Option<bool>;
    async fn send(&self, message: &str) -> Result<()>;
    fn get_state<T: DeserializeOwned>(&self, key: &str) -> Option<T>;
    fn set_state<T: Serialize>(&self, key: &str, value: T);
    fn update_state<T, F>(&self, key: &str, transform: F); // atomic read-modify-write
}
```

The context holds a **weak** handle to the session — the connection owns the
tool runner, so a strong one would be a cycle that never frees. The two
`Option`-returning methods are `None`, and `send` errors, once the agent has
stopped.

`Agent::start()` attaches the context. A `needs_context` tool invoked with none
attached returns an error result rather than falling back to `call()`, which
would run the tool in a subtly different mode.

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
| `AskQuestion` | Put a multiple-choice question to the user |
| `GrepSearch` | Grep-based search |

### Read-Only Tools

`BuiltinTools::read_only()` returns: `ListDir`, `SearchDir`, `FindFile`, `ViewFile`, `Finish` —
matching upstream's `BuiltinTools.read_only()`. `Finish` is included because an agent that
cannot finish cannot terminate a turn or emit structured output.

Related sets: `BuiltinTools::all_tools()`, `BuiltinTools::file_tools()` (upstream's exact
three file tools) and `BuiltinTools::path_scoped_tools()` (what `workspace_only` scopes —
a deliberate superset, see [policy scoping](#workspace-scoping)).

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

## Python SDK Comparison

| Python | Rust |
|--------|------|
| `@tool` decorator or `ToolWithSchema` | `impl Tool for T` trait |
| `ToolRunner.register()` | `ToolRunner::register()` |
| `ToolRunner.execute()` | `ToolRunner::execute()` |
| `ToolContext` with `get_state`/`set_state` | `ToolContext` with `get_state`/`set_state` |
| Sync/async auto-detection | All tools are async |

## Names must be unique

Registering two tools with the same name is an error, surfaced from
`Agent::start()`. Silently replacing the first — the old behaviour — meant a
collision between two modules' tools resolved to whichever registered last, and
the model called something the caller never meant to expose.

Registration order is preserved, and it is the order the tool list reaches the
model in.

## A batch runs concurrently

When the model asks for several tools at once, they execute concurrently and
the batch takes as long as its slowest member rather than the sum. Results come
back in call order regardless of which finished first. Tools that share mutable
state need their own synchronisation.
