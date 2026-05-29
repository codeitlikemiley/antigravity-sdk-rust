   # Google Antigravity Rust SDK
   

https://github.com/user-attachments/assets/93e00393-6fe8-4546-85be-214b91b4de58




[![CI](https://github.com/codeitlikemiley/antigravity-sdk-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/codeitlikemiley/antigravity-sdk-rust/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/antigravity-sdk-rust.svg)](https://crates.io/crates/antigravity-sdk-rust)
[![Docs.rs](https://docs.rs/antigravity-sdk-rust/badge.svg)](https://docs.rs/antigravity-sdk-rust)
[![Rust](https://img.shields.io/badge/rust-2024-blue.svg)](https://www.rust-lang.org)

The Google Antigravity SDK is a Rust library for building AI agents powered by Antigravity and Gemini. It provides a secure, scalable, and stateful infrastructure layer that abstracts the agentic loop, letting you focus on what your agent *does* rather than how it runs.

## Installation

1. Add the SDK and tokio to your project:
   ```sh
   cargo add antigravity-sdk-rust
   cargo add tokio --features full
   ```

2. **Obtain the `localharness` binary:**
   The SDK relies on a compiled native Go runtime binary (`localharness`) to orchestrate agent operations. You have two options to install it:

   - **Option A: Without Python/pip (Recommended for Rust-only environments & web servers)**
     Run the helper installer script to download and extract the binary directly from PyPI (wheels are standard ZIP files):
     ```sh
     ./scripts/install_harness.sh
     export ANTIGRAVITY_HARNESS_PATH="$(pwd)/bin/localharness"
     ```

   - **Option B: Using Python/pip**
     If Python is already installed on your development machine, simply run:
     ```sh
     pip install google-antigravity
     ```
     The Rust SDK will automatically locate the binary inside the Python package installation directory fallback.

## Quickstart

Get started by setting your API key and running the `hello_world` example:

```sh
export GEMINI_API_KEY="your_api_key_here"
cargo run --example hello_world
```

## Concepts

### Simple Agent

The `Agent` struct manages the full lifecycle — binary discovery, tool wiring, hook registration, and policy defaults.

```rust,no_run
use antigravity_sdk_rust::agent::Agent;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // Configures and constructs the Agent using the builder pattern.
    // PhantomData guarantees compile-time verification of safety policies.
    let agent = Agent::builder()
        .allow_all()
        .build();

    // Spawns the harness session and transitions the Agent state to Started.
    let agent = agent.start().await?;

    let response = agent.chat("Say 'Hello World!'").await?;
    println!("Agent: {}", response.text);

    agent.stop().await?;
    Ok(())
}
```

### Advanced Usage with Conversation

For full control over the connection lifecycle, use `Conversation` with a `ConnectionStrategy` directly. `Conversation` is a stateful session that accumulates step history:

```rust,no_run
use antigravity_sdk_rust::agent::AgentConfig;
use antigravity_sdk_rust::connection::AnyConnection;
use antigravity_sdk_rust::conversation::Conversation;
use antigravity_sdk_rust::local::LocalConnectionStrategy;
use antigravity_sdk_rust::tools::ToolRunner;
use antigravity_sdk_rust::types::GeminiConfig;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let tool_runner = ToolRunner::new();
    let strategy = LocalConnectionStrategy::new(
        "localharness".to_string(), // path to harness
        GeminiConfig {
            enable_google_search: Some(true),
            enable_url_context: Some(true),
            ..Default::default()
        },
        Default::default(),
        None,
        None,
        vec![],
        vec![],
        Some(tool_runner),
        None,
        "my_conversation_id".to_string(),
        vec![], // MCP servers
    );

    let connection = strategy.connect().await?;
    let conversation = Conversation::new(AnyConnection::Local(Arc::new(connection)), None);

    let response = conversation.chat_to_completion("What files are here?").await?;
    println!("Agent: {}", response.text);

    Ok(())
}
```

## Features

### Custom Tools

Register Rust functions as custom tools:

```rust,no_run
use antigravity_sdk_rust::agent::{Agent, AgentConfig};
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
```

### Hooks and Policies

Control agent behavior with a declarative policy system:

```rust,no_run
use antigravity_sdk_rust::policy::{self, Policy};

let policies = vec![
    policy::deny_all(),                          // Block all tools by default
    policy::allow("VIEW_FILE"),                  // Allow reading/viewing files
];
```

### Error Handling

The SDK provides a typed error hierarchy via `AntigravityError`:

```rust,no_run
use antigravity_sdk_rust::error::AntigravityError;

// Three error categories:
// - AntigravityError::Connection(msg) — network/transport failures
// - AntigravityError::Execution(msg) — agent execution failures  
// - AntigravityError::Validation { message, errors } — input validation failures
```

### Multimodal Content

Send images, documents, audio, and video to the agent:

```rust,no_run
use antigravity_sdk_rust::types::{Content, Media, MimeType, ImageMime};

// Text-only (backwards compatible)
let response = agent.chat("What is 2+2?").await?;

// From file (auto-detects MIME type from extension)
let content = Content::from_file("photo.jpg", Some("Describe this")).unwrap();

// From raw bytes
let media = Media {
    data: vec![0xFF, 0xD8, 0xFF],
    mime_type: MimeType::Image(ImageMime::Jpeg),
    description: Some("A photo".to_string()),
};
let content = Content::media(media);
```

Supported formats: BMP, JPEG, PNG, WebP (images), PDF, JSON, CSS, CSV, HTML, JS, TXT, RTF, XML (documents), WAV, MP3, AAC, OGG, FLAC, Opus, MPEG, M4A, L16 (audio), 3GPP, AVI, MP4, MPEG, MOV, WebM, WMV, FLV (video).

### Lifecycle Hooks

The full hook lifecycle is available:

```rust,no_run
use antigravity_sdk_rust::hooks::Hook;
use antigravity_sdk_rust::types::{HookResult, ToolCall, ToolResult, ChatResponse};

struct MyHook;

impl Hook for MyHook {
    // Session lifecycle
    async fn on_session_start(&self) -> Result<(), anyhow::Error> { Ok(()) }
    async fn on_session_end(&self) -> Result<(), anyhow::Error> { Ok(()) }
    
    // Turn lifecycle
    async fn pre_turn(&self) -> Result<HookResult, anyhow::Error> {
        Ok(HookResult { allow: true, message: String::new() })
    }
    async fn post_turn(&self, _response: &ChatResponse) -> Result<(), anyhow::Error> { Ok(()) }
    
    // Tool lifecycle
    async fn pre_tool_call(&self, _call: &ToolCall) -> Result<HookResult, anyhow::Error> {
        Ok(HookResult { allow: true, message: String::new() })
    }
    async fn post_tool_call(&self, _result: &ToolResult) -> Result<(), anyhow::Error> { Ok(()) }
    
    // History compaction
    async fn on_compaction(&self, _summary: &str) -> Result<(), anyhow::Error> { Ok(()) }
}
```

### Hook Context (Hierarchical State)

Hooks can share state across lifecycle events via a parent-chaining key-value store:

```rust,no_run
use antigravity_sdk_rust::context::HookContext;
use std::sync::Arc;

// Session context (root)
let session = Arc::new(HookContext::new());
session.set("user_id", "u-123");

// Turn context (inherits session)
let turn = Arc::new(HookContext::child(session));
turn.set("turn_count", 1i32);

// get() walks up the chain:
assert_eq!(turn.get::<String>("user_id"), Some("u-123".to_string()));
```

### Context-Aware Tools

Tools can opt-in to receiving a `ToolContext` for session state and agent communication:

```rust,no_run
use antigravity_sdk_rust::tools::Tool;
use antigravity_sdk_rust::tool_context::ToolContext;
use serde_json::Value;

struct StatefulTool;

impl Tool for StatefulTool {
    fn name(&self) -> &str { "stateful_tool" }
    fn description(&self) -> &str { "A tool with session state" }
    fn parameters_json_schema(&self) -> &str { r#"{"type":"object"}"# }

    fn needs_context(&self) -> bool { true } // Opt-in

    async fn call(&self, args: Value) -> Result<Value, anyhow::Error> {
        Ok(Value::Null) // Fallback
    }

    async fn call_with_context(
        &self,
        args: Value,
        ctx: &ToolContext,
    ) -> Result<Value, anyhow::Error> {
        // Access session state
        let count: i32 = ctx.get_state("call_count").unwrap_or(0);
        ctx.set_state("call_count", count + 1);
        Ok(serde_json::json!({ "calls": count + 1 }))
    }
}
```

### Trigger Helpers

Convenience factories for common trigger patterns:

```rust,no_run
use antigravity_sdk_rust::trigger_helpers::every;
use std::time::Duration;

// Periodic trigger — fires every 30 seconds
let heartbeat = every(Duration::from_secs(30), "check_status");
```

### Interactive Loop

Built-in REPL for conversational agents:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::interactive::run_interactive_loop;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let agent = Agent::builder().allow_all().build().start().await?;
    run_interactive_loop(&agent).await?;
    agent.stop().await?;
    Ok(())
}
```

### MCP Integration

Connect to external MCP servers and expose their tools to the agent:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::types::McpServerConfig;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let agent = Agent::builder()
        .mcp_server(McpServerConfig::Stdio {
            name: "my_server".to_string(),
            command: "npx".to_string(),
            args: vec!["my-mcp-server".to_string()],
            enabled_tools: None,
            disabled_tools: None,
        })
        .allow_all()
        .build();

    let agent = agent.start().await?;
    let response = agent.chat("Use the MCP tools to help me.").await?;
    println!("{}", response.text);
    agent.stop().await?;
    Ok(())
}
```

Three transport types are supported:
- **`McpServerConfig::Stdio`** — launch a local subprocess (e.g., `npx`, `uvx`)
- **`McpServerConfig::Sse`** — connect via Server-Sent Events
- **`McpServerConfig::Http`** — connect via standard HTTP with configurable timeouts

Each variant supports `enabled_tools` / `disabled_tools` for fine-grained tool filtering.

### Sugared Thoughts & Tool Call Streams (Advanced)

For more complex use cases, stream internal model reasoning/thinking and intercept tool call dispatches in real-time using `StreamChunk`:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::types::StreamChunk;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let agent = Agent::builder().allow_all().build().start().await?;
    let conversation = agent.conversation();
    let mut stream = conversation.chat("Explain quantum computing").await?;

    while let Some(chunk_res) = stream.next().await {
        match chunk_res? {
            // 1. Stream reasoning/thinking deltas
            StreamChunk::Thought { text, .. } => {
                eprint!("💭 {}", text); // show thinking in grey/stderr
            }
            // 2. Stream response text tokens
            StreamChunk::Text { text, .. } => {
                print!("{}", text);
            }
            // 3. Stream strongly-typed ToolCall events
            StreamChunk::ToolCall(call) => {
                println!("\n🔧 Executing: {} (args: {})", call.name, call.args);
            }
        }
    }
    agent.stop().await?;
    Ok(())
}
```

### Google Search Grounding & Web Search Fallback

The SDK supports server-side Google Search grounding and provides a client-side search fallback:

- **Google Search Grounding**: Enable Gemini's native Google Search grounding tool by setting `enable_google_search: Some(true)` in `GeminiConfig`. This enables the model to natively perform search queries server-side to return up-to-date online information.
- **Web Search Fallback**: If the model decides to invoke a tool call named `google_search` or `web_search` and no custom search tool has been registered with `ToolRunner`, the SDK automatically runs a built-in search fallback. On native platforms, it spawns a `python3` subprocess to scrape and parse DuckDuckGo search results. On WASM platforms, it returns an empty result block indicating search is not available.

### Client-Side Tool Step Updates

When custom client-side tools are executed, the SDK connection dynamically generates and streams synthetic `Step` updates:
- Enters `ACTIVE` state before executing the tool, allowing the user interface to render an active tool execution card.
- Emits `DONE` (on success) or `ERROR` (on failure) once execution completes, displaying the results/error on the timeline.
- To prevent collision with step indices assigned by the harness, client-side tool steps use synthetic indices starting at `50,000`.


## Web Integration

The SDK provides two patterns for building web applications with AI agents:

### Leptos + Axum (Native)

For standard web servers (VPS, Docker, bare metal). The Agent runs in-process with full SDK features.

```sh
cd examples/leptos_axum
echo "GEMINI_API_KEY=your-key" > .env
cargo leptos serve
# Open http://localhost:3000
```

### Leptos + Spin/WASI (Edge)

For edge/serverless deployments on [Spin](https://github.com/spinframework/spin) (formerly Fermyon Spin, now under Akamai). Since Spin components cannot make outbound TCP/WebSocket connections directly, this target runs in **Sidecar Mode**, where the component communicates via HTTP with a native runner process (`agent_server`):

```sh
# Terminal 1: Start the agent sidecar
cd examples/agent_server
GEMINI_API_KEY=your-key cargo run

# Terminal 2: Start Spin
cd examples/leptos_ssr_axum
spin build --up
# Open http://localhost:3000
```

> See [`examples/README.md`](examples/README.md) for detailed architecture diagrams and configuration options.

## WebAssembly (WASM) & Edge Compilation

The SDK supports compiling directly to the `wasm32-wasip1` WebAssembly target. This enables running agents inside sandboxed Wasm runtimes (such as Spin, Wasmtime, or Wasmer) that support WASI network sockets.

### Wasm Connection Architecture

In standard native environments, the SDK spawns the `localharness` binary as a local subprocess. Since process spawning is not supported inside WebAssembly sandboxes, when targeting WASM, the SDK switches to **Direct Network Connection Mode** via `WasmConnectionStrategy` and `WasmConnection`:

1. **Host Server**: A host-side `localharness` WebSocket server is run on the machine/VM (configured via `ANTIGRAVITY_HARNESS_HOST` and `ANTIGRAVITY_HARNESS_PORT`).
2. **WebSocket Handshake**: The `WasmConnectionStrategy` opens a TCP socket to the harness server, initiates a WebSocket client handshake with the Gemini API key passed via the `x-goog-api-key` header, and upgrades it to a bi-directional stream.
3. **Session Handshake**: The client sends an `InitializeConversationEvent` containing the `HarnessConfig` (tools, workspaces, and system instructions) to start the stateful agent trajectory.
4. **Asynchronous Event Loop**: Two asynchronous tasks are spawned under the WASM event loop:
   - **Reader Loop**: Continuously reads WebSocket frames, parses incoming `OutputEvent` messages (including `StepUpdate` and `TrajectoryStateUpdate`), maps them to SDK `Step` types, dispatches lifecycle hooks, and routes tool calls.
   - **Sender Loop**: Buffers and sends outgoing `InputEvent` messages (user messages, tool responses, question replies, etc.) back to the harness.

### Running the WASM Mock Test Suite

The mock and unit test suite in `src/wasm.rs` validates WASM-specific functionality (connection lifecycle, event loops, tool call/result extraction, and idle state transitions). Since it uses a standard WebSocket framework, you can run the suite natively on your host machine without a full WebAssembly runner:

```sh
# Run all unit tests including the WASM mock connection tests
cargo test wasm::tests
```

This mock test suite programmatically binds a local WebSocket server to a dynamically allocated free port, completes the initialization handshake, feeds mock trajectory state updates/step updates, and asserts that `WasmConnection` correctly receives messages, invokes callback hooks, and cleanly terminates the stream upon transitioning to the idle state.

### Rust 2024 and Native Async Traits

The SDK has been fully refactored to use native async traits (stable since Rust 1.75 / Rust 2024), completely removing the dependency on the `#[async_trait]` macro.

- **Native Async Traits**: Custom implementations of `Connection`, `Hook`, `Tool`, and `Trigger` now use standard async function definitions returning `impl Future<Output = ...> + Send` where necessary to maintain compile-time thread safety bounds.
- **Dynamic Dispatch via Blanket Impls**: Because native async traits are not directly object-safe (`dyn Trait` compatible) without manual workarounds, the SDK defines companion traits `DynConnection`, `DynHook`, `DynTool`, and `DynTrigger` which are object-safe and automatically implemented via blanket implementations for any type implementing the base trait. This allows clean, zero-overhead dynamic dispatch (e.g. `Arc<dyn DynHook>`).

## Local Development

This project uses [just](https://github.com/casey/just) to manage development tasks.

- **Help / List Commands**: List all available `just` recipes.
  ```sh
  just
  # or
  just help
  ```
- **Run SDK Examples**: Start any of the SDK examples by directory or file name (handles both file and folder examples, converting dashes to underscores automatically).
  ```sh
  # Run the hello_world.rs console example
  just example hello_world

  # Run the agent_server HTTP sidecar example
  just example agent_server

  # Run the leptos_ssr_axum edge web example
  just example leptos_ssr_axum
  ```
- **Check Code Quality**: Run all style, lint, and test checks.
  ```sh
  just check
  ```
- **Install Local Harness**: Download and configure the required `localharness` binary.
  ```sh
  just install
  ```
- **Bump version & Release tag**: Bump the package version, update Cargo.lock, commit using `--no-verify` (skipping git hooks), and tag the release.
  ```sh
  # Auto-bump patch version (e.g., 0.1.0 -> 0.1.1)
  just version
  
  # Or force/override with a specific version
  just version 0.2.0
  ```
- **Publish to Crates.io**: Manually publish the package to crates.io (runs `just check` first).
  ```sh
  just publish
  ```

## Component Documentation

For more detailed documentation on specific components, see:

- **[Agent](docs/agent.md)** — High-level, batteries-included entry point.
- **[Connections](docs/connections.md)** — Transport and backend abstraction.
- **[Conversation](docs/conversation.md)** — Stateful session management.
- **[Hooks](docs/hooks.md)** — Agent lifecycle interception and policies.
- **[MCP](docs/mcp.md)** — Model Context Protocol integration.
- **[Tools](docs/tools.md)** — In-process tool execution.
- **[Triggers](docs/triggers.md)** — Background tasks and external events.

## Architecture

For more information, see [ARCHITECTURE.md](ARCHITECTURE.md).

## License

This project is licensed under the [MIT License](LICENSE).
