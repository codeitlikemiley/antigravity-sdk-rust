//! Lifecycle event hooks for the agent execution loop.
//!
//! This module defines the [`Hook`] trait, which allows implementing custom observers and middlewares
//! to intercept session startup, pre/post tool invocations, execution errors, and user interactions.

use crate::types::{AskQuestionEntry, HookResult, QuestionHookResult, ToolCall, ToolResult};
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
    /// Triggered when a tool execution fails.
    ///
    /// Returning `Some(message)` **replaces the error text** the model is
    /// shown — useful for turning a stack trace into an instruction the model
    /// can act on. Returning `None` leaves it as it is.
    ///
    /// It cannot turn a failure into a success. It used to: a hook could
    /// substitute a result and clear the error, so a tool that had failed was
    /// reported to the model as having worked, and the step was downgraded from
    /// `Error` to `Done`. Upstream narrowed this in 0.1.6 for the same reason.
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
    /// Triggered when a turn completes, receiving the model's final text.
    ///
    /// Takes the text rather than a `ChatResponse`: the dispatch happens at the
    /// terminal user-facing model step, inside the connection, where no
    /// `ChatResponse` exists yet. Building one there would have meant a second,
    /// partly-filled shape with the same name.
    fn post_turn<'a>(
        &'a self,
        _response: &'a str,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send {
        async { Ok(()) }
    }
    /// Triggered when the conversation history is compacted/summarized.
    ///
    /// Receives the compaction step itself, not just its text — a hook that
    /// archives history needs the step's index and trajectory to know what was
    /// replaced.
    fn on_compaction<'a>(
        &'a self,
        _step: &'a crate::types::Step,
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

    /// Triggered when a tool execution fails; may replace the error text.
    fn on_tool_error<'a>(
        &'a self,
        error: &'a anyhow::Error,
    ) -> BoxFuture<'a, Result<Option<String>, anyhow::Error>>;

    /// Intercepts a prompt to ask the user clarifying questions.
    fn on_interaction<'a>(
        &'a self,
        questions: &'a [AskQuestionEntry],
    ) -> BoxFuture<'a, Result<Option<QuestionHookResult>, anyhow::Error>>;

    /// Triggered when the session is ending.
    fn on_session_end(&self) -> BoxFuture<'_, Result<(), anyhow::Error>>;

    /// Triggered after a turn completes.
    fn post_turn<'a>(&'a self, response: &'a str) -> BoxFuture<'a, Result<(), anyhow::Error>>;

    /// Triggered when the conversation history is compacted.
    fn on_compaction<'a>(
        &'a self,
        step: &'a crate::types::Step,
    ) -> BoxFuture<'a, Result<(), anyhow::Error>>;
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
    ) -> BoxFuture<'a, Result<Option<String>, anyhow::Error>> {
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

    fn post_turn<'a>(&'a self, response: &'a str) -> BoxFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move { self.post_turn(response).await })
    }

    fn on_compaction<'a>(
        &'a self,
        step: &'a crate::types::Step,
    ) -> BoxFuture<'a, Result<(), anyhow::Error>> {
        Box::pin(async move { self.on_compaction(step).await })
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

    /// Decides whether a tool call may run, failing **closed**.
    ///
    /// [`dispatch_pre_tool_call`](Self::dispatch_pre_tool_call) returns an
    /// error when a hook itself fails — a panic in a policy predicate, an
    /// `ask_user` handler that could not reach the user. Both transports
    /// previously treated that error as "no objection" and ran the tool, which
    /// turns any hook bug into an open gate. A gate that cannot decide must
    /// deny.
    ///
    /// `None` means no hooks are registered at all, which is not a failure:
    /// there is nothing to object.
    pub async fn gate_tool_call(runner: Option<&Self>, tool_call: &ToolCall) -> (bool, String) {
        let Some(runner) = runner else {
            return (true, String::new());
        };
        match runner.dispatch_pre_tool_call(tool_call).await {
            Ok(res) if res.allow => (true, String::new()),
            Ok(res) => (false, res.message),
            Err(e) => {
                tracing::error!(
                    "pre_tool_call failed for {}; denying the call: {e:?}",
                    tool_call.name
                );
                (
                    false,
                    format!("the pre-tool-call gate could not decide, so the call was denied: {e}"),
                )
            }
        }
    }

    pub async fn dispatch_post_tool_call(&self, result: &ToolResult) -> Result<(), anyhow::Error> {
        let hooks = self.hooks.read().await.clone();
        for hook in &hooks {
            hook.post_tool_call(result).await?;
        }
        Ok(())
    }

    /// Gives each hook a chance to reword a tool failure.
    ///
    /// The first hook to return a replacement wins. A hook that errors is
    /// logged and skipped — one broken hook must not suppress the ones after
    /// it, and must not replace the tool's failure with its own.
    ///
    /// The failure itself always stands: this cannot clear the error.
    pub async fn dispatch_on_tool_error(&self, error: &anyhow::Error) -> Option<String> {
        let hooks = self.hooks.read().await.clone();
        for hook in &hooks {
            match hook.on_tool_error(error).await {
                Ok(Some(message)) => return Some(message),
                Ok(None) => {}
                Err(hook_err) => {
                    tracing::error!("on_tool_error hook failed: {hook_err:?}");
                }
            }
        }
        None
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
    pub async fn dispatch_post_turn(&self, response: &str) -> Result<(), anyhow::Error> {
        let hooks = self.hooks.read().await.clone();
        for hook in &hooks {
            hook.post_turn(response).await?;
        }
        Ok(())
    }

    /// Dispatches `on_compaction` to all registered hooks.
    pub async fn dispatch_on_compaction(
        &self,
        step: &crate::types::Step,
    ) -> Result<(), anyhow::Error> {
        let hooks = self.hooks.read().await.clone();
        for hook in &hooks {
            hook.on_compaction(step).await?;
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
    use crate::types::{HookResult, QuestionHookResult, ToolCall, ToolResult};
    use std::sync::Mutex;

    /// A hook whose gate cannot decide. Before S2 this ran the tool.
    struct BrokenHook;

    impl Hook for BrokenHook {
        async fn pre_tool_call(&self, _tool_call: &ToolCall) -> Result<HookResult, anyhow::Error> {
            Err(anyhow::anyhow!("the policy store is unreachable"))
        }
    }

    fn probe_call() -> ToolCall {
        ToolCall {
            id: "1".to_string(),
            name: "RUN_COMMAND".to_string(),
            args: serde_json::json!({}),
            canonical_path: None,
        }
    }

    #[tokio::test]
    async fn gate_denies_when_a_hook_errors() {
        let runner = HookRunner::new();
        runner.register(Arc::new(BrokenHook)).await;
        let (allow, reason) = HookRunner::gate_tool_call(Some(&runner), &probe_call()).await;
        assert!(!allow, "a gate that cannot decide must not allow the call");
        assert!(reason.contains("policy store is unreachable"), "{reason}");
    }

    /// No hooks registered is not a failure — there is nothing to object.
    #[tokio::test]
    async fn gate_allows_with_no_runner() {
        let (allow, reason) = HookRunner::gate_tool_call(None, &probe_call()).await;
        assert!(allow);
        assert!(reason.is_empty());
    }

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
        ) -> Result<Option<String>, anyhow::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{}:on_tool_error", self.name));
            if self.name == "recover" {
                Ok(Some("reworded".to_string()))
            } else {
                Ok(None)
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

        async fn post_turn(&self, _response: &str) -> Result<(), anyhow::Error> {
            self.calls
                .lock()
                .unwrap()
                .push(format!("{}:post_turn", self.name));
            Ok(())
        }

        async fn on_compaction(&self, _step: &crate::types::Step) -> Result<(), anyhow::Error> {
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

    /// A hook that errors must not silence the hooks registered after it, and
    /// must not replace the tool's failure with its own.
    #[tokio::test]
    async fn test_dispatch_on_tool_error_contains_a_failing_hook() {
        struct FailingHook;
        impl Hook for FailingHook {
            async fn on_tool_error(
                &self,
                _error: &anyhow::Error,
            ) -> Result<Option<String>, anyhow::Error> {
                Err(anyhow::anyhow!("hook exploded"))
            }
        }
        struct RewordingHook;
        impl Hook for RewordingHook {
            async fn on_tool_error(
                &self,
                _error: &anyhow::Error,
            ) -> Result<Option<String>, anyhow::Error> {
                Ok(Some("try a smaller page size".to_string()))
            }
        }

        let runner = HookRunner::new();
        runner.register(Arc::new(FailingHook)).await;
        runner.register(Arc::new(RewordingHook)).await;

        let replacement = runner
            .dispatch_on_tool_error(&anyhow::anyhow!("original tool failure"))
            .await;

        assert_eq!(replacement.as_deref(), Some("try a smaller page size"));
    }

    /// No hook with an opinion leaves the tool's own message standing.
    #[tokio::test]
    async fn test_dispatch_on_tool_error_defaults_to_no_replacement() {
        let runner = HookRunner::new();
        assert!(
            runner
                .dispatch_on_tool_error(&anyhow::anyhow!("boom"))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_dispatch_on_tool_error_first_replacement_wins() {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let runner = HookRunner::new();
        for name in ["h1", "recover", "h2"] {
            runner
                .register(Arc::new(TrackerHook {
                    name: name.to_string(),
                    calls: calls.clone(),
                }))
                .await;
        }

        let err = anyhow::anyhow!("error occurred");
        assert_eq!(
            runner.dispatch_on_tool_error(&err).await.as_deref(),
            Some("reworded")
        );

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

        runner.dispatch_post_turn("hello").await.unwrap();

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

        let step = crate::types::Step {
            r#type: crate::types::StepType::Compaction,
            content: "summary text".to_string(),
            ..Default::default()
        };
        runner.dispatch_on_compaction(&step).await.unwrap();

        let recorded = calls.lock().unwrap().clone();
        assert_eq!(recorded, vec!["h1:on_compaction", "h2:on_compaction"]);
    }
}
