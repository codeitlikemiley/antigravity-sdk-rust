# Conversation

Stateful session management for the Antigravity Rust SDK.

## Overview

`Conversation` is a stateful wrapper around a [`Connection`](./connections.md) that tracks
step history, accumulates token usage metadata, and processes stream chunks. It is the
primary interface for interacting with an active agent session — whether you want streaming
chunk-by-chunk output or a simple blocking call-and-response.

Most users access `Conversation` through `Agent<Started>`:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let agent = Agent::builder()
        .allow_all()
        .build()
        .start().await?;

    // Get the active Conversation (Arc<Conversation>)
    let conversation = agent.conversation();

    // Use it for streaming or blocking chat
    let response = conversation.chat_to_completion("Hello!").await?;
    println!("{}", response.text);

    agent.stop().await
}
```

## Creating a Conversation

`Conversation` wraps an `AnyConnection` and an optional history size limit:

```rust,no_run
use antigravity_sdk_rust::conversation::Conversation;
use antigravity_sdk_rust::connection::AnyConnection;

// Created internally by Agent::start(), but can be constructed directly:
let conversation = Conversation::new(
    any_connection,     // AnyConnection (Local, Wasm, or Mock)
    Some(5000),         // max_history_size (None → default 10,000; Some(0) → unlimited)
);
```

### Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `conn` | `AnyConnection` | — | The underlying connection to the harness |
| `max_history_size` | `Option<usize>` | `Some(10_000)` | Max steps retained in memory. `None` uses 10,000. `Some(0)` disables trimming. |

## Key Methods

### Chatting

#### `chat(prompt)` — Streaming

Sends a prompt and returns a `BoxStream<StreamChunk>` for real-time processing:

```rust,no_run
use antigravity_sdk_rust::types::StreamChunk;
use futures_util::StreamExt;

let mut stream = conversation.chat("Explain monads").await?;

while let Some(chunk_res) = stream.next().await {
    match chunk_res? {
        StreamChunk::Thought { text, step_index } => {
            // Model's internal reasoning (thinking tokens)
            eprint!("{}", text);
        }
        StreamChunk::Text { text, step_index } => {
            // Response text fragments
            print!("{}", text);
        }
        StreamChunk::ToolCall(call) => {
            // The model requested a tool execution
            println!("[Tool: {} | Args: {}]", call.name, call.args);
        }
    }
}
```

**Signature:**
```rust,no_run
pub async fn chat(
    &self,
    prompt: &str,
) -> Result<BoxStream<'static, Result<StreamChunk, anyhow::Error>>, anyhow::Error>
```

#### `chat_to_completion(prompt)` — Blocking

Sends a prompt and waits for the complete response, accumulating all chunks internally:

```rust,no_run
let response = conversation.chat_to_completion("What is 2 + 2?").await?;

println!("Response: {}", response.text);
println!("Thinking: {}", response.thinking);
println!("Steps: {}", response.steps.len());
println!("Total tokens: {}", response.usage_metadata.total_token_count);
```

**Signature:**
```rust,no_run
pub async fn chat_to_completion(
    &self,
    prompt: &str,
) -> Result<ChatResponse, anyhow::Error>
```

### Sending & Receiving (Low-Level)

| Method | Signature | Description |
|--------|-----------|-------------|
| `send(prompt)` | `async fn send(&self, prompt: &str) -> Result<()>` | Sends a raw prompt and registers a turn boundary. Does **not** return a stream. |
| `receive_steps()` | `fn receive_steps(&self) -> BoxStream<'static, Result<Step>>` | Raw step-level stream. Inserts steps into history, tracks compaction, enforces history limits. |
| `receive_chunks()` | `fn receive_chunks(&self) -> BoxStream<'static, Result<StreamChunk>>` | Filters `receive_steps()` into high-level `StreamChunk` events. Only emits model→user content. |

#### `receive_chunks()` Filtering Logic

The chunk stream applies these rules:

1. **Thought chunks** — emitted when `source == Model`, `target == User`, and `thinking_delta` is non-empty
2. **Text chunks** — emitted when `source == Model`, `target == User`, and `content_delta` is non-empty
3. **ToolCall chunks** — emitted for each `ToolCall` in the step, **deduplicated by `call.id`** (empty IDs are never deduplicated)
4. **Environment-targeted steps** are silently filtered out (e.g., harness internal tool dispatches)

### Connection Access

| Method | Return Type | Description |
|--------|-------------|-------------|
| `connection()` | `AnyConnection` | Returns the underlying connection (clone of `Arc`) |
| `conversation_id()` | `&str` | Session ID (delegates to `Connection::conversation_id()`) |
| `is_idle()` | `bool` | Whether the connection is idle |
| `disconnect()` | `async -> Result<()>` | Gracefully closes the connection |

## StreamChunk

The streaming fragment enum used by `chat()` and `receive_chunks()`:

```rust,no_run
use antigravity_sdk_rust::types::ToolCall;

#[derive(Debug, Clone)]
pub enum StreamChunk {
    /// Model's internal reasoning/thinking fragment.
    Thought {
        /// Index of the step that produced this chunk.
        step_index: u32,
        /// Thinking text delta.
        text: String,
    },

    /// Response text fragment directed at the user.
    Text {
        /// Index of the step that produced this chunk.
        step_index: u32,
        /// Text content delta.
        text: String,
    },

    /// A complete tool call requested by the model.
    ToolCall(ToolCall),
}
```

### ToolCall Structure

```rust,no_run
#[derive(Debug, Clone)]
pub struct ToolCall {
    /// Unique correlation ID for this call.
    pub id: String,
    /// Name of the tool to invoke.
    pub name: String,
    /// Arguments as a JSON value.
    pub args: serde_json::Value,
    /// Optional canonical filesystem path (for file-targeting tools).
    pub canonical_path: Option<String>,
}
```

## Streaming Example

The complete streaming pattern from `examples/streaming.rs`:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::types::StreamChunk;
use futures_util::StreamExt;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();

    let agent = Agent::builder()
        .default_model("gemini-3.5-flash")
        .allow_all()
        .build()
        .start().await?;

    let conversation = agent.conversation();

    let prompt = "Solve this riddle: I speak without a mouth \
        and hear without ears. What am I?";
    println!("User: {}\n", prompt);

    let mut stream = conversation.chat(prompt).await?;

    while let Some(chunk_res) = stream.next().await {
        match chunk_res? {
            StreamChunk::Thought { text, .. } => {
                // Print reasoning tokens as they arrive
                print!("{}", text);
                std::io::Write::flush(&mut std::io::stdout())?;
            }
            StreamChunk::Text { text, .. } => {
                // Print response text tokens as they arrive
                print!("{}", text);
                std::io::Write::flush(&mut std::io::stdout())?;
            }
            StreamChunk::ToolCall(call) => {
                println!(
                    "\n[Tool Call: {} with args: {}]",
                    call.name, call.args
                );
            }
        }
    }

    println!(); // Final newline
    agent.stop().await?;
    Ok(())
}
```

## State Management

`Conversation` maintains a `ConversationState` protected by an async `Mutex`:

```rust,no_run
pub struct ConversationState {
    /// All executed steps (prompts, tool calls, responses, compaction markers).
    pub steps: Vec<Step>,
    /// Step indices marking each user prompt turn boundary.
    pub turn_start_indices: Vec<usize>,
    /// Step indices where history compaction occurred.
    pub compaction_indices: Vec<usize>,
    /// Cumulative token usage across all turns.
    pub cumulative_usage: UsageMetadata,
    /// Token usage for the current active turn.
    pub turn_usage: Option<UsageMetadata>,
}
```

### State Query Methods

All state methods are `async` because they acquire the internal `Mutex`:

| Method | Return Type | Description |
|--------|-------------|-------------|
| `history()` | `Vec<Step>` | Clone of the full step history |
| `turn_count()` | `usize` | Number of user-initiated turns |
| `compaction_indices()` | `Vec<usize>` | Where the harness compacted history |
| `last_response()` | `String` | Text content of the last complete model response |
| `total_usage()` | `UsageMetadata` | Cumulative token usage for the entire session |
| `last_turn_usage()` | `Option<UsageMetadata>` | Token usage for the most recent turn |
| `clear_history()` | `()` | Resets all steps, turns, compactions, and usage stats |

### Usage Example

```rust,no_run
// After a chat interaction
let history = conversation.history().await;
println!("Total steps: {}", history.len());
println!("Turns completed: {}", conversation.turn_count().await);

let total = conversation.total_usage().await;
println!("Session tokens: {} prompt, {} generated, {} total",
    total.prompt_token_count,
    total.candidates_token_count,
    total.total_token_count,
);

if let Some(turn) = conversation.last_turn_usage().await {
    println!("Last turn: {} thinking tokens", turn.thoughts_token_count);
}

// Check if compaction happened
let compactions = conversation.compaction_indices().await;
if !compactions.is_empty() {
    println!("History was compacted at step indices: {:?}", compactions);
}
```

### History Auto-Trimming

When `max_history_size > 0` and the step count exceeds the limit, older steps are
drained from the front. Turn and compaction indices are adjusted accordingly:

```rust,no_run
// Only keep the last 100 steps in memory
let conversation = Conversation::new(any_connection, Some(100));

// Disable auto-trimming entirely
let conversation = Conversation::new(any_connection, Some(0));

// Use the default (10,000 steps)
let conversation = Conversation::new(any_connection, None);
```

## ChatResponse

The complete result returned by `chat_to_completion()`:

```rust,no_run
#[derive(Debug, Clone)]
pub struct ChatResponse {
    /// Combined final response text.
    pub text: String,
    /// Combined model reasoning/thinking text.
    pub thinking: String,
    /// All steps executed during this turn.
    pub steps: Vec<Step>,
    /// Cumulative token usage metrics.
    pub usage_metadata: UsageMetadata,
}
```

### UsageMetadata

```rust,no_run
#[derive(Debug, Clone, Default)]
pub struct UsageMetadata {
    /// Tokens included in the request prompt.
    pub prompt_token_count: i32,
    /// Tokens generated in model candidates.
    pub candidates_token_count: i32,
    /// Total combined tokens.
    pub total_token_count: i32,
    /// Cache-hit content tokens.
    pub cached_content_token_count: i32,
    /// Tokens consumed during reasoning/thinking.
    pub thoughts_token_count: i32,
}
```

## Step Model

Each event in the trajectory is represented as a `Step`:

```rust,no_run
#[derive(Debug, Clone)]
pub struct Step {
    pub id: String,                              // Unique step ID
    pub step_index: u32,                         // Position in trajectory
    pub r#type: StepType,                        // TextResponse, ToolCall, Compaction, Finish, etc.
    pub source: StepSource,                      // System, User, Model
    pub target: StepTarget,                      // User, Environment
    pub status: StepStatus,                      // Active, Done, Error, TerminalError, etc.
    pub content: String,                         // Full accumulated text
    pub content_delta: String,                   // Incremental text delta
    pub thinking: String,                        // Full accumulated thinking
    pub thinking_delta: String,                  // Incremental thinking delta
    pub tool_calls: Vec<ToolCall>,               // Tool calls in this step
    pub error: String,                           // Error message (if any)
    pub is_complete_response: Option<bool>,       // True for final model response
    pub structured_output: Option<serde_json::Value>, // Parsed structured output
    pub usage_metadata: Option<UsageMetadata>,   // Per-step token usage
    pub cascade_id: String,                      // Execution cascade group
    pub trajectory_id: String,                   // Sub-agent trajectory group
    pub http_code: u32,                          // HTTP status code (if applicable)
}
```

### Step Enums

| Enum | Variants |
|------|----------|
| `StepType` | `TextResponse`, `ToolCall`, `SystemMessage`, `Compaction`, `Finish`, `Unknown` |
| `StepSource` | `System`, `User`, `Model`, `Unknown` |
| `StepTarget` | `User`, `Environment`, `Unspecified`, `Unknown` |
| `StepStatus` | `Active`, `Done`, `WaitingForUser`, `Error`, `Canceled`, `TerminalError`, `Unknown` |

## Python SDK Comparison

| Concept | Python SDK | Rust SDK |
|---------|-----------|----------|
| Conversation class | `Conversation` | `Conversation` (same name) |
| Streaming | `async for chunk in conversation.chat(prompt)` | `conversation.chat(prompt).await?.next().await` |
| Blocking | `conversation.chat_to_completion(prompt)` | `conversation.chat_to_completion(prompt).await?` |
| History | `conversation.history` (property) | `conversation.history().await` (async method) |
| Turn count | `conversation.turn_count` | `conversation.turn_count().await` |
| Token usage | `conversation.total_usage` | `conversation.total_usage().await` |
| State mutex | GIL + threading lock | `tokio::sync::Mutex<ConversationState>` |
| Stream type | `AsyncIterator[StreamChunk]` | `BoxStream<'static, Result<StreamChunk>>` |
| Max history | `max_history_size` param | `max_history_size: Option<usize>` (default 10,000) |

> **Key difference:** In the Rust SDK, all state-querying methods are `async` because the
> internal state is protected by a `tokio::sync::Mutex`. In Python, these are synchronous
> properties protected by the GIL.
