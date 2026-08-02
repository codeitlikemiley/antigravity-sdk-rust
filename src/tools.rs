//! Custom client-side tool definition and execution.
//!
//! This module defines the [`Tool`] trait, allowing custom functionality (e.g. database access,
//! API requests) to be registered with the agent and executed when requested by the model.
//! Registration and concurrent execution is managed via [`ToolRunner`].

use crate::tool_context::ToolContext;
use futures_util::future::BoxFuture;
use serde_json::Value;
use std::sync::Arc;

/// A trait defining custom tool behaviors that can be invoked by the model.
pub trait Tool: Send + Sync {
    /// Returns the unique name of the tool, matching what the model will call.
    fn name(&self) -> &str;

    /// Returns a description of the tool to help the model understand when to use it.
    fn description(&self) -> &str;

    /// Returns a JSON schema describing the expected parameters of the tool.
    fn parameters_json_schema(&self) -> &str;

    /// Executes the tool with the given JSON arguments.
    ///
    /// # Errors
    ///
    /// Returns an error if argument validation or tool execution fails.
    fn call(
        &self,
        args: Value,
    ) -> impl std::future::Future<Output = Result<Value, anyhow::Error>> + Send;

    /// Returns `true` if this tool requires a [`ToolContext`] to be injected.
    /// Defaults to `false` for backwards compatibility.
    fn needs_context(&self) -> bool {
        false
    }

    /// Executes the tool with access to the session-scoped [`ToolContext`].
    ///
    /// Only called when [`needs_context()`](Tool::needs_context) returns `true`.
    /// The default implementation ignores the context and delegates to [`call()`](Tool::call).
    fn call_with_context(
        &self,
        args: Value,
        _context: &ToolContext,
    ) -> impl std::future::Future<Output = Result<Value, anyhow::Error>> + Send {
        self.call(args)
    }
}

/// Object-safe version of the [`Tool`] trait, automatically implemented via a blanket impl.
///
/// This trait is used internally by the SDK to allow dynamic dispatch and storage of tools.
pub trait DynTool: Send + Sync {
    /// Returns the unique name of the tool, matching what the model will call.
    fn name(&self) -> &str;

    /// Returns a description of the tool to help the model understand when to use it.
    fn description(&self) -> &str;

    /// Returns a JSON schema describing the expected parameters of the tool.
    fn parameters_json_schema(&self) -> &str;

    /// Executes the tool with the given JSON arguments.
    fn call(&self, args: Value) -> BoxFuture<'_, Result<Value, anyhow::Error>>;

    /// Returns `true` if this tool requires a `ToolContext`.
    fn needs_context(&self) -> bool;

    /// Executes the tool with a `ToolContext`.
    fn call_with_context<'a>(
        &'a self,
        args: Value,
        context: &'a ToolContext,
    ) -> BoxFuture<'a, Result<Value, anyhow::Error>>;
}

impl<T: Tool + ?Sized> DynTool for T {
    fn name(&self) -> &str {
        self.name()
    }

    fn description(&self) -> &str {
        self.description()
    }

    fn parameters_json_schema(&self) -> &str {
        self.parameters_json_schema()
    }

    fn call(&self, args: Value) -> BoxFuture<'_, Result<Value, anyhow::Error>> {
        Box::pin(async move { self.call(args).await })
    }

    fn needs_context(&self) -> bool {
        Tool::needs_context(self)
    }

    fn call_with_context<'a>(
        &'a self,
        args: Value,
        context: &'a ToolContext,
    ) -> BoxFuture<'a, Result<Value, anyhow::Error>> {
        Box::pin(async move { self.call_with_context(args, context).await })
    }
}

/// Registry and concurrent runner for custom tool implementations.
#[derive(Clone, Default)]
pub struct ToolRunner {
    /// The session context handed to tools that ask for one.
    ///
    /// Set once by `Agent::start`, after the connection exists. Nothing set it
    /// before, so `needs_context()` tools silently ran through the plain
    /// `call()` path and context-aware tools did not work at all (audit T1).
    context: Arc<tokio::sync::RwLock<Option<Arc<ToolContext>>>>,
    /// Active tools, in registration order.
    ///
    /// A `HashMap` before, which meant the tool list sent to the harness came
    /// out in a different order on every run — the model's tool list is part of
    /// its prompt, so that was gratuitous nondeterminism. Insertion order also
    /// matches upstream, whose registry is a Python dict.
    pub tools: Arc<tokio::sync::RwLock<Vec<Arc<dyn DynTool>>>>,
}

impl std::fmt::Debug for ToolRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRunner")
            .field("tools_count", &self.tools.try_read().map_or(0, |t| t.len()))
            .finish_non_exhaustive()
    }
}

impl ToolRunner {
    /// Creates a new, empty `ToolRunner`.
    pub fn new() -> Self {
        Self {
            context: Arc::new(tokio::sync::RwLock::new(None)),
            tools: Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    /// Attaches the session context handed to tools that request one.
    pub async fn set_context(&self, context: Arc<ToolContext>) {
        *self.context.write().await = Some(context);
    }

    /// Registers a custom tool implementation.
    ///
    /// # Errors
    ///
    /// Returns an error if a tool with the same name is already registered.
    /// Silently replacing it — the old behaviour — meant a name collision
    /// between two modules' tools resolved to whichever registered last, and
    /// the model called something the caller had not intended to expose.
    pub async fn register(&self, tool: Arc<dyn DynTool>) -> Result<(), anyhow::Error> {
        let mut tools = self.tools.write().await;
        let duplicate = tools.iter().any(|t| t.name() == tool.name());
        if duplicate {
            drop(tools);
            return Err(anyhow::anyhow!(
                "a tool named `{}` is already registered",
                tool.name()
            ));
        }
        tools.push(tool);
        drop(tools);
        Ok(())
    }

    /// Looks a tool up by the name the model called.
    pub async fn get(&self, name: &str) -> Option<Arc<dyn DynTool>> {
        self.tools
            .read()
            .await
            .iter()
            .find(|t| t.name() == name)
            .map(Arc::clone)
    }

    /// Executes a list of tool invocations, mapping their outputs to [`ToolResult`](crate::types::ToolResult)s.
    ///
    /// The batch runs concurrently — a model that asks for three independent
    /// lookups waits for the slowest, not the sum. The registry lock is
    /// released before any tool body runs, so a tool that registers another
    /// tool cannot deadlock the batch it is part of.
    pub async fn process_tool_calls(
        &self,
        calls: Vec<crate::types::ToolCall>,
    ) -> Vec<crate::types::ToolResult> {
        let resolved: Vec<(crate::types::ToolCall, Option<Arc<dyn DynTool>>)> = {
            let tools = self.tools.read().await;
            calls
                .into_iter()
                .map(|call| {
                    let tool = tools.iter().find(|t| t.name() == call.name).map(Arc::clone);
                    (call, tool)
                })
                .collect()
        };

        let context = self.context.read().await.clone();

        futures_util::future::join_all(resolved.into_iter().map(|(call, tool)| {
            let context = context.clone();
            async move {
                let Some(tool) = tool else {
                    return crate::types::ToolResult {
                        id: Some(call.id),
                        name: call.name.clone(),
                        result: None,
                        error: Some(format!("Tool {} not found", call.name)),
                        server_name: None,
                        exception: None,
                    };
                };
                // The model's arguments are shaped to the tool's own schema
                // before it sees them, so `"3"` for an integer is not reported
                // to the model as the tool being broken.
                let call_args = crate::coerce::coerced(call.args, tool.parameters_json_schema());
                let outcome = match (tool.needs_context(), context.as_ref()) {
                    (true, Some(ctx)) => tool.call_with_context(call_args, ctx).await,
                    // A tool that asks for a context and finds none would
                    // otherwise run without it and behave subtly differently.
                    (true, None) => Err(anyhow::anyhow!(
                        "`{}` requires a ToolContext and none is attached",
                        call.name
                    )),
                    (false, _) => tool.call(call_args).await,
                };
                match outcome {
                    Ok(val) => crate::types::ToolResult {
                        id: Some(call.id),
                        server_name: call.server_name,
                        name: call.name,
                        result: Some(val),
                        error: None,
                        exception: None,
                    },
                    Err(e) => crate::types::ToolResult {
                        id: Some(call.id),
                        // The message the model sees, plus the same failure in
                        // structured form so a hook can route or count it
                        // without parsing prose.
                        error: Some(e.to_string()),
                        exception: Some(crate::error::ToolExecutionError {
                            message: e.to_string(),
                            tool_name: call.name.clone(),
                            server_name: call.server_name.clone(),
                        }),
                        server_name: call.server_name,
                        name: call.name,
                        result: None,
                    },
                }
            }
        }))
        .await
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::{Tool, ToolRunner};
    use serde_json::Value;
    use std::sync::Arc;

    struct Echo {
        name: &'static str,
        delay_ms: u64,
    }

    impl Tool for Echo {
        fn name(&self) -> &str {
            self.name
        }
        fn description(&self) -> &'static str {
            "echoes"
        }
        fn parameters_json_schema(&self) -> &'static str {
            "{}"
        }
        async fn call(&self, args: Value) -> Result<Value, anyhow::Error> {
            tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;
            Ok(args)
        }
    }

    fn echo(name: &'static str) -> Arc<Echo> {
        Arc::new(Echo { name, delay_ms: 0 })
    }

    #[tokio::test]
    async fn a_duplicate_name_is_rejected() {
        let runner = ToolRunner::new();
        runner.register(echo("lookup")).await.unwrap();
        let err = runner
            .register(echo("lookup"))
            .await
            .expect_err("the second registration must be refused");
        assert!(err.to_string().contains("already registered"), "{err}");
        assert_eq!(runner.tools.read().await.len(), 1);
    }

    /// The tool list is part of the model's prompt, so its order must not
    /// change from run to run.
    #[tokio::test]
    async fn registration_order_is_preserved() {
        let runner = ToolRunner::new();
        for name in ["zeta", "alpha", "mid"] {
            runner.register(echo(name)).await.unwrap();
        }
        let names: Vec<String> = runner
            .tools
            .read()
            .await
            .iter()
            .map(|t| t.name().to_string())
            .collect();
        assert_eq!(names, vec!["zeta", "alpha", "mid"]);
    }

    /// Three 100ms tools run in ~100ms, not ~300ms.
    #[tokio::test]
    async fn a_batch_runs_concurrently() {
        let runner = ToolRunner::new();
        for name in ["a", "b", "c"] {
            runner
                .register(Arc::new(Echo {
                    name,
                    delay_ms: 100,
                }))
                .await
                .unwrap();
        }
        let calls: Vec<crate::types::ToolCall> = ["a", "b", "c"]
            .iter()
            .enumerate()
            .map(|(i, name)| crate::types::ToolCall {
                id: i.to_string(),
                name: (*name).to_string(),
                args: serde_json::json!({"n": i}),
                canonical_path: None,
                server_name: None,
            })
            .collect();

        let started = std::time::Instant::now();
        let results = runner.process_tool_calls(calls).await;
        let elapsed = started.elapsed();

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.error.is_none()));
        assert!(
            elapsed < std::time::Duration::from_millis(250),
            "batch took {elapsed:?}; it ran sequentially"
        );
        // Results stay in call order regardless of completion order.
        let ids: Vec<_> = results.iter().filter_map(|r| r.id.clone()).collect();
        assert_eq!(ids, vec!["0", "1", "2"]);
    }

    struct NeedsContext;

    impl Tool for NeedsContext {
        fn name(&self) -> &'static str {
            "whoami"
        }
        fn description(&self) -> &'static str {
            "reports the conversation id"
        }
        fn parameters_json_schema(&self) -> &'static str {
            "{}"
        }
        async fn call(&self, _args: Value) -> Result<Value, anyhow::Error> {
            Err(anyhow::anyhow!("must be called with a context"))
        }
        fn needs_context(&self) -> bool {
            true
        }
        async fn call_with_context(
            &self,
            _args: Value,
            context: &crate::tool_context::ToolContext,
        ) -> Result<Value, anyhow::Error> {
            Ok(Value::String(context.conversation_id().unwrap_or_default()))
        }
    }

    fn whoami_call() -> crate::types::ToolCall {
        crate::types::ToolCall {
            id: "1".to_string(),
            name: "whoami".to_string(),
            args: Value::Null,
            canonical_path: None,
            server_name: None,
        }
    }

    /// The context existed but was never constructed, so a `needs_context`
    /// tool silently ran through the plain `call()` path (audit T1).
    #[tokio::test]
    async fn a_context_aware_tool_is_called_with_the_context() {
        use crate::connection::{AnyConnection, MockConnection};

        let runner = ToolRunner::new();
        runner.register(Arc::new(NeedsContext)).await.unwrap();

        let conn = Arc::new(MockConnection::new("conv-42"));
        let any = AnyConnection::Mock(conn.clone());
        runner
            .set_context(Arc::new(crate::tool_context::ToolContext::new(
                any.downgrade(),
            )))
            .await;

        let results = runner.process_tool_calls(vec![whoami_call()]).await;
        assert_eq!(
            results[0].result,
            Some(Value::String("conv-42".to_string()))
        );

        // Once the session is gone the context reports nothing rather than
        // keeping it alive.
        drop(any);
        drop(conn);
        let results = runner.process_tool_calls(vec![whoami_call()]).await;
        assert_eq!(results[0].result, Some(Value::String(String::new())));
    }

    /// Running such a tool without a context would behave subtly differently
    /// rather than failing, which is worse.
    #[tokio::test]
    async fn a_context_aware_tool_without_a_context_errors() {
        let runner = ToolRunner::new();
        runner.register(Arc::new(NeedsContext)).await.unwrap();
        let results = runner.process_tool_calls(vec![whoami_call()]).await;
        assert!(
            results[0]
                .error
                .as_deref()
                .unwrap()
                .contains("requires a ToolContext")
        );
    }

    #[tokio::test]
    async fn an_unknown_tool_reports_an_error_result() {
        let runner = ToolRunner::new();
        let results = runner
            .process_tool_calls(vec![crate::types::ToolCall {
                id: "1".to_string(),
                name: "nope".to_string(),
                args: Value::Null,
                canonical_path: None,
                server_name: None,
            }])
            .await;
        assert_eq!(results.len(), 1);
        assert!(results[0].error.as_deref().unwrap().contains("not found"));
    }
}
