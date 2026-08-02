# Lifecycle Hooks

Hooks in the Google Antigravity Rust SDK allow you to intercept, monitor, and override various lifecycle events of the agent session.

## Hook Trait Definition

To create custom hooks, implement the `Hook` trait:

```rust
use antigravity_sdk_rust::hooks::Hook;
use antigravity_sdk_rust::types::{AskQuestionEntry, HookResult, QuestionHookResult, ToolCall, ToolResult};

pub trait Hook: Send + Sync {
    /// Triggered when the agent establishes a connection and starts a session.
    fn on_session_start(&self) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send {
        async { Ok(()) }
    }

    /// Intercepts the start of a user turn before the LLM processes the prompt.
    /// Returns `allow: false` to halt execution.
    fn pre_turn(&self) -> impl std::future::Future<Output = Result<HookResult, anyhow::Error>> + Send {
        async {
            Ok(HookResult {
                allow: true,
                message: String::new(),
            })
        }
    }

    /// Intercepts a tool call immediately before it is executed by the runner.
    /// Returns `allow: false` to prevent execution.
    fn pre_tool_call<'a>(&'a self, _tool_call: &'a ToolCall) -> impl std::future::Future<Output = Result<HookResult, anyhow::Error>> + Send {
        async {
            Ok(HookResult {
                allow: true,
                message: String::new(),
            })
        }
    }

    /// Triggered after a tool successfully returns a result.
    fn post_tool_call<'a>(&'a self, _result: &'a ToolResult) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send {
        async { Ok(()) }
    }

    /// Triggered when a tool execution fails.
    /// `Some(message)` replaces the error text the model is shown; `None`
    /// leaves it. A failure cannot be turned into a success.
    fn on_tool_error<'a>(
        &'a self,
        _error: &'a anyhow::Error,
    ) -> impl std::future::Future<Output = Result<Option<String>, anyhow::Error>> + Send {
        async { Ok(None) }
    }

    /// Intercepts a prompt to ask the user clarifying questions.
    fn on_interaction<'a>(
        &'a self,
        _questions: &'a [AskQuestionEntry],
    ) -> impl std::future::Future<Output = Result<Option<QuestionHookResult>, anyhow::Error>> + Send {
        async { Ok(None) }
    }
}
```

---

## Code Example

Below is an example showing how to declare and register a custom logging Hook inside `AgentConfig`:

```rust
use antigravity_sdk_rust::agent::{Agent, AgentConfig};
use antigravity_sdk_rust::hooks::Hook;
use antigravity_sdk_rust::policy;
use antigravity_sdk_rust::types::{HookResult, ToolCall, ToolResult};
use std::sync::Arc;

// 1. Define custom hook struct
struct LoggerHook;

impl Hook for LoggerHook {
    fn on_session_start(&self) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send {
        async {
            println!("[Hook] Session has successfully started!");
            Ok(())
        }
    }

    fn pre_turn(&self) -> impl std::future::Future<Output = Result<HookResult, anyhow::Error>> + Send {
        async {
            println!("[Hook] Preparing next turn...");
            Ok(HookResult {
                allow: true,
                message: String::new(),
            })
        }
    }

    fn pre_tool_call<'a>(&'a self, tool_call: &'a ToolCall) -> impl std::future::Future<Output = Result<HookResult, anyhow::Error>> + Send {
        async move {
            println!("[Hook] Inspecting tool call request: {}", tool_call.name);
            Ok(HookResult {
                allow: true,
                message: String::new(),
            })
        }
    }

    fn post_tool_call<'a>(&'a self, result: &'a ToolResult) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send {
        async move {
            println!("[Hook] Tool call completed with result: {:?}", result.result);
            Ok(())
        }
    }
}

// 2. Register hook inside configuration
#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let mut config = AgentConfig::default();
    config.policies = Some(vec![policy::allow_all()]);

    // Register our hook
    config.hooks.push(Arc::new(LoggerHook));

    let mut agent = Agent::new(config);
    agent.start().await?;

    let response = agent.chat("What files are in the current workspace?").await?;
    println!("Agent response: {}", response.text);

    agent.stop().await?;
    Ok(())
}
```
