# Connections

Transport and backend abstraction layer for the Antigravity Rust SDK.

## Overview

The connection layer abstracts communication between the SDK and the `localharness` backend.
Every agent session is backed by a **connection** — an active transport channel that sends
prompts, receives step updates, dispatches tool handshakes, and manages idle/disconnect lifecycle.

Most users never interact with connections directly; the [`Agent`](./agent.md) builder handles
connection setup automatically. This module is relevant when you need to:

- Understand the transport architecture
- Implement a custom connection strategy
- Use `Conversation` directly without the `Agent` wrapper

## Connection Trait

The `Connection` trait defines the transport-agnostic interface that all connection
implementations must satisfy:

```rust,no_run
use futures_util::stream::BoxStream;
use antigravity_sdk_rust::types::{QuestionHookResult, Step, ToolResult};

pub trait Connection: Send + Sync {
    /// Returns the unique conversation ID for this session.
    fn conversation_id(&self) -> &str;

    /// Returns whether the connection is currently idle (no active turns).
    fn is_idle(&self) -> bool;

    /// Subscribes to the stream of step updates from the harness.
    fn receive_steps(&self) -> BoxStream<'static, Result<Step, anyhow::Error>>;

    /// Sends a user text prompt to the harness.
    fn send(
        &self,
        content: &str,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;

    /// Sends a trigger notification message (from automated triggers).
    fn send_trigger_notification(
        &self,
        content: &str,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;

    /// Sends a halt/cancellation request.
    fn send_halt_request(
        &self,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;

    /// Sends approval or rejection for a tool execution confirmation.
    fn send_tool_confirmation(
        &self,
        trajectory_id: &str,
        step_index: u32,
        accepted: bool,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;

    /// Sends the result of a client-side tool execution back to the harness.
    fn send_tool_response(
        &self,
        id: &str,
        result: ToolResult,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;

    /// Sends answers to interactive user questions back to the harness.
    fn send_question_response(
        &self,
        trajectory_id: &str,
        step_index: u32,
        answers: QuestionHookResult,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;

    /// Gracefully closes the connection and releases resources.
    fn disconnect(
        &self,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;
}
```

### Key Design Points

| Method | Purpose |
|--------|---------|
| `conversation_id()` | Session identifier — learned from the harness during the first `StepUpdate` |
| `is_idle()` | Tracks idle state via `AtomicBool`; transitions on `TrajectoryStateUpdate` events |
| `receive_steps()` | Returns a `BoxStream` that yields `Step` events until the connection goes idle |
| `send()` | Serializes the prompt into an `InputEvent::UserInput` and sends over WebSocket |
| `send_halt_request()` | Cancels an in-progress agent turn |
| `send_tool_confirmation()` | Accepts or rejects a built-in tool call (policy enforcement) |
| `send_tool_response()` | Returns custom tool results to the harness for continued execution |
| `disconnect()` | Kills the subprocess (local) or closes the WebSocket (WASM) |

## AnyConnection

`AnyConnection` is a target-aware enum that dispatches to the concrete connection for the
current compilation target. It implements `Connection` by delegating every method to the
inner variant:

```rust,no_run
#[derive(Clone)]
pub enum AnyConnection {
    // Native platforms (not wasm32)
    #[cfg(not(target_arch = "wasm32"))]
    Local(std::sync::Arc<LocalConnection>),

    // WebAssembly targets
    #[cfg(target_arch = "wasm32")]
    Wasm(std::sync::Arc<WasmConnection>),

    // Testing only
    #[cfg(test)]
    Mock(std::sync::Arc<MockConnection>),
}
```

| Variant | Target | Transport |
|---------|--------|-----------|
| `Local(Arc<LocalConnection>)` | Native (macOS, Linux, Windows) | Subprocess → WebSocket |
| `Wasm(Arc<WasmConnection>)` | `wasm32` | TCP → WebSocket to remote harness |
| `Mock(Arc<MockConnection>)` | `#[cfg(test)]` only | In-memory channel for unit tests |

The `AnyConnection` enum is `Clone` and `Debug`, and is the concrete type stored inside
`Conversation`. Users typically never construct it directly — `Agent::start()` does this
internally.

## LocalConnectionStrategy

`LocalConnectionStrategy` is the connection factory for **native platforms**. It orchestrates
a multi-phase startup sequence:

### Connection Lifecycle

```text
┌──────────────────────────────────────────────────────────────┐
│  1. Spawn localharness subprocess (stdin/stdout/stderr)      │
│  2. Length-prefixed protobuf handshake over stdin/stdout      │
│     ├─ Send: InputConfig { storage_dir, client_info }        │
│     └─ Recv: OutputConfig { port, api_key }                  │
│  3. WebSocket upgrade to ws://localhost:{port}/               │
│     └─ Header: x-goog-api-key = {harness_api_key}           │
│  4. Send InitializeConversationEvent with HarnessConfig      │
│     ├─ GeminiConfig (model, API key, thinking level)         │
│     ├─ SystemInstructions (custom or appended)               │
│     ├─ Custom tools (name, description, JSON schema)         │
│     ├─ Harness-side tools (find, edit, run_command, etc.)    │
│     ├─ Workspaces (filesystem directories)                   │
│     └─ Skills paths                                          │
│  5. Spawn background tasks:                                  │
│     ├─ WS Sender loop (mpsc channel → ws_write)              │
│     ├─ WS Reader loop (ws_read → step_tx channel)            │
│     └─ Stderr reader (logs harness stderr as tracing::info)  │
└──────────────────────────────────────────────────────────────┘
```

### Configuration

```rust,no_run
use antigravity_sdk_rust::local::LocalConnectionStrategy;
use antigravity_sdk_rust::types::{
    GeminiConfig, CapabilitiesConfig, SystemInstructions,
};
use antigravity_sdk_rust::tools::ToolRunner;
use antigravity_sdk_rust::hooks::HookRunner;

let strategy = LocalConnectionStrategy::new(
    "/path/to/localharness".to_string(),     // binary_path
    GeminiConfig::default(),                  // gemini_config
    CapabilitiesConfig::default(),            // capabilities_config
    None,                                     // system_instructions
    Some("/tmp/agent-state".to_string()),     // save_dir
    vec!["/my/workspace".to_string()],        // workspaces
    vec![],                                   // skills_paths
    Some(ToolRunner::new()),                  // tool_runner
    Some(HookRunner::new()),                  // hook_runner
    "conv-123".to_string(),                   // conversation_id
);
```

### API Key Resolution

The strategy resolves the Gemini API key in this priority order:

1. `gemini_config.models.default.api_key` (model-level override)
2. `gemini_config.api_key` (global config)
3. `GEMINI_API_KEY` environment variable

For **Vertex AI** mode (`gemini_config.vertex = true`), either an API key (Express Mode)
or both `project` and `location` must be set.

### WebSocket Retry

The initial WebSocket connection retries up to **5 times** with exponential backoff
starting at 100ms. If all attempts fail, the subprocess is killed and an error is returned.

## WasmConnectionStrategy

`WasmConnectionStrategy` is the connection factory for **WebAssembly** targets. Instead of
spawning a subprocess, it connects to a **remote** `localharness` instance over TCP → WebSocket:

```rust,no_run
use antigravity_sdk_rust::wasm::WasmConnectionStrategy;
use antigravity_sdk_rust::types::{GeminiConfig, CapabilitiesConfig};

let strategy = WasmConnectionStrategy {
    gemini_config: GeminiConfig::default(),
    capabilities_config: CapabilitiesConfig::default(),
    system_instructions: None,
    save_dir: None,
    workspaces: vec![],
    skills_paths: vec![],
    tool_runner: None,
    hook_runner: None,
    conversation_id: "wasm-conv-1".to_string(),
};
```

### Environment Variables

| Variable | Default | Purpose |
|----------|---------|---------|
| `ANTIGRAVITY_HARNESS_HOST` | `127.0.0.1` | Remote harness host |
| `ANTIGRAVITY_HARNESS_PORT` | `8000` | Remote harness port |
| `ANTIGRAVITY_API_KEY` | — | API key (fallback after `GEMINI_API_KEY`) |

### Key Differences from LocalConnectionStrategy

| Aspect | Local | WASM |
|--------|-------|------|
| Subprocess | Spawns `localharness` | Connects to pre-running instance |
| Transport | `tokio-tungstenite` async WebSocket | `tungstenite` sync WebSocket with non-blocking TCP |
| Task spawning | `tokio::spawn` | `any_spawner::Executor::spawn_local` |
| API key env var | `GEMINI_API_KEY` | `ANTIGRAVITY_API_KEY` (then `GEMINI_API_KEY`) |
| Reader loop | Async `ws_read.next().await` | Polling `ws_lock.read()` with yield |

## MockConnection (Testing)

Available only in `#[cfg(test)]` builds, `MockConnection` provides an in-memory
connection for unit testing:

```rust,no_run
// Only available in test builds
use antigravity_sdk_rust::connection::MockConnection;
use std::sync::Arc;

let mock = Arc::new(MockConnection::new("test-conv-1"));

// Pre-load steps for the test
mock.steps_to_yield.lock().unwrap().push(/* Step { ... } */);

// After use, inspect what was sent
let sent = mock.sent_prompts.lock().unwrap();
assert_eq!(sent[0], "Hello");
```

### MockConnection Fields

| Field | Type | Purpose |
|-------|------|---------|
| `id` | `String` | Conversation ID returned by `conversation_id()` |
| `is_idle` | `AtomicBool` | Controllable idle state |
| `steps_to_yield` | `Mutex<Vec<Step>>` | Pre-loaded steps for `receive_steps()` |
| `sent_prompts` | `Mutex<Vec<String>>` | Records all prompts passed to `send()` |

## Direct Usage Example

For advanced users who need direct control over the connection layer without the
`Agent` builder:

```rust,no_run
use antigravity_sdk_rust::local::LocalConnectionStrategy;
use antigravity_sdk_rust::connection::{AnyConnection, Connection};
use antigravity_sdk_rust::conversation::Conversation;
use antigravity_sdk_rust::types::{
    GeminiConfig, CapabilitiesConfig, StreamChunk,
};
use futures_util::StreamExt;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // 1. Create the connection strategy
    let strategy = LocalConnectionStrategy::new(
        std::env::var("ANTIGRAVITY_HARNESS_PATH")
            .unwrap_or_else(|_| "./bin/localharness".to_string()),
        GeminiConfig {
            api_key: std::env::var("GEMINI_API_KEY").ok(),
            ..Default::default()
        },
        CapabilitiesConfig::default(),
        None,                                       // system_instructions
        None,                                       // save_dir
        vec![".".to_string()],                      // workspaces
        vec![],                                     // skills_paths
        None,                                       // tool_runner
        None,                                       // hook_runner
        String::new(),                              // conversation_id (auto-assigned)
    );

    // 2. Connect — spawns subprocess, handshakes, upgrades to WebSocket
    let conn = strategy.connect().await?;
    let any_conn = AnyConnection::Local(Arc::new(conn));

    // 3. Wrap in a Conversation for state tracking
    let conversation = Conversation::new(any_conn, Some(5000));

    // 4. Chat with streaming
    let mut stream = conversation.chat("Explain Rust's ownership model").await?;
    while let Some(chunk_res) = stream.next().await {
        match chunk_res? {
            StreamChunk::Thought { text, .. } => {
                eprint!("{}", text); // thoughts to stderr
            }
            StreamChunk::Text { text, .. } => {
                print!("{}", text); // response to stdout
            }
            StreamChunk::ToolCall(call) => {
                eprintln!("[Tool: {} args: {}]", call.name, call.args);
            }
        }
    }

    // 5. Inspect usage
    let usage = conversation.total_usage().await;
    eprintln!(
        "\nTokens — prompt: {}, candidates: {}, total: {}",
        usage.prompt_token_count,
        usage.candidates_token_count,
        usage.total_token_count,
    );

    // 6. Disconnect
    conversation.disconnect().await?;
    Ok(())
}
```

> **Note:** When using `LocalConnectionStrategy` directly, you bypass the `Agent`'s
> automatic binary resolution, policy enforcement, hook registration, and trigger startup.
> This is useful for embedding the SDK in custom orchestration frameworks.

## Python SDK Comparison

| Concept | Python SDK | Rust SDK |
|---------|-----------|----------|
| Connection trait | `Connection` ABC | `trait Connection: Send + Sync` |
| Dispatch enum | Runtime dispatch | `AnyConnection` compile-time `#[cfg]` dispatch |
| Local strategy | `LocalConnectionStrategy` class | `LocalConnectionStrategy` struct |
| WASM strategy | N/A | `WasmConnectionStrategy` (wasm32 target) |
| Subprocess init | Length-prefixed protobuf | Same protocol, `prost` + `tokio::process` |
| WebSocket lib | `websockets` (Python) | `tokio-tungstenite` / `tungstenite` |
| Idle detection | Event-based | `AtomicBool` + `TrajectoryStateUpdate` sentinel pattern |
| Mock for tests | `unittest.mock` | `MockConnection` with `#[cfg(test)]` |

## Architecture Diagram

```text
┌─────────────────────────┐
│      Agent::start()     │
│  (resolves binary,      │
│   wires policies/tools) │
└──────────┬──────────────┘
           │
           ▼
┌─────────────────────────┐
│  ConnectionStrategy     │
│  ├─ Local (native)      │
│  └─ Wasm (wasm32)       │
└──────────┬──────────────┘
           │ .connect()
           ▼
┌─────────────────────────┐
│  AnyConnection          │
│  (enum dispatch)        │
└──────────┬──────────────┘
           │
           ▼
┌─────────────────────────┐
│  Conversation           │
│  (state tracking,       │
│   streaming, usage)     │
└─────────────────────────┘
```
