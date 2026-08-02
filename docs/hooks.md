# Hooks

Agent lifecycle interception and policies.

## Overview

Hooks observe or modify agent behavior at key lifecycle points. They let you
log events, enforce safety rules, rate-limit turns, audit tool calls, and
recover from errors — all without modifying the core agent loop.

### Python SDK vs Rust SDK

The Python SDK splits hooks into separate base classes by category:

| Python Category | Python Base Classes | Rust Equivalent |
|---|---|---|
| **Inspect** (read-only) | `OnSessionStartHook`, `PostToolCallHook`, `OnSessionEndHook`, `PostTurnHook`, `OnCompactionHook` | Default no-op methods on `Hook` |
| **Decide** (blocking) | `PreTurnHook`, `PreToolCallDecideHook` | `pre_turn()`, `pre_tool_call()` return `HookResult` |
| **Transform** (modifying) | `OnToolErrorHook`, `OnInteractionHook` | `on_tool_error()` rewords a failure, `on_interaction()` answers questions |

The Rust SDK merges all of these into a **single `Hook` trait** with 9 async
methods. Every method has a default no-op implementation, so you only override
what you need.

---

## Hook Trait

All 9 lifecycle methods are defined on the `Hook` trait. Each has a default
no-op implementation so you only need to override the methods relevant to your
use case:

```rust,no_run
use antigravity_sdk_rust::hooks::Hook;
use antigravity_sdk_rust::types::{
    AskQuestionEntry, ChatResponse, HookResult, QuestionHookResult,
    ToolCall, ToolResult,
};

pub trait Hook: Send + Sync {
    // ── Session lifecycle ──────────────────────────────────────────

    /// Called when the agent establishes a connection and starts a session.
    async fn on_session_start(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    /// Called when the session is ending (agent shutdown or disconnect).
    async fn on_session_end(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    // ── Turn lifecycle ─────────────────────────────────────────────

    /// Intercepts the start of a user turn before the LLM processes the prompt.
    /// Return `allow: false` to halt execution.
    async fn pre_turn(&self) -> Result<HookResult, anyhow::Error> {
        Ok(HookResult { allow: true, message: String::new() })
    }

    /// Called when a turn completes, receiving the model's final text.
    async fn post_turn(&self, _response: &str) -> Result<(), anyhow::Error> {
        Ok(())
    }

    // ── Tool lifecycle ─────────────────────────────────────────────

    /// Intercepts a tool call before execution.
    /// Return `allow: false` to prevent the tool from running.
    async fn pre_tool_call(&self, _tool_call: &ToolCall) -> Result<HookResult, anyhow::Error> {
        Ok(HookResult { allow: true, message: String::new() })
    }

    /// Called after a tool successfully returns a result.
    async fn post_tool_call(&self, _result: &ToolResult) -> Result<(), anyhow::Error> {
        Ok(())
    }

    // ── Error recovery ─────────────────────────────────────────────

    /// Called when a tool execution fails.
    /// Return `Some(message)` to replace the error text the model is shown;
    /// `None` leaves it as it is. A failure cannot be turned into a success.
    async fn on_tool_error(
        &self,
        _error: &anyhow::Error,
    ) -> Result<Option<String>, anyhow::Error> {
        Ok(None)
    }

    // ── User interaction ───────────────────────────────────────────

    /// Intercepts interactive user questions.
    /// Return `Some(result)` to answer programmatically.
    async fn on_interaction(
        &self,
        _questions: &[AskQuestionEntry],
    ) -> Result<Option<QuestionHookResult>, anyhow::Error> {
        Ok(None)
    }

    // ── History compaction ─────────────────────────────────────────

    /// Called when the conversation history is compacted, receiving the
    /// compaction step itself.
    async fn on_compaction(&self, _step: &Step) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
```

### Method Reference

| Method | Category | Return | Short-Circuit? |
|---|---|---|---|
| `on_session_start` | Inspect | `()` | No |
| `on_session_end` | Inspect | `()` | No |
| `pre_turn` | Decide | `HookResult` | Yes — first `allow: false` |
| `post_turn` | Inspect | `()` | No |
| `pre_tool_call` | Decide | `HookResult` | Yes — first `allow: false` |
| `post_tool_call` | Inspect | `()` | No |
| `on_tool_error` | Transform | `(HookResult, Option<Value>)` | Yes — first `allow: true` |
| `on_interaction` | Transform | `Option<QuestionHookResult>` | Yes — first `Some(result)` |
| `on_compaction` | Inspect | `()` | No |

---

## DynHook (Object-Safe Wrapper)

Rust's `async fn` in traits produces `impl Future` return types, which are not
object-safe — you cannot use `dyn Hook` directly. The SDK solves this with a
companion `DynHook` trait that wraps every method in a `BoxFuture`:

```rust,no_run
use futures_util::future::BoxFuture;
use antigravity_sdk_rust::types::{HookResult, ToolCall, ToolResult, ChatResponse, AskQuestionEntry, QuestionHookResult};

/// Object-safe version of `Hook`, used internally for dynamic dispatch.
pub trait DynHook: Send + Sync {
    fn on_session_start(&self) -> BoxFuture<'_, Result<(), anyhow::Error>>;
    fn pre_turn(&self) -> BoxFuture<'_, Result<HookResult, anyhow::Error>>;
    fn pre_tool_call<'a>(&'a self, tool_call: &'a ToolCall) -> BoxFuture<'a, Result<HookResult, anyhow::Error>>;
    fn post_tool_call<'a>(&'a self, result: &'a ToolResult) -> BoxFuture<'a, Result<(), anyhow::Error>>;
    fn on_tool_error<'a>(&'a self, error: &'a anyhow::Error) -> BoxFuture<'a, Result<(HookResult, Option<serde_json::Value>), anyhow::Error>>;
    fn on_interaction<'a>(&'a self, questions: &'a [AskQuestionEntry]) -> BoxFuture<'a, Result<Option<QuestionHookResult>, anyhow::Error>>;
    fn on_session_end(&self) -> BoxFuture<'_, Result<(), anyhow::Error>>;
    fn post_turn<'a>(&'a self, response: &'a str) -> BoxFuture<'a, Result<(), anyhow::Error>>;
    fn on_compaction<'a>(&'a self, step: &'a Step) -> BoxFuture<'a, Result<(), anyhow::Error>>;
}
```

A **blanket implementation** automatically bridges the two traits:

```rust,no_run
// Any type that implements Hook also implements DynHook — for free.
// impl<T: Hook + ?Sized> DynHook for T { ... }
```

This means you always implement `Hook` (the ergonomic trait) and the SDK
converts it to `Arc<dyn DynHook>` for storage and dispatch. You never need to
implement `DynHook` directly.

---

## HookRunner

The `HookRunner` is the internal dispatch engine that manages registered hooks
and dispatches lifecycle events sequentially. It stores hooks as
`Vec<Arc<dyn DynHook>>` behind a `tokio::sync::RwLock`.

### Dispatch Behavior

| Dispatch Method | Behavior |
|---|---|
| `dispatch_session_start` | Calls all hooks sequentially |
| `dispatch_session_end` | Calls all hooks sequentially |
| `dispatch_pre_turn` | **Short-circuits** at first `allow: false` |
| `dispatch_post_turn` | Calls all hooks sequentially |
| `dispatch_pre_tool_call` | **Short-circuits** at first `allow: false` |
| `dispatch_post_tool_call` | Calls all hooks sequentially |
| `dispatch_on_tool_error` | **Short-circuits** at first `allow: true` (recovery) |
| `dispatch_interaction` | **Short-circuits** at first `Some(result)` |
| `dispatch_on_compaction` | Calls all hooks sequentially |

Short-circuiting means that once a decisive result is found, remaining hooks
are skipped for that event. This allows safety hooks to block execution
without later hooks overriding the decision.

### Registration

```rust,no_run
use antigravity_sdk_rust::hooks::{HookRunner, DynHook};
use std::sync::Arc;

let runner = HookRunner::new();

// Register a hook
// runner.register(Arc::new(my_hook)).await;
```

---

## HookContext (Hierarchical State)

`HookContext` provides a parent-chaining key-value store for sharing state
across hook invocations at different scopes. Values are stored as
`serde_json::Value` internally.

### 3-Level Hierarchy

```text
┌─────────────────────────────────┐
│  Session Context (root)         │  ← HookContext::new()
│  "session_id" = "abc-123"       │
│                                 │
│  ┌───────────────────────────┐  │
│  │  Turn Context             │  │  ← HookContext::child(session)
│  │  "turn_count" = 5         │  │
│  │                           │  │
│  │  ┌─────────────────────┐  │  │
│  │  │  Operation Context  │  │  │  ← HookContext::child(turn)
│  │  │  "tool_name" = "X"  │  │  │
│  │  └─────────────────────┘  │  │
│  └───────────────────────────┘  │
└─────────────────────────────────┘
```

### Key Semantics

- **`get(key)`** — searches the local store first, then walks up the parent
  chain. Returns `None` if the key is not found at any level.
- **`set(key, value)`** — writes only to the **local** store. This shadows
  (but does not mutate) any parent value with the same key.
- **`has_parent()`** — returns `true` if this context is not the root.

### Usage

```rust,no_run
use antigravity_sdk_rust::context::HookContext;
use std::sync::Arc;

// Session-level context (root)
let session = Arc::new(HookContext::new());
session.set("user_id", "user-42");

// Turn-level context
let turn = Arc::new(HookContext::child(session.clone()));
turn.set("turn_number", 1u32);

// Operation-level context
let operation = HookContext::child(turn.clone());
operation.set("tool_name", "read_file");

// get() walks up the chain:
assert_eq!(operation.get::<String>("tool_name"), Some("read_file".to_string()));
assert_eq!(operation.get::<u32>("turn_number"), Some(1));
assert_eq!(operation.get::<String>("user_id"), Some("user-42".to_string()));

// set() is local-only — parent is unchanged:
operation.set("user_id", "override");
assert_eq!(operation.get::<String>("user_id"), Some("override".to_string()));
assert_eq!(session.get::<String>("user_id"), Some("user-42".to_string()));
```

---

## PolicyEnforcer

`PolicyEnforcer` is a built-in `Hook` implementation that enforces safety
policies on tool calls. It intercepts `pre_tool_call` and evaluates registered
policies against a 9-bucket priority system.

### Priority Bucket System

Policies are sorted into 9 buckets organized by **specificity** (3 levels) ×
**decision type** (3 types). Lower bucket index = higher priority:

| Bucket | Specificity | Decision | Index |
|---|---|---|---|
| Specific Deny | Exact tool name | `Deny` | 0 (highest) |
| Specific Ask | Exact tool name | `AskUser` | 1 |
| Specific Allow | Exact tool name | `Approve` | 2 |
| Prefix Deny | Server wildcard (`server/*`) | `Deny` | 3 |
| Prefix Ask | Server wildcard | `AskUser` | 4 |
| Prefix Allow | Server wildcard | `Approve` | 5 |
| Global Deny | Global wildcard (`*`) | `Deny` | 6 |
| Global Ask | Global wildcard | `AskUser` | 7 |
| Global Allow | Global wildcard | `Approve` | 8 (lowest) |

**Key rules:**
- Specific rules always override prefix and global rules.
- Prefix rules (`server/*`) override global wildcards (`*`).
- At the same specificity, **Deny beats Ask beats Allow**.
- If no policy matches, the tool call is **allowed** (open by default).

### Policy Helper Functions

```rust,no_run
use antigravity_sdk_rust::policy;

// ── Single-tool policies ───────────────────────────────────────────
let _ = policy::allow("VIEW_FILE");            // Approve a specific tool
let _ = policy::deny("RUN_COMMAND");           // Deny a specific tool
let _ = policy::ask_user("RUN_COMMAND", |_tc| {
    // Return true = user approved, false = user denied
    true
});

// ── Wildcard policies ──────────────────────────────────────────────
let _ = policy::allow_all();                   // Approve all tools (wildcard)
let _ = policy::deny_all();                    // Deny all tools (wildcard)

// ── Composite policies ────────────────────────────────────────────
// Require user confirmation for RUN_COMMAND, allow everything else:
let _ = policy::confirm_run_command(Some(std::sync::Arc::new(|_| true)));

// Restrict filesystem tools to specific workspace directories:
let _ = policy::workspace_only(vec!["/my/project".to_string()]);
```

### MCP Server Policies

For controlling tools exposed by MCP (Model Context Protocol) servers:

```rust,no_run
use antigravity_sdk_rust::policy;
use antigravity_sdk_rust::types::McpServerConfig;

let server = McpServerConfig::Stdio {
    name: "math".to_string(),
    command: "math-server".to_string(),
    args: vec![],
    enabled_tools: None,
    disabled_tools: None,
};

// Allow all tools from the server
let _ = policy::allow_mcp(&server, None);

// Deny specific tools from the server
let _ = policy::deny_mcp(&server, Some(&["dangerous_calc"]));

// Ask user for specific MCP tools
let _ = policy::ask_user_mcp(&server, Some(&["execute"]), |_| true);
```

### Conditional Policies with `when()`

Policies can include a predicate that narrows when they apply:

```rust,no_run
use antigravity_sdk_rust::policy;

// Only deny run_command when the command contains "rm"
let _ = policy::deny("RUN_COMMAND").when(|tc| {
    tc.args
        .get("CommandLine")
        .and_then(|v| v.as_str())
        .map_or(false, |s| s.contains("rm"))
});
```

### Creating a PolicyEnforcer

Use `policy::enforce()` to validate and compile policies:

```rust,no_run
use antigravity_sdk_rust::policy;

// Without MCP servers
let enforcer = policy::enforce(
    vec![
        policy::deny("RUN_COMMAND"),
        policy::allow_all(),
    ],
    None,  // no MCP servers
).expect("policy validation failed");

// enforce() will return an error if:
// - An AskUser policy is missing its handler callback
// - MCP policies are present but no MCP servers are registered (fail-closed)
```

---

## Examples

### 1. Logging Hook

A simple hook that logs all lifecycle events:

```rust,no_run
use antigravity_sdk_rust::hooks::Hook;
use antigravity_sdk_rust::types::{ChatResponse, HookResult, ToolCall, ToolResult};

struct LoggingHook;

impl Hook for LoggingHook {
    async fn on_session_start(&self) -> Result<(), anyhow::Error> {
        println!("🟢 Session started");
        Ok(())
    }

    async fn on_session_end(&self) -> Result<(), anyhow::Error> {
        println!("🔴 Session ended");
        Ok(())
    }

    async fn pre_tool_call(&self, tool_call: &ToolCall) -> Result<HookResult, anyhow::Error> {
        println!("🔧 Calling tool: {}", tool_call.name);
        Ok(HookResult { allow: true, message: String::new() })
    }

    async fn post_tool_call(&self, result: &ToolResult) -> Result<(), anyhow::Error> {
        println!("✅ Tool {} completed", result.name);
        Ok(())
    }

    async fn post_turn(&self, response: &str) -> Result<(), anyhow::Error> {
        println!("💬 Response length: {} chars", response.len());
        Ok(())
    }
}
```

### 2. Rate-Limiting Hook

A `pre_turn` hook that enforces a maximum number of turns per session:

```rust,no_run
use antigravity_sdk_rust::hooks::Hook;
use antigravity_sdk_rust::types::HookResult;
use std::sync::atomic::{AtomicU32, Ordering};

struct RateLimitHook {
    max_turns: u32,
    turn_count: AtomicU32,
}

impl RateLimitHook {
    fn new(max_turns: u32) -> Self {
        Self {
            max_turns,
            turn_count: AtomicU32::new(0),
        }
    }
}

impl Hook for RateLimitHook {
    async fn pre_turn(&self) -> Result<HookResult, anyhow::Error> {
        let count = self.turn_count.fetch_add(1, Ordering::SeqCst);
        if count >= self.max_turns {
            Ok(HookResult {
                allow: false,
                message: format!(
                    "Rate limit exceeded: {} of {} turns used",
                    count, self.max_turns
                ),
            })
        } else {
            Ok(HookResult { allow: true, message: String::new() })
        }
    }
}
```

### 3. Tool Audit Hook

A `post_tool_call` hook that records all tool invocations for auditing:

```rust,no_run
use antigravity_sdk_rust::hooks::Hook;
use antigravity_sdk_rust::types::{HookResult, ToolCall, ToolResult};
use std::sync::Mutex;

struct AuditRecord {
    tool_name: String,
    success: bool,
    timestamp: std::time::Instant,
}

struct AuditHook {
    records: Mutex<Vec<AuditRecord>>,
}

impl AuditHook {
    fn new() -> Self {
        Self {
            records: Mutex::new(Vec::new()),
        }
    }
}

impl Hook for AuditHook {
    async fn pre_tool_call(&self, tool_call: &ToolCall) -> Result<HookResult, anyhow::Error> {
        println!("📝 Audit: tool '{}' invoked", tool_call.name);
        Ok(HookResult { allow: true, message: String::new() })
    }

    async fn post_tool_call(&self, result: &ToolResult) -> Result<(), anyhow::Error> {
        let record = AuditRecord {
            tool_name: result.name.clone(),
            success: result.error.is_none(),
            timestamp: std::time::Instant::now(),
        };
        if let Ok(mut records) = self.records.lock() {
            records.push(record);
        }
        Ok(())
    }
}
```

### 4. Error Recovery Hook

An `on_tool_error` hook that provides fallback values when specific tools fail:

```rust,no_run
use antigravity_sdk_rust::hooks::Hook;
use antigravity_sdk_rust::types::HookResult;

struct ErrorRecoveryHook;

impl Hook for ErrorRecoveryHook {
    async fn on_tool_error(
        &self,
        error: &anyhow::Error,
    ) -> Result<(HookResult, Option<serde_json::Value>), anyhow::Error> {
        let msg = error.to_string();

        // Provide a fallback value for network-related errors
        if msg.contains("timeout") || msg.contains("connection refused") {
            println!("⚠️  Network error detected, providing fallback");
            Ok((
                HookResult {
                    allow: true,
                    message: "Recovered from network error with fallback".to_string(),
                },
                Some(serde_json::json!({
                    "error": "network_unavailable",
                    "fallback": true,
                    "message": "Service temporarily unavailable, using cached data"
                })),
            ))
        } else {
            // Let other hooks handle it, or propagate the error
            Ok((
                HookResult {
                    allow: false,
                    message: msg,
                },
                None,
            ))
        }
    }
}
```

### 5. Registering Hooks with the Agent Builder

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::hooks::DynHook;
use std::sync::Arc;

// Assuming LoggingHook and RateLimitHook are defined as above

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let agent = Agent::builder()
        // Register hooks via the builder
        .hook(Arc::new(LoggingHook) as Arc<dyn DynHook>)
        // .hook(Arc::new(RateLimitHook::new(100)) as Arc<dyn DynHook>)
        .allow_all()
        .build();

    let agent = agent.start().await?;
    let response = agent.chat("Hello!").await?;
    println!("{}", response.text);
    agent.stop().await?;
    Ok(())
}

struct LoggingHook;
impl antigravity_sdk_rust::hooks::Hook for LoggingHook {
    async fn on_session_start(&self) -> Result<(), anyhow::Error> {
        println!("Session started");
        Ok(())
    }
}
```

You can also register hooks after construction but before starting:

```rust,no_run
use antigravity_sdk_rust::agent::Agent;
use antigravity_sdk_rust::hooks::DynHook;
use std::sync::Arc;

struct MyHook;
impl antigravity_sdk_rust::hooks::Hook for MyHook {}

let mut agent = Agent::builder().allow_all().build();
agent.register_hook(Arc::new(MyHook) as Arc<dyn DynHook>);
// agent.start().await?;
```

## `on_tool_error` rewords, it does not recover

A tool that failed stays failed. `on_tool_error` returns `Option<String>`: the
error text the model is shown, or `None` to leave it alone. The first hook with
an opinion wins, and a hook that itself errors is logged and skipped.

It used to be able to substitute a result and clear the error, which reported a
tool that had failed to the model as having worked and downgraded the step from
`Error` to `Done`. Upstream narrowed this for the same reason. Recovery belongs
inside the tool, where it can decide whether the fallback is honest.

```rust,no_run
# use antigravity_sdk_rust::hooks::Hook;
struct Explain;

impl Hook for Explain {
    async fn on_tool_error(&self, error: &anyhow::Error) -> Result<Option<String>, anyhow::Error> {
        if error.to_string().contains("response too large") {
            // The model can act on this; it cannot act on a stack trace.
            return Ok(Some("the result was too large — request fewer rows".to_string()));
        }
        Ok(None)
    }
}
```

## A hook that errors denies the call

`pre_tool_call` returning `Err` is not "no objection" — the call is **denied**
and the model is told the gate could not decide. A gate that cannot reach its
policy store, or whose predicate panicked, must not fall open.

This is a behaviour change: a hook that used to error and let tools through now
blocks them. If a hook has a failure mode you want to tolerate, handle it inside
the hook and return `allow: true` explicitly.

## `post_tool_call` and subagents

A `START_SUBAGENT` call completes when the subagent's trajectory goes idle —
the harness sends no tool response for it. `post_tool_call` fires at that point
with `name = "START_SUBAGENT"` and `result` set to the subagent's last model
text, falling back to its trajectory id when it produced none.

## `pre_turn` can refuse a turn

Returning `allow: false` from `pre_turn` stops the prompt from being sent at
all: `send()` returns the hook's message as an error rather than starting a
turn that produces nothing. As with `pre_tool_call`, a hook that *errors*
refuses the turn too.

## When the turn hooks fire

`post_turn` fires at the terminal user-facing model step, carrying that step's
text. `on_compaction` fires on the compaction step, carrying the step — a hook
that archives history needs its index and trajectory, not only its summary
text.

Both were defined and dispatched from nowhere until now; a `post_turn` hook
simply never ran.

## Declaring hook kinds

`Hook::declares()` returns the `HookKinds` an implementation wants the
**harness** to call. It is opt-in and defaults to `NONE`.

This does not affect local dispatch: every method is called by the runner
either way. Declaring is what will put a kind into
`HarnessConfig.enabled_hooks`, and the harness then blocks its turn waiting for
an answer — so declare only what you handle.

```rust,no_run
# use antigravity_sdk_rust::hooks::Hook;
# use antigravity_sdk_rust::hook_dispatch::HookKinds;
# struct Audit;
impl Hook for Audit {
    fn declares(&self) -> HookKinds {
        HookKinds::PRE_TOOL | HookKinds::POST_TOOL
    }
}
```

`enabled_hooks` now carries exactly what registered hooks declared. It became
safe to emit only once the router existed: the harness blocks its turn waiting
for a `CallHookResponse` for every kind named in that field, so emitting it
first would have deadlocked the turn rather than being a no-op.

## Harness-side hook requests

The harness dispatches `CallHookRequest` for every declared kind and waits.
Every path through the router answers, including a request it does not
understand — that one is answered with `error_message`, which the harness treats
as a hook failure. Not answering is a deadlock.

Deny semantics match the local gates: a hook that errors refuses, because a gate
that cannot decide must not fall open. Rewriting the model's arguments is not
offered, so `modified_arguments_json` comes back unset rather than echoed.

## Hook and tool state

Both `HookContext` and `ToolContext` are built on the same `StateStore`, which
provides `get`, `set` and an atomic `update`. They remain **separate stores**:
sharing the type is not sharing the data, and a hook should not silently depend
on a tool's bookkeeping.

`HookContext` additionally chains to a parent — `get` walks up, `set` and
`update` stay local. `update` deliberately does not walk: a read-modify-write
that fell through to a parent would write its result locally and leave the
parent stale, which reads as a lost update.

## What `post_tool_call` receives for a built-in

`ToolResult::result` carries a structured object per tool rather than the
harness's display text: `RUN_COMMAND` reports `exit_code` and
`combined_output`, `READ_URL_CONTENT` reports `content_path` and `title`,
`EDIT_FILE` reports the `diff_block`. A hook that wanted a command's exit code
previously had to parse prose, and one that wanted a fetched page's location
could not get it at all.

Anything the SDK does not recognise still falls back to the step's text, so an
unfamiliar built-in degrades rather than disappearing.
