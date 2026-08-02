//! Custom client-side tool definition and execution.
//!
//! This module defines the [`Tool`] trait, allowing custom functionality (e.g. database access,
//! API requests) to be registered with the agent and executed when requested by the model.
//! Registration and concurrent execution is managed via [`ToolRunner`].

use crate::tool_context::ToolContext;
use futures_util::future::BoxFuture;
use serde_json::Value;
use std::collections::HashMap;
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
    /// Active tools registered with the runner.
    pub tools: Arc<tokio::sync::RwLock<HashMap<String, Arc<dyn DynTool>>>>,
}

impl std::fmt::Debug for ToolRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ToolRunner")
            .field("tools_count", &self.tools.try_read().map_or(0, |t| t.len()))
            .finish()
    }
}

impl ToolRunner {
    /// Creates a new, empty `ToolRunner`.
    pub fn new() -> Self {
        Self {
            tools: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Registers a custom tool implementation.
    pub async fn register(&self, tool: Arc<dyn DynTool>) {
        self.tools
            .write()
            .await
            .insert(tool.name().to_string(), tool);
    }

    /// Executes a list of tool invocations, mapping their outputs to [`ToolResult`](crate::types::ToolResult)s.
    pub async fn process_tool_calls(
        &self,
        calls: Vec<crate::types::ToolCall>,
    ) -> Vec<crate::types::ToolResult> {
        let mut results = Vec::new();
        for call in calls {
            let tools = self.tools.read().await;
            if let Some(tool) = tools.get(&call.name) {
                match tool.call(call.args).await {
                    Ok(val) => {
                        results.push(crate::types::ToolResult {
                            id: Some(call.id),
                            name: call.name.clone(),
                            result: Some(val),
                            error: None,
                        });
                    }
                    Err(e) => {
                        results.push(crate::types::ToolResult {
                            id: Some(call.id),
                            name: call.name.clone(),
                            result: None,
                            error: Some(e.to_string()),
                        });
                    }
                }
            } else {
                results.push(crate::types::ToolResult {
                    id: Some(call.id),
                    name: call.name.clone(),
                    result: None,
                    error: Some(format!("Tool {} not found", call.name)),
                });
            }
        }
        results
    }
}
