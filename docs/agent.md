# Agent

High-level, batteries-included entry point for building AI agents.

## Overview

The `Agent` struct manages the full lifecycle of an agentic AI session — binary discovery, tool wiring, hook registration, policy enforcement, and harness communication. It is the primary construct you'll interact with when building applications on the Antigravity SDK.

```text
┌─────────────────────────────────────────────────────────────────┐
│                        Agent<Unstarted>                         │
│  ┌──────────────┐  ┌──────────┐  ┌─────────┐  ┌────────────┐  │
│  │ AgentConfig   │  │ToolRunner│  │HookRunner│  │PolicyEnforcer│ │
│  │  (model, key, │  │(custom   │  │(lifecycle│  │ (safety     │ │
│  │   policies…)  │  │  tools)  │  │  hooks)  │  │  policies)  │ │
│  └──────────────┘  └──────────┘  └─────────┘  └────────────┘  │
│                          │                                      │
│                    .start().await?                               │
│                          ▼                                      │
│                   Agent<Started>                                │
│            ┌──────────────────────┐                             │
│            │  Conversation (Arc)  │                             │
│            │  ┌────────────────┐  │                             │
│            │  │ Connection     │  │                             │
│            │  │ (WebSocket/IPC)│  │                             │
│            │  └────────────────┘  │                             │
│            └──────────────────────┘                             │
└─────────────────────────────────────────────────────────────────┘
```

**Key responsibilities:**

- **Binary discovery** — locates `localharness` automatically via env var, local install, `PATH`, or Python site-packages
- **Tool registration** — wires custom Rust `Tool` implementations into the model's callable toolset
- **Hook dispatch** — sequences lifecycle observer hooks (pre/post tool call, session start/end, etc.)
- **Policy enforcement** — compiles safety policies into a prioritized hook that gates tool execution
- **Connection management** — spawns the subprocess, upgrades to WebSocket, manages the conversation stream

---

## Builder Pattern (Typestate)

The agent uses a **compile-time typestate pattern** to guarantee that safety policies are always configured before an agent can be built. This eliminates an entire class of "forgot to set policies" runtime errors.

```text
Agent::builder()
    │
    ▼
AgentBuilder<NoPolicies>    ← .binary_path(), .api_key(), .tools(), .hooks(), etc.
    │
    │── .allow_all()  ──────────►  AgentBuilder<HasPolicies>  ──► .build()
    │── .read_only()  ──────────►  AgentBuilder<HasPolicies>  ──► .build()
    │── .policies(…)  ──────────►  AgentBuilder<HasPolicies>  ──► .build()
    │── .policy(…)    ──────────►  AgentBuilder<HasPolicies>  ──► .build()
    │
    └── .build_unchecked()  ────►  Agent<Unstarted>  (escape hatch, skips check)
```

### How it works

1. `Agent::builder()` returns `AgentBuilder<NoPolicies>`
2. Configuration methods (`.api_key()`, `.tools()`, `.hooks()`, etc.) return `Self`, preserving the current type-state
3. Policy-setting methods (`.allow_all()`, `.read_only()`, `.policies()`, `.policy()`) **consume** the builder and return `AgentBuilder<HasPolicies>`
4. `.build()` is only implemented on `AgentBuilder<HasPolicies>` — calling it on `NoPolicies` is a **compile error**
5. `.build_unchecked()` is available on any `AgentBuilder<P>` as an escape hatch

```rust,no_run
use antigravity_sdk_rust::agent::Agent;

// ✅ Compiles — policies are set via .allow_all()
let agent = Agent::builder()
    .api_key("my-key")
    .allow_all()
    .build();

// ❌ Compile error — .build() is not available on AgentBuilder<NoPolicies>
// let agent = Agent::builder()
//     .api_key("my-key")
//     .build();  // error[E0599]: no method named `build` found

// ⚠️ Escape hatch — bypasses compile-time policy check
let agent = Agent::builder()
    .api_key("my-key")
    .build_unchecked();
```

### Type-state markers

```rust,no_run
// Marker types — you won't construct these directly
pub struct NoPolicies;   // Initial state, .build() is NOT available
pub struct HasPolicies;  // After setting policies, .build() IS available
```

> **Python SDK comparison:** The Python SDK performs this check at runtime during `agent.start()`, raising a `ValueError` if write tools are enabled without policies. The Rust SDK catches it at compile time, shifting the error left.

---

## AgentConfig Fields

`AgentConfig` is the resolved configuration struct that powers the `Agent`. While you'll typically use the builder, understanding the underlying fields is useful for advanced use cases.

```rust,no_run
use antigravity_sdk_rust::agent::AgentConfig;

// AgentConfig is Default — all fields start as None/empty
let config = AgentConfig::default();
```

| Field | Type | Description |
|---|---|---|
| `binary_path` | `Option<String>` | Path to the `localharness` binary. If `None`, auto-discovered via the [binary discovery](#binary-discovery) algorithm. |
| `gemini_config` | `GeminiConfig` | Model selection, API key, Vertex AI project/location, search grounding, URL context. |
| `capabilities` | `CapabilitiesConfig` | Tool enable/disable lists, compaction threshold, image model override, finish tool schema. |
| `system_instructions` | `Option<SystemInstructions>` | Either `Custom` (full text override) or `Appended` (sections added to default identity). |
| `save_dir` | `Option<String>` | Directory for persisting session state/logs. |
| `workspaces` | `Option<Vec<String>>` | Working directories the agent may access. Defaults to `cwd` if `None`. Used by `workspace_only` policies. |
| `skills_paths` | `Vec<String>` | Paths to folders containing custom skill modules. |
| `policies` | `Option<Vec<Policy>>` | Safety policies controlling tool execution (approve, deny, ask_user). |
| `hooks` | `Vec<Arc<dyn DynHook>>` | Lifecycle hooks — observe/intercept session start, tool calls, turns, errors, etc. |
| `triggers` | `Vec<Arc<dyn DynTrigger>>` | Background async workers spawned when the agent starts. |
| `tools` | `Vec<Arc<dyn DynTool>>` | Custom Rust tools registered for model invocation. |
| `mcp_servers` | `Vec<McpServerConfig>` | Model Context Protocol server configurations (stdio, SSE, or HTTP transports). |
| `conversation_id` | `Option<String>` | Assign or resume a specific conversation ID. |
| `app_data_dir` | `Option<String>` | Application data directory for cache/configs. Defaults to `$HOME/.gemini/antigravity`. |
| `response_schema` | `Option<String>` | JSON Schema string constraining the agent's final structured output. |

---

## Builder Methods

All builder methods follow the fluent chaining pattern. Methods that don't affect policy state preserve the current typestate (`Self`). Methods that set policies transition from `NoPolicies` → `HasPolicies`.

### Configuration Methods (available on any `AgentBuilder<P>`)

These methods return `Self` and can be called in any order, regardless of policy state.

| Method | Signature | Description |
|---|---|---|
| `.binary_path()` | `fn binary_path(self, path: impl Into<String>) -> Self` | Sets the path to the `localharness` binary. |
| `.gemini_config()` | `fn gemini_config(self, config: GeminiConfig) -> Self` | Sets the full Gemini configuration (model, API key, Vertex AI, search). |
| `.api_key()` | `fn api_key(self, key: impl Into<String>) -> Self` | Shorthand to set the API key on `gemini_config`. |
| `.default_model()` | `fn default_model(self, model: impl Into<String>) -> Self` | Shorthand to set the default model name (e.g. `"gemini-3.5-flash"`). |
| `.capabilities()` | `fn capabilities(self, caps: CapabilitiesConfig) -> Self` | Sets tool enable/disable lists and thresholds. |
| `.system_instructions()` | `fn system_instructions(self, si: SystemInstructions) -> Self` | Sets custom or appended system instructions. |
| `.save_dir()` | `fn save_dir(self, dir: impl Into<String>) -> Self` | Sets the session state log directory. |
| `.workspaces()` | `fn workspaces(self, ws: Vec<String>) -> Self` | Sets the allowed workspace directories. |
| `.skills_paths()` | `fn skills_paths(self, paths: Vec<String>) -> Self` | Sets custom skill module folder paths. |
| `.hooks()` | `fn hooks(self, hooks: Vec<Arc<dyn DynHook>>) -> Self` | Sets the full list of lifecycle hooks (replaces any existing). |
| `.hook()` | `fn hook(self, hook: Arc<dyn DynHook>) -> Self` | Appends a single lifecycle hook. |
| `.triggers()` | `fn triggers(self, triggers: Vec<Arc<dyn DynTrigger>>) -> Self` | Sets the full list of background triggers (replaces any existing). |
| `.trigger()` | `fn trigger(self, trigger: Arc<dyn DynTrigger>) -> Self` | Appends a single background trigger. |
| `.tools()` | `fn tools(self, tools: Vec<Arc<dyn DynTool>>) -> Self` | Sets the full list of custom tools (replaces any existing). |
| `.tool()` | `fn tool(self, tool: Arc<dyn DynTool>) -> Self` | Appends a single custom tool. |
| `.conversation_id()` | `fn conversation_id(self, id: impl Into<String>) -> Self` | Assigns or resumes a conversation ID. |
| `.app_data_dir()` | `fn app_data_dir(self, dir: impl Into<String>) -> Self` | Sets the application data directory. |
| `.response_schema()` | `fn response_schema(self, schema: impl Into<String>) -> Self` | Sets a JSON Schema for structured output. |
| `.mcp_server()` | `fn mcp_server(self, server: McpServerConfig) -> Self` | Appends a single MCP server configuration. |
| `.mcp_servers()` | `fn mcp_servers(self, servers: Vec<McpServerConfig>) -> Self` | Sets the full list of MCP server configurations (replaces any existing). |

### Policy Methods (transition `NoPolicies` → `HasPolicies`)

These methods **consume** the builder and return `AgentBuilder<HasPolicies>`, enabling `.build()`.

| Method | Signature | Description |
|---|---|---|
| `.policy()` | `fn policy(self, policy: Policy) -> AgentBuilder<HasPolicies>` | Appends a single policy and transitions to `HasPolicies`. |
| `.policies()` | `fn policies(self, policies: Vec<Policy>) -> AgentBuilder<HasPolicies>` | Sets the full policy list and transitions to `HasPolicies`. |
| `.allow_all()` | `fn allow_all(self) -> AgentBuilder<HasPolicies>` | Convenience: sets `policy::allow_all()` — approves all tool calls unconditionally. |
| `.read_only()` | `fn read_only(self) -> AgentBuilder<HasPolicies>` | Convenience: denies all tools except read-only ones (`FindFile`, `ListDir`, `ViewFile`, `SearchDir`). |

### Build Methods

| Method | Signature | Available On | Description |
|---|---|---|---|
| `.build()` | `fn build(self) -> Agent<Unstarted>` | `AgentBuilder<HasPolicies>` only | Constructs the agent. Compile error if policies are not set. |
| `.build_unchecked()` | `fn build_unchecked(self) -> Agent<Unstarted>` | Any `AgentBuilder<P>` | Escape hatch — builds without compile-time policy check. Runtime errors may still occur at `.start()` if write tools are enabled without policies. |

---

## Lifecycle: Unstarted → Started

The `Agent` is generic over its lifecycle state, using the `AgentLifecycle` trait:

```text
Agent<Unstarted>  ──  .start().await?  ──►  Agent<Started>  ──  .stop().await?
```

### `Agent<Unstarted>`

An agent that has been configured but not yet connected. Available methods:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::hooks::DynHook;
use antigravity_sdk_rust::tools::DynTool;
use antigravity_sdk_rust::triggers::DynTrigger;
use std::sync::Arc;

let mut agent = Agent::builder().allow_all().build();

// Additional registrations before starting
// agent.register_hook(hook);
// agent.register_tool(tool);
// agent.register_trigger(trigger)?;
```

| Method | Signature | Description |
|---|---|---|
| `Agent::new()` | `fn new(config: AgentConfig) -> Self` | Direct construction from config (prefer the builder). |
| `Agent::builder()` | `fn builder() -> AgentBuilder<NoPolicies>` | Returns a new builder. |
| `.register_hook()` | `fn register_hook(&mut self, hook: Arc<dyn DynHook>)` | Adds a hook after construction but before starting. |
| `.register_tool()` | `fn register_tool(&mut self, tool: Arc<dyn DynTool>)` | Adds a tool after construction but before starting. |
| `.register_trigger()` | `fn register_trigger(&mut self, trigger: Arc<dyn DynTrigger>) -> Result<()>` | Adds a trigger after construction but before starting. |
| `.start()` | `fn start(self) -> BoxFuture<'static, Result<Agent<Started>>>` | Resolves the binary, connects, registers tools/hooks/policies, starts triggers. Consumes `self`. |

### `Agent<Started>`

A running agent with an active connection. Available methods:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;

# async fn example() -> Result<(), anyhow::Error> {
let agent = Agent::builder()
    .allow_all()
    .build()
    .start()
    .await?;

// Send a prompt and wait for the full response
let response = agent.chat("What is 2+2?").await?;
println!("Text: {}", response.text);
println!("Thinking: {}", response.thinking);
println!("Steps: {}", response.steps.len());

// Access the conversation for streaming or advanced use
let conversation = agent.conversation();

// Get the conversation ID
let id = agent.conversation_id();

// Shut down gracefully
agent.stop().await?;
# Ok(())
# }
```

| Method | Signature | Description |
|---|---|---|
| `.chat()` | `async fn chat(&self, prompt: &str) -> Result<ChatResponse>` | Sends a prompt and awaits the complete response. Returns text, thinking, steps, and usage metadata. |
| `.conversation()` | `fn conversation(&self) -> Arc<Conversation>` | Returns the active `Conversation` for streaming or direct access. |
| `.conversation_id()` | `fn conversation_id(&self) -> String` | Returns the conversation session ID. |
| `.stop()` | `async fn stop(&self) -> Result<()>` | Gracefully disconnects the harness and stops the session. |

### `ChatResponse`

The response from `.chat()` contains:

```rust,no_run
use antigravity_sdk_rust::types::ChatResponse;

// ChatResponse {
//     text: String,              // Combined model text output
//     thinking: String,          // Combined reasoning/thinking text
//     steps: Vec<Step>,          // All intermediate execution steps
//     usage_metadata: UsageMetadata, // Token consumption stats
// }
```

---

## `.start()` Internals

When `.start()` is called, the following happens in order:

1. **Resolve binary path** — Uses the [binary discovery](#binary-discovery) algorithm to find `localharness`
2. **Register hooks** — All configured hooks are added to the `HookRunner`
3. **Process capabilities** — Resolves enabled/disabled tool lists (mutually exclusive)
4. **Compile policies** — Builds `PolicyEnforcer` from configured policies; prepends `workspace_only` policies unless `allow_all()` was used
5. **Safety check** — Fails if write tools are enabled without any policies
6. **Register tools** — All custom tools are added to the `ToolRunner`
7. **Connect** — Spawns `localharness` subprocess, establishes WebSocket connection
8. **Start triggers** — Spawns background trigger tasks

### Errors

`.start()` returns `Err` if:

- The `localharness` binary cannot be found
- `enabled_tools` and `disabled_tools` are both set (mutually exclusive)
- Write tools are enabled but no policies are configured
- The subprocess or WebSocket connection fails

---

## Binary Discovery

When `binary_path` is not explicitly set, the SDK searches for `localharness` in the following order:

| Priority | Location | Description |
|---|---|---|
| 1 | `$ANTIGRAVITY_HARNESS_PATH` | Environment variable override |
| 2 | `./bin/localharness` | Local install relative to `cwd` (where `just install` places it) |
| 3 | `$PATH` lookup | Standard PATH search (e.g. via `pip install google-antigravity`) |
| 4 | Python site-packages | Fallback: queries `python3 -c "import site; ..."` and checks `google/antigravity/bin/localharness` |

If none are found, `.start()` returns an error with a message to specify `binary_path` explicitly.

```rust,no_run
use antigravity_sdk_rust::agent::Agent;

# async fn example() -> Result<(), anyhow::Error> {
// Explicit path — skips discovery
let agent = Agent::builder()
    .binary_path("/usr/local/bin/localharness")
    .allow_all()
    .build()
    .start()
    .await?;

// Auto-discovery — checks env var, ./bin, PATH, site-packages
let agent = Agent::builder()
    .allow_all()
    .build()
    .start()
    .await?;
# Ok(())
# }
```

---

## Examples

### 1. Minimal Hello World

The simplest possible agent — auto-discovers the binary, uses default model, permits all tool calls:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let agent = Agent::builder()
        .allow_all()
        .build()
        .start()
        .await?;

    let response = agent.chat("Say 'Hello World!'").await?;
    println!("Agent: {}", response.text);

    agent.stop().await?;
    Ok(())
}
```

### 2. Custom Model + API Key

Configure a specific model and API key, with an explicit binary path:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();

    let agent = Agent::builder()
        .binary_path(std::env::var("ANTIGRAVITY_HARNESS_PATH").unwrap())
        .api_key(std::env::var("GEMINI_API_KEY").unwrap())
        .default_model("gemini-3.5-flash")
        .allow_all()
        .build()
        .start()
        .await?;

    let response = agent.chat("Explain Rust's ownership model in 3 sentences.").await?;
    println!("{}", response.text);
    println!("Tokens used: {}", response.usage_metadata.total_token_count);

    agent.stop().await?;
    Ok(())
}
```

### 3. Custom Tools + Hooks + Policies

Register custom tools with fine-grained safety policies — deny all built-in tools, allow only your custom ones:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::hooks::Hook;
use antigravity_sdk_rust::policy;
use antigravity_sdk_rust::tools::Tool;
use antigravity_sdk_rust::types::{
    ChatResponse, CustomSystemInstructions, HookResult, SystemInstructions, ToolCall,
};
use serde_json::Value;
use std::sync::Arc;

// ── Custom Tool ─────────────────────────────────────────────────────

struct WeatherTool;

impl Tool for WeatherTool {
    fn name(&self) -> &'static str {
        "get_weather"
    }

    fn description(&self) -> &'static str {
        "Returns the current weather for a given city."
    }

    fn parameters_json_schema(&self) -> &'static str {
        r#"{
            "type": "object",
            "properties": {
                "city": { "type": "string", "description": "City name" }
            },
            "required": ["city"]
        }"#
    }

    async fn call(&self, args: Value) -> Result<Value, anyhow::Error> {
        let city = args.get("city").and_then(Value::as_str).unwrap_or("unknown");
        Ok(Value::String(format!("Weather in {city}: 22°C, partly cloudy")))
    }
}

// ── Custom Hook ─────────────────────────────────────────────────────

struct AuditHook;

impl Hook for AuditHook {
    async fn pre_tool_call(&self, tool_call: &ToolCall) -> Result<HookResult, anyhow::Error> {
        println!("[AUDIT] Tool called: {} with args: {}", tool_call.name, tool_call.args);
        Ok(HookResult { allow: true, message: String::new() })
    }

    async fn post_turn(&self, response: &ChatResponse) -> Result<(), anyhow::Error> {
        println!("[AUDIT] Turn complete. Tokens: {}", response.usage_metadata.total_token_count);
        Ok(())
    }
}

// ── Main ────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let agent = Agent::builder()
        .api_key("your-api-key")
        .default_model("gemini-3.5-flash")
        .system_instructions(SystemInstructions::Custom(CustomSystemInstructions {
            text: "You are a helpful weather assistant. Use get_weather to answer weather questions.".to_string(),
        }))
        .tool(Arc::new(WeatherTool))
        .hook(Arc::new(AuditHook))
        .policies(vec![
            policy::deny_all(),              // Deny everything by default
            policy::allow("get_weather"),     // Allow only our custom tool
        ])
        .build()
        .start()
        .await?;

    let response = agent.chat("What's the weather like in Tokyo?").await?;
    println!("Agent: {}", response.text);

    agent.stop().await?;
    Ok(())
}
```

### 4. Structured Output

Constrain the agent to return JSON matching a schema:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let schema = r#"{
        "type": "object",
        "properties": {
            "name":    { "type": "string" },
            "capital": { "type": "string" },
            "population": { "type": "integer" }
        },
        "required": ["name", "capital", "population"]
    }"#;

    let agent = Agent::builder()
        .api_key("your-api-key")
        .default_model("gemini-3.5-flash")
        .response_schema(schema)
        .allow_all()
        .build()
        .start()
        .await?;

    let response = agent.chat("Tell me about Japan.").await?;
    println!("Raw text: {}", response.text);

    // The last Finish step contains the parsed structured output
    for step in &response.steps {
        if let Some(ref output) = step.structured_output {
            println!("Structured: {}", serde_json::to_string_pretty(output)?);
        }
    }

    agent.stop().await?;
    Ok(())
}
```

### 5. Streaming via `conversation()`

For token-by-token streaming, access the `Conversation` directly:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::types::StreamChunk;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let agent = Agent::builder()
        .allow_all()
        .build()
        .start()
        .await?;

    let conversation = agent.conversation();

    // Send a prompt and get a streaming chunk iterator
    let mut stream = conversation.chat("Write a haiku about Rust.").await?;

    // Process chunks as they arrive
    while let Some(chunk_result) = stream.next().await {
        match chunk_result? {
            StreamChunk::Thought { text, .. } => {
                eprint!("[thinking] {}", text);
            }
            StreamChunk::Text { text, .. } => {
                print!("{}", text);
            }
            StreamChunk::ToolCall(call) => {
                println!("\n[tool] {} called with: {}", call.name, call.args);
            }
        }
    }
    println!();

    // Access conversation metadata
    println!("Total turns: {}", conversation.turn_count().await);
    println!("Total tokens: {}", conversation.total_usage().await.total_token_count);
    println!("History steps: {}", conversation.history().await.len());

    agent.stop().await?;
    Ok(())
}
```

### 6. Read-Only Agent

Restrict the agent to only read files — no writes, no commands:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let agent = Agent::builder()
        .workspaces(vec!["/home/user/project".to_string()])
        .read_only()    // Only FindFile, ListDir, ViewFile, SearchDir
        .build()
        .start()
        .await?;

    let response = agent.chat("Summarize the README.md").await?;
    println!("{}", response.text);

    agent.stop().await?;
    Ok(())
}
```

### 7. Multi-Turn Conversation

The agent maintains state across turns within a single session:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let agent = Agent::builder()
        .allow_all()
        .build()
        .start()
        .await?;

    let r1 = agent.chat("My name is Alice.").await?;
    println!("Agent: {}", r1.text);

    let r2 = agent.chat("What's my name?").await?;
    println!("Agent: {}", r2.text);  // Should reference "Alice"

    let r3 = agent.chat("Summarize our conversation.").await?;
    println!("Agent: {}", r3.text);

    agent.stop().await?;
    Ok(())
}
```

### 8. Advanced: Vertex AI + Full GeminiConfig

Use Vertex AI backend with custom model configuration:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::types::{
    GeminiConfig, GenerationConfig, ModelConfig, ModelEntry, ThinkingLevel,
};

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let gemini_config = GeminiConfig {
        vertex: true,
        project: Some("my-gcp-project".to_string()),
        location: Some("us-central1".to_string()),
        models: ModelConfig {
            default: ModelEntry {
                name: "gemini-3.5-flash".to_string(),
                api_key: None,
                generation: GenerationConfig {
                    thinking_level: Some(ThinkingLevel::High),
                },
            },
            ..Default::default()
        },
        enable_google_search: Some(true),
        enable_url_context: Some(true),
        ..Default::default()
    };

    let agent = Agent::builder()
        .gemini_config(gemini_config)
        .allow_all()
        .build()
        .start()
        .await?;

    let response = agent.chat("Explain quantum computing.").await?;
    println!("{}", response.text);
    if !response.thinking.is_empty() {
        println!("\n--- Thinking ---\n{}", response.thinking);
    }

    agent.stop().await?;
    Ok(())
}
```

---

## Comparison with Python SDK

| Feature | Python SDK | Rust SDK |
|---|---|---|
| Policy enforcement | Runtime `ValueError` during `agent.connect()` | Compile-time typestate — `.build()` unavailable without policies |
| Tool trait | `Tool` base class with `call(args)` | `Tool` trait with `name()`, `description()`, `parameters_json_schema()`, `call(args)` |
| Hook trait | `Hook` base class, overridable methods | `Hook` trait with default async no-ops |
| Builder pattern | `Agent(model=..., tools=[...], ...)` constructor kwargs | Typestate builder: `Agent::builder().model().tools().allow_all().build()` |
| Streaming | `async for chunk in conversation.chat(prompt)` | `conversation.chat(prompt).await?` → `StreamExt::next()` on `BoxStream` |
| Lifecycle states | Implicit (`agent.connect()` / `agent.close()`) | Typestate: `Agent<Unstarted>` / `Agent<Started>` — method availability enforced at compile time |
| Async runtime | `asyncio` | `tokio` |
| Object safety | Not applicable (duck typing) | `DynTool` / `DynHook` / `DynTrigger` object-safe wrappers via blanket impls |

---

## Related Types

For deeper dives into the types referenced here, see:

- **[`Conversation`](conversation.md)** — Stateful session wrapper with streaming and history
- **[`Tool`](tools.md)** — Custom tool trait and registration
- **[`Hook`](hooks.md)** — Lifecycle event hooks
- **[`Policy`](policy.md)** — Safety policy system
- **[`Trigger`](triggers.md)** — Background async workers
- **[`GeminiConfig`](types.md)** — Model and API configuration
- **[`McpServerConfig`](types.md)** — MCP server configuration (stdio, SSE, HTTP)
