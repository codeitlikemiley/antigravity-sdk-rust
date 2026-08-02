# Error Handling & Lifecycle Hooks

This guide explains how errors are handled, propagated, and intercepted within the Google Antigravity Rust SDK.

## Error Propagation

API functions (e.g. `Agent::start()`, `Agent::chat()`, `Conversation::chat_to_completion()`) return standard Rust `Result<T, anyhow::Error>` types. 

Critical errors that stop execution:
* **Harness Handshake Failure**: Subprocess terminates before stdin/stdout exchange finishes, or Port information is invalid.
* **WebSocket Disconnection**: Network or websocket client errors during an active stream.
* **Platform Errors**: Server-side credentials invalid, rate limits exceeded (mapped from HTTP status codes inside `StepUpdate` streams).

---

## Intercepting Errors & Lifecycle Events: The Hook Trait

The SDK provides a thread-safe lifecycle hook mechanism. Registering structures implementing the `Hook` trait allows you to audit execution, print diagnostic traces, or handle tool failures.

### The Hook Trait Definition

```rust
use antigravity_sdk_rust::types::{AskQuestionEntry, HookResult, QuestionHookResult, ToolCall, ToolResult};

pub trait Hook: Send + Sync {
    /// Triggered when the agent establishes a connection and starts a session.
    fn on_session_start<'a>(
        &'a self,
        _context: &'a HookContext,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send {
        async { Ok(()) }
    }

    /// Intercepts the start of a user turn before the LLM processes the prompt.
    /// Returns `allow: false` to halt execution.
    fn pre_turn(
        &self,
    ) -> impl std::future::Future<Output = Result<HookResult, anyhow::Error>> + Send {
        async {
            Ok(HookResult {
                allow: true,
                message: String::new(),
            })
        }
    }

    /// Intercepts a tool call immediately before it is executed by the runner.
    /// Returns `allow: false` to prevent execution.
    fn pre_tool_call<'a>(
        &'a self,
        _tool_call: &'a ToolCall,
        context: &'a HookContext,
    ) -> impl std::future::Future<Output = Result<HookResult, anyhow::Error>> + Send {
        async {
            Ok(HookResult {
                allow: true,
                message: String::new(),
            })
        }
    }

    /// Triggered after a tool successfully returns a result.
    fn post_tool_call<'a>(
        &'a self,
        _result: &'a ToolResult,
        context: &'a HookContext,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send {
        async { Ok(()) }
    }

    /// Triggered when a tool execution fails; may reword the error.
    fn on_tool_error<'a>(
        &'a self,
        _error: &'a anyhow::Error,
        context: &'a HookContext,
    ) -> impl std::future::Future<Output = Result<Option<String>, anyhow::Error>> + Send {
        // `Some(message)` replaces the error text shown to the model;
        // `None` leaves it. A failure cannot be turned into a success.
        async { Ok(None) }
    }

    /// Intercepts a prompt to ask the user clarifying questions.
    fn on_interaction<'a>(
        &'a self,
        _questions: &'a [AskQuestionEntry],
    ) -> impl std::future::Future<Output = Result<Option<QuestionHookResult>, anyhow::Error>> + Send
    {
        async { Ok(None) }
    }
}
```

---

## Implementing a Diagnostic Logging Hook

Below is an implementation of a custom monitoring hook that logs tool execution results and captures error conditions:

```rust
use antigravity_sdk_rust::agent::{Agent, AgentConfig};
use antigravity_sdk_rust::hooks::Hook;
use antigravity_sdk_rust::types::{ToolCall, ToolResult, HookResult};
use std::sync::Arc;

struct DiagnosticLogger;

impl Hook for DiagnosticLogger {
    fn pre_tool_call<'a>(&'a self, tool_call: &'a ToolCall) -> impl std::future::Future<Output = Result<HookResult, anyhow::Error>> + Send {
        async move {
            println!("[HOOK] Executing Tool: {} with args: {}", tool_call.name, tool_call.args);
            Ok(HookResult {
                allow: true,
                message: String::new(),
            })
        }
    }

    fn post_tool_call<'a>(&'a self, result: &'a ToolResult) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send {
        async move {
            println!("[HOOK] Tool Complete: {} - Result: {:?}", result.name, result.result);
            Ok(())
        }
    }

    fn on_tool_error<'a>(
        &'a self,
        error: &'a anyhow::Error,
        context: &'a HookContext,
    ) -> impl std::future::Future<Output = Result<Option<String>, anyhow::Error>> + Send {
        async move {
            eprintln!("[HOOK ERROR] Tool failed: {}", error);
            // Telemetry, or reword the failure into something the model can act
            // on. The tool still failed either way.
            Ok(None)
        }
    }
}

// Registering the hook in AgentConfig
let mut config = AgentConfig::default();
config.hooks.push(Arc::new(DiagnosticLogger));
```
