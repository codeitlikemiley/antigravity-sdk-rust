//! Lifecycle event hooks for the agent execution loop.
//!
//! This module defines the [`Hook`] trait, which allows implementing custom observers and middlewares
//! to intercept session startup, pre/post tool invocations, execution errors, and user interactions.

use crate::types::{
    AskQuestionEntry, ChatResponse, HookResult, QuestionHookResult, ToolCall, ToolResult,
};
use futures_util::future::BoxFuture;
use std::sync::Arc;

/// Trait representing an active interceptor of agent lifecycle events.
///
/// Implementors can register hooks via [`Agent::register_hook`](crate::agent::Agent::register_hook)
/// to audit tool invocations, log events, or restrict actions dynamically.
pub trait Hook: Send + Sync {
    /// Triggered when the agent establishes a connection and starts a session.
    fn on_session_start(
        &self,
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
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send {
        async { Ok(()) }
    }
    /// Triggered when a tool execution encounters an error.
    /// Allows fallback logic or customized error payloads.
    fn on_tool_error<'a>(
        &'a self,
        error: &'a anyhow::Error,
    ) -> impl std::future::Future<
        Output = Result<(HookResult, Option<serde_json::Value>), anyhow::Error>,
    > + Send {
        async move {
            Ok((
                HookResult {
                    allow: false,
                    message: error.to_string(),
                },
                None,
            ))
        }
    }
    /// Intercepts a prompt to ask the user clarifying questions.
    fn on_interaction<'a>(
        &'a self,
        _questions: &'a [AskQuestionEntry],
    ) -> impl std::future::Future<Output = Result<Option<QuestionHookResult>, anyhow::Error>> + Send
    {
        async { Ok(None) }
    }
    /// Triggered when the session is ending (agent shutdown or disconnect).
    fn on_session_end(
        &self,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send {
        async { Ok(()) }
    }
    /// Triggered after a turn completes, receiving the full response.
    fn post_turn<'a>(
        &'a self,
        _response: &'a ChatResponse,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send {
        async { Ok(()) }
    }
    /// Triggered when the conversation history is compacted/summarized.
    fn on_compaction<'a>(
        &'a self,
        _summary: &'a str,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send {
        async { Ok(()) }
    }
}

/// Object-safe version of the [`Hook`] trait, automatically implemented via a blanket impl.
///
/// This trait is used internally by the SDK to allow dynamic dispatch and storage of hooks.
pub trait DynHook: Send + Sync {
    /// Triggered when the agent establishes a connection and starts a session.
    fn on_session_start(&self) -> BoxFuture<'_, Result<(), anyhow::Error>>;

    /// Intercepts the start of a user turn before the LLM processes the prompt.
    fn pre_turn(&self) -> BoxFuture<'_, Result<HookResult, anyhow::Error>>;

    /// Intercepts a tool call immediately before it is executed by the runner.
    fn pre_tool_call<'a>(
        &'a self,
        tool_call: &'a ToolCall,
    ) -> BoxFuture<'a, Result<HookResult, anyhow::Error>>;

    /// Triggered after a tool successfully returns a result.
    fn post_tool_call<'a>(
        &'a self,
        result: &'a ToolResult,
    ) -> BoxFuture<'a, Result<(), anyhow::Error>>;

    /// Triggered when a tool execution encounters an error.
    fn on_tool_error<'a>(
        &'a self,
        error: &'a anyhow::Error,
    ) -> BoxFuture<'a, Result<(HookResult, Option<serde_json::Value>), anyhow::Error>>;

    /// Intercepts a prompt to ask the user clarifying questions.
    fn on_interaction<'a>(
        &'a self,
        questions: &'a [AskQuestionEntry],
    ) -> BoxFuture<'a, Result<Option<QuestionHookResult>, anyhow::Error>>;

    /// Triggered when the session is ending.
    fn on_session_end(&self) -> BoxFuture<'_, Result<(), anyhow::Error>>;

    /// Triggered after a turn completes.
    fn post_turn<'a>(
        &'a self,
        response: &'a ChatResponse,
    ) -> BoxFuture<'a, Result<(), anyhow::Error>>;

    /// Triggered when the conversation history is compacted.
    fn on_compaction<'a>(&'a self, summary: &'a str) -> BoxFuture<'a, Result<(), anyhow::Error>>;
}

impl<T: Hook + ?Sized> DynHook for T {
    fn on_session_start(&self) -> BoxFuture<'_, Result<(), anyhow::Error>> {
        Box::pin(async move { self.on_session_start().await })
    }

    fn pre_turn(&self) -> BoxFuture<'_, Result<HookResult, anyhow::Error>> {
        Box::pin(async move { self.pre_turn().await })
    }

    fn pre_tool_call<'a>(
        &'a self,
        tool_call: &'a ToolCall,
    ) -> BoxFuture<'a, Result<HookResult, anyhow::Error>> {
        Box::pin(async move { self.pre_tool_call(tool_call).await })
    }

    fn post_tool_call<'a>(
        &'a self,
        result: &'a ToolResult,
    ) -> BoxFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move { self.post_tool_call(result).await })
    }

    fn on_tool_error<'a>(
        &'a self,
        error: &'a anyhow::Error,
    ) -> BoxFuture<'a, Result<(HookResult, Option<serde_json::Value>), anyhow::Error>> {
        Box::pin(async move { self.on_tool_error(error).await })
    }

    fn on_interaction<'a>(
        &'a self,
        questions: &'a [AskQuestionEntry],
    ) -> BoxFuture<'a, Result<Option<QuestionHookResult>, anyhow::Error>> {
        Box::pin(async move { self.on_interaction(questions).await })
    }

    fn on_session_end(&self) -> BoxFuture<'_, Result<(), anyhow::Error>> {
        Box::pin(async move { self.on_session_end().await })
    }

    fn post_turn<'a>(
        &'a self,
        response: &'a ChatResponse,
    ) -> BoxFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move { self.post_turn(response).await })
    }

    fn on_compaction<'a>(&'a self, summary: &'a str) -> BoxFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move { self.on_compaction(summary).await })
    }
}

/// Internal helper that manages a collection of registered [`Hook`]s and dispatches events sequentially.
#[derive(Clone, Default)]
pub struct HookRunner {
    hooks: Arc<tokio::sync::RwLock<Vec<Arc<dyn DynHook>>>>,
}

impl std::fmt::Debug for HookRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HookRunner")
            .field("hooks_count", &self.hooks.try_read().map_or(0, |h| h.len()))
            .finish()
    }
}

impl HookRunner {
    /// Creates a new, empty `HookRunner`.
    pub fn new() -> Self {
        Self {
            hooks: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    pub async fn register(&self, hook: Arc<dyn DynHook>) {
        self.hooks.write().await.push(hook);
    }

    pub async fn dispatch_session_start(&self) -> Result<(), anyhow::Error> {
        let hooks = self.hooks.read().await.clone();
        for hook in &hooks {
            hook.on_session_start().await?;
        }
        Ok(())
    }

    pub async fn dispatch_pre_turn(&self) -> Result<HookResult, anyhow::Error> {
        let hooks = self.hooks.read().await.clone();
        for hook in &hooks {
            let res = hook.pre_turn().await?;
            if !res.allow {
                return Ok(res);
            }
        }
        Ok(HookResult {
            allow: true,
            message: String::new(),
        })
    }

    pub async fn dispatch_pre_tool_call(
        &self,
        tool_call: &ToolCall,
    ) -> Result<HookResult, anyhow::Error> {
        let hooks = self.hooks.read().await.clone();
        for hook in &hooks {
            let res = hook.pre_tool_call(tool_call).await?;
            if !res.allow {
                return Ok(res);
            }
        }
        Ok(HookResult {
            allow: true,
            message: String::new(),
        })
    }

    pub async fn dispatch_post_tool_call(&self, result: &ToolResult) -> Result<(), anyhow::Error> {
        let hooks = self.hooks.read().await.clone();
        for hook in &hooks {
            hook.post_tool_call(result).await?;
        }
        Ok(())
    }

    pub async fn dispatch_on_tool_error(
        &self,
        error: &anyhow::Error,
    ) -> Result<(HookResult, Option<serde_json::Value>), anyhow::Error> {
        let hooks = self.hooks.read().await.clone();
        for hook in &hooks {
            let (res, val) = hook.on_tool_error(error).await?;
            if res.allow {
                return Ok((res, val));
            }
        }
        Ok((
            HookResult {
                allow: false,
                message: error.to_string(),
            },
            None,
        ))
    }

    pub async fn dispatch_interaction(
        &self,
        questions: &[AskQuestionEntry],
    ) -> Result<Option<QuestionHookResult>, anyhow::Error> {
        let hooks = self.hooks.read().await.clone();
        for hook in &hooks {
            if let Some(res) = hook.on_interaction(questions).await? {
                return Ok(Some(res));
            }
        }
        Ok(None)
    }

    /// Dispatches `on_session_end` to all registered hooks.
    pub async fn dispatch_session_end(&self) -> Result<(), anyhow::Error> {
        let hooks = self.hooks.read().await.clone();
        for hook in &hooks {
            hook.on_session_end().await?;
        }
        Ok(())
    }

    /// Dispatches `post_turn` to all registered hooks.
    pub async fn dispatch_post_turn(&self, response: &ChatResponse) -> Result<(), anyhow::Error> {
        let hooks = self.hooks.read().await.clone();
        for hook in &hooks {
            hook.post_turn(response).await?;
        }
        Ok(())
    }

    /// Dispatches `on_compaction` to all registered hooks.
    pub async fn dispatch_on_compaction(&self, summary: &str) -> Result<(), anyhow::Error> {
        let hooks = self.hooks.read().await.clone();
        for hook in &hooks {
            hook.on_compaction(summary).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::field_reassign_with_default
    )]
    use super::*;
    use crate::types::{HookResult, QuestionHookResult, ToolCall, ToolResult, UsageMetadata};
    use std::sync::Mutex;

    struct TrackerHook {
        name: String,
        calls: Arc<Mutex<Vec<String>>>,
    }

    impl Hook for TrackerHook {
        async fn on_session_start(&self) -> Result<(), anyhow::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{}:session_start", self.name));
            Ok(())
        }

        async fn pre_turn(&self) -> Result<HookResult, anyhow::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{}:pre_turn", self.name));
            if self.name == "deny" {
                Ok(HookResult {
                    allow: false,
                    message: "denied".to_string(),
                })
            } else {
                Ok(HookResult {
                    allow: true,
                    message: String::new(),
                })
            }
        }

        async fn pre_tool_call(&self, _tool_call: &ToolCall) -> Result<HookResult, anyhow::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{}:pre_tool_call", self.name));
            if self.name == "deny" {
                Ok(HookResult {
                    allow: false,
                    message: "denied".to_string(),
                })
            } else {
                Ok(HookResult {
                    allow: true,
                    message: String::new(),
                })
            }
        }

        async fn post_tool_call(&self, _result: &ToolResult) -> Result<(), anyhow::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{}:post_tool_call", self.name));
            Ok(())
        }

        async fn on_tool_error(
            &self,
            _error: &anyhow::Error,
        ) -> Result<(HookResult, Option<serde_json::Value>), anyhow::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{}:on_tool_error", self.name));
            if self.name == "recover" {
                Ok((
                    HookResult {
                        allow: true,
                        message: "recovered".to_string(),
                    },
                    Some(serde_json::json!({"recovered": true})),
                ))
            } else {
                Ok((
                    HookResult {
                        allow: false,
                        message: "not recovered".to_string(),
                    },
                    None,
                ))
            }
        }

        async fn on_interaction(
            &self,
            _questions: &[AskQuestionEntry],
        ) -> Result<Option<QuestionHookResult>, anyhow::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{}:on_interaction", self.name));
            if self.name == "answer" {
                Ok(Some(QuestionHookResult {
                    responses: vec![],
                    cancelled: false,
                }))
            } else {
                Ok(None)
            }
        }

        async fn on_session_end(&self) -> Result<(), anyhow::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{}:session_end", self.name));
            Ok(())
        }

        async fn post_turn(&self, _response: &ChatResponse) -> Result<(), anyhow::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{}:post_turn", self.name));
            Ok(())
        }

        async fn on_compaction(&self, _summary: &str) -> Result<(), anyhow::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{}:on_compaction", self.name));
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_dispatch_session_start() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = HookRunner::new();
        runner
            .register(Arc::new(TrackerHook {
                name: "h1".to_string(),
                calls: calls.clone(),
            }))
            .await;
        runner
            .register(Arc::new(TrackerHook {
                name: "h2".to_string(),
                calls: calls.clone(),
            }))
            .await;

        runner.dispatch_session_start().await.unwrap();

        let recorded = calls.lock().unwrap().clone();
        assert_eq!(recorded, vec!["h1:session_start", "h2:session_start"]);
    }

    #[tokio::test]
    async fn test_dispatch_pre_turn_allow() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = HookRunner::new();
        runner
            .register(Arc::new(TrackerHook {
                name: "h1".to_string(),
                calls: calls.clone(),
            }))
            .await;
        runner
            .register(Arc::new(TrackerHook {
                name: "h2".to_string(),
                calls: calls.clone(),
            }))
            .await;

        let res = runner.dispatch_pre_turn().await.unwrap();
        assert!(res.allow);

        let recorded = calls.lock().unwrap().clone();
        assert_eq!(recorded, vec!["h1:pre_turn", "h2:pre_turn"]);
    }

    #[tokio::test]
    async fn test_dispatch_pre_turn_deny_short_circuits() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = HookRunner::new();
        runner
            .register(Arc::new(TrackerHook {
                name: "h1".to_string(),
                calls: calls.clone(),
            }))
            .await;
        runner
            .register(Arc::new(TrackerHook {
                name: "deny".to_string(),
                calls: calls.clone(),
            }))
            .await;
        runner
            .register(Arc::new(TrackerHook {
                name: "h2".to_string(),
                calls: calls.clone(),
            }))
            .await;

        let res = runner.dispatch_pre_turn().await.unwrap();
        assert!(!res.allow);
        assert_eq!(res.message, "denied");

        let recorded = calls.lock().unwrap().clone();
        assert_eq!(recorded, vec!["h1:pre_turn", "deny:pre_turn"]);
    }

    #[tokio::test]
    async fn test_dispatch_pre_tool_call_deny_short_circuits() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = HookRunner::new();
        runner
            .register(Arc::new(TrackerHook {
                name: "h1".to_string(),
                calls: calls.clone(),
            }))
            .await;
        runner
            .register(Arc::new(TrackerHook {
                name: "deny".to_string(),
                calls: calls.clone(),
            }))
            .await;
        runner
            .register(Arc::new(TrackerHook {
                name: "h2".to_string(),
                calls: calls.clone(),
            }))
            .await;

        let tool_call = ToolCall {
            id: "call_1".to_string(),
            name: "tool_1".to_string(),
            args: serde_json::Value::Null,
            canonical_path: None,
        };
        let res = runner.dispatch_pre_tool_call(&tool_call).await.unwrap();
        assert!(!res.allow);
        assert_eq!(res.message, "denied");

        let recorded = calls.lock().unwrap().clone();
        assert_eq!(recorded, vec!["h1:pre_tool_call", "deny:pre_tool_call"]);
    }

    #[tokio::test]
    async fn test_dispatch_post_tool_call() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = HookRunner::new();
        runner
            .register(Arc::new(TrackerHook {
                name: "h1".to_string(),
                calls: calls.clone(),
            }))
            .await;

        let res = ToolResult {
            name: "tool_1".to_string(),
            id: Some("call_1".to_string()),
            result: Some(serde_json::Value::Null),
            error: None,
        };
        runner.dispatch_post_tool_call(&res).await.unwrap();

        let recorded = calls.lock().unwrap().clone();
        assert_eq!(recorded, vec!["h1:post_tool_call"]);
    }

    #[tokio::test]
    async fn test_dispatch_on_tool_error_recovery_short_circuits() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = HookRunner::new();
        runner
            .register(Arc::new(TrackerHook {
                name: "h1".to_string(),
                calls: calls.clone(),
            }))
            .await;
        runner
            .register(Arc::new(TrackerHook {
                name: "recover".to_string(),
                calls: calls.clone(),
            }))
            .await;
        runner
            .register(Arc::new(TrackerHook {
                name: "h2".to_string(),
                calls: calls.clone(),
            }))
            .await;

        let err = anyhow::anyhow!("error occurred");
        let (res, val) = runner.dispatch_on_tool_error(&err).await.unwrap();
        assert!(res.allow);
        assert_eq!(res.message, "recovered");
        assert_eq!(val.unwrap(), serde_json::json!({"recovered": true}));

        let recorded = calls.lock().unwrap().clone();
        assert_eq!(recorded, vec!["h1:on_tool_error", "recover:on_tool_error"]);
    }

    #[tokio::test]
    async fn test_dispatch_interaction_short_circuits() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = HookRunner::new();
        runner
            .register(Arc::new(TrackerHook {
                name: "h1".to_string(),
                calls: calls.clone(),
            }))
            .await;
        runner
            .register(Arc::new(TrackerHook {
                name: "answer".to_string(),
                calls: calls.clone(),
            }))
            .await;
        runner
            .register(Arc::new(TrackerHook {
                name: "h2".to_string(),
                calls: calls.clone(),
            }))
            .await;

        let entries = vec![];
        let res = runner.dispatch_interaction(&entries).await.unwrap();
        assert!(res.is_some());

        let recorded = calls.lock().unwrap().clone();
        assert_eq!(recorded, vec!["h1:on_interaction", "answer:on_interaction"]);
    }

    #[tokio::test]
    async fn test_dispatch_session_end() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = HookRunner::new();
        runner
            .register(Arc::new(TrackerHook {
                name: "h1".to_string(),
                calls: calls.clone(),
            }))
            .await;
        runner
            .register(Arc::new(TrackerHook {
                name: "h2".to_string(),
                calls: calls.clone(),
            }))
            .await;

        runner.dispatch_session_end().await.unwrap();

        let recorded = calls.lock().unwrap().clone();
        assert_eq!(recorded, vec!["h1:session_end", "h2:session_end"]);
    }

    #[tokio::test]
    async fn test_dispatch_post_turn() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = HookRunner::new();
        runner
            .register(Arc::new(TrackerHook {
                name: "h1".to_string(),
                calls: calls.clone(),
            }))
            .await;

        let response = ChatResponse {
            text: "hello".to_string(),
            thinking: String::new(),
            steps: vec![],
            usage_metadata: UsageMetadata::default(),
        };
        runner.dispatch_post_turn(&response).await.unwrap();

        let recorded = calls.lock().unwrap().clone();
        assert_eq!(recorded, vec!["h1:post_turn"]);
    }

    #[tokio::test]
    async fn test_dispatch_on_compaction() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = HookRunner::new();
        runner
            .register(Arc::new(TrackerHook {
                name: "h1".to_string(),
                calls: calls.clone(),
            }))
            .await;
        runner
            .register(Arc::new(TrackerHook {
                name: "h2".to_string(),
                calls: calls.clone(),
            }))
            .await;

        runner.dispatch_on_compaction("summary text").await.unwrap();

        let recorded = calls.lock().unwrap().clone();
        assert_eq!(recorded, vec!["h1:on_compaction", "h2:on_compaction"]);
    }
}
