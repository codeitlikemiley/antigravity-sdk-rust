//! Connection abstractions for communication with the agent execution harness.
//!
//! This module defines the [`Connection`] trait, which provides a transport-agnostic interface
//! for sending prompts, receiving step updates, and executing tool and question handshakes.

use crate::types::{QuestionHookResult, Step, ToolResult};
#[cfg(test)]
use futures_util::StreamExt;
use futures_util::stream::BoxStream;

/// Trait representing an active session connection with the agentic harness.
///
/// A connection manages process or network lifecycle, handshaking, and event streaming.
/// Common implementations include [`LocalConnection`](crate::local::LocalConnection) which spawns
/// a local helper subprocess and upgrades communication to a WebSocket protocol.
pub trait Connection: Send + Sync {
    /// Returns the conversation ID for the connection.
    fn conversation_id(&self) -> &str;

    /// Returns whether the connection is currently idle.
    fn is_idle(&self) -> bool;

    /// Resolves once the connection is idle.
    ///
    /// Returns immediately if it already is. Callers that need to know a turn
    /// has finished previously had to poll `is_idle()` in a sleep loop, which
    /// is both slower to notice and easy to write as a busy wait.
    fn wait_for_idle(&self) -> impl std::future::Future<Output = ()> + Send;

    /// Subscribes to the stream of step updates from the connection.
    fn receive_steps(&self) -> BoxStream<'static, Result<Step, anyhow::Error>>;

    /// Sends a standard user text prompt to the connection.
    fn send(
        &self,
        content: &str,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;

    /// Sends a trigger notification message to the connection.
    fn send_trigger_notification(
        &self,
        content: &str,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;

    /// Sends a halt/cancellation request to the connection.
    fn send_halt_request(
        &self,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;

    /// Sends approval/rejection for a tool execution confirmation request.
    fn send_tool_confirmation(
        &self,
        trajectory_id: &str,
        step_index: u32,
        accepted: bool,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;

    /// Sends the result of a tool execution back to the connection.
    fn send_tool_response(
        &self,
        id: &str,
        result: ToolResult,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;

    /// Sends the answers to user questions back to the connection.
    fn send_question_response(
        &self,
        trajectory_id: &str,
        step_index: u32,
        answers: QuestionHookResult,
    ) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;

    /// Gracefully closes the connection.
    fn disconnect(&self) -> impl std::future::Future<Output = Result<(), anyhow::Error>> + Send;
}

/// A target-agnostic wrapper enum for active connection types.
#[derive(Clone)]
pub enum AnyConnection {
    #[cfg(not(target_arch = "wasm32"))]
    Local(std::sync::Arc<crate::local::LocalConnection>),
    #[cfg(target_arch = "wasm32")]
    Wasm(std::sync::Arc<crate::wasm::WasmConnection>),
    #[cfg(test)]
    Mock(std::sync::Arc<MockConnection>),
}

/// A non-owning handle to a connection.
///
/// The tool runner is owned by the connection, and a [`ToolContext`] handed to
/// a tool points back at that connection — holding it strongly would make a
/// reference cycle that never frees the session. Tools upgrade on use and see
/// `None` once the agent has stopped.
///
/// [`ToolContext`]: crate::tool_context::ToolContext
#[derive(Clone)]
pub enum WeakConnection {
    #[cfg(not(target_arch = "wasm32"))]
    Local(std::sync::Weak<crate::local::LocalConnection>),
    #[cfg(target_arch = "wasm32")]
    Wasm(std::sync::Weak<crate::wasm::WasmConnection>),
    #[cfg(test)]
    Mock(std::sync::Weak<MockConnection>),
}

impl std::fmt::Debug for WeakConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WeakConnection")
    }
}

impl WeakConnection {
    /// Returns a live connection, or `None` if the session has ended.
    #[must_use]
    pub fn upgrade(&self) -> Option<AnyConnection> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local(c) => c.upgrade().map(AnyConnection::Local),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(c) => c.upgrade().map(AnyConnection::Wasm),
            #[cfg(test)]
            Self::Mock(c) => c.upgrade().map(AnyConnection::Mock),
        }
    }
}

impl AnyConnection {
    /// Produces a non-owning handle to this connection.
    #[must_use]
    pub fn downgrade(&self) -> WeakConnection {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local(c) => WeakConnection::Local(std::sync::Arc::downgrade(c)),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(c) => WeakConnection::Wasm(std::sync::Arc::downgrade(c)),
            #[cfg(test)]
            Self::Mock(c) => WeakConnection::Mock(std::sync::Arc::downgrade(c)),
        }
    }
}

impl std::fmt::Debug for AnyConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local(_) => f.write_str("AnyConnection::Local"),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(_) => f.write_str("AnyConnection::Wasm"),
            #[cfg(test)]
            Self::Mock(_) => f.write_str("AnyConnection::Mock"),
        }
    }
}

impl Connection for AnyConnection {
    fn conversation_id(&self) -> &str {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local(c) => c.conversation_id(),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(c) => c.conversation_id(),
            #[cfg(test)]
            Self::Mock(c) => c.conversation_id(),
        }
    }

    fn is_idle(&self) -> bool {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local(c) => c.is_idle(),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(c) => c.is_idle(),
            #[cfg(test)]
            Self::Mock(c) => c.is_idle(),
        }
    }

    async fn wait_for_idle(&self) {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local(c) => c.wait_for_idle().await,
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(c) => c.wait_for_idle().await,
            #[cfg(test)]
            Self::Mock(c) => c.wait_for_idle().await,
        }
    }

    fn receive_steps(&self) -> BoxStream<'static, Result<Step, anyhow::Error>> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local(c) => c.receive_steps(),
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(c) => c.receive_steps(),
            #[cfg(test)]
            Self::Mock(c) => c.receive_steps(),
        }
    }

    async fn send(&self, content: &str) -> Result<(), anyhow::Error> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local(c) => c.send(content).await,
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(c) => c.send(content).await,
            #[cfg(test)]
            Self::Mock(c) => c.send(content).await,
        }
    }

    async fn send_trigger_notification(&self, content: &str) -> Result<(), anyhow::Error> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local(c) => c.send_trigger_notification(content).await,
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(c) => c.send_trigger_notification(content).await,
            #[cfg(test)]
            Self::Mock(c) => c.send_trigger_notification(content).await,
        }
    }

    async fn send_halt_request(&self) -> Result<(), anyhow::Error> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local(c) => c.send_halt_request().await,
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(c) => c.send_halt_request().await,
            #[cfg(test)]
            Self::Mock(c) => c.send_halt_request().await,
        }
    }

    async fn send_tool_confirmation(
        &self,
        trajectory_id: &str,
        step_index: u32,
        accepted: bool,
    ) -> Result<(), anyhow::Error> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local(c) => {
                c.send_tool_confirmation(trajectory_id, step_index, accepted)
                    .await
            }
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(c) => {
                c.send_tool_confirmation(trajectory_id, step_index, accepted)
                    .await
            }
            #[cfg(test)]
            Self::Mock(c) => {
                c.send_tool_confirmation(trajectory_id, step_index, accepted)
                    .await
            }
        }
    }

    async fn send_tool_response(&self, id: &str, result: ToolResult) -> Result<(), anyhow::Error> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local(c) => c.send_tool_response(id, result).await,
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(c) => c.send_tool_response(id, result).await,
            #[cfg(test)]
            Self::Mock(c) => c.send_tool_response(id, result).await,
        }
    }

    async fn send_question_response(
        &self,
        trajectory_id: &str,
        step_index: u32,
        answers: QuestionHookResult,
    ) -> Result<(), anyhow::Error> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local(c) => {
                c.send_question_response(trajectory_id, step_index, answers)
                    .await
            }
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(c) => {
                c.send_question_response(trajectory_id, step_index, answers)
                    .await
            }
            #[cfg(test)]
            Self::Mock(c) => {
                c.send_question_response(trajectory_id, step_index, answers)
                    .await
            }
        }
    }

    async fn disconnect(&self) -> Result<(), anyhow::Error> {
        match self {
            #[cfg(not(target_arch = "wasm32"))]
            Self::Local(c) => c.disconnect().await,
            #[cfg(target_arch = "wasm32")]
            Self::Wasm(c) => c.disconnect().await,
            #[cfg(test)]
            Self::Mock(c) => c.disconnect().await,
        }
    }
}

#[cfg(test)]
pub struct MockConnection {
    pub id: String,
    pub is_idle: std::sync::atomic::AtomicBool,
    pub steps_to_yield: std::sync::Mutex<Vec<Step>>,
    pub sent_prompts: std::sync::Mutex<Vec<String>>,
}

#[cfg(test)]
impl std::fmt::Debug for MockConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MockConnection")
            .field("id", &self.id)
            .field(
                "is_idle",
                &self.is_idle.load(std::sync::atomic::Ordering::Relaxed),
            )
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
impl MockConnection {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            is_idle: std::sync::atomic::AtomicBool::new(true),
            steps_to_yield: std::sync::Mutex::new(Vec::new()),
            sent_prompts: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// Queues the steps `receive_steps()` will yield.
    pub fn set_steps(&self, steps: Vec<Step>) {
        *self
            .steps_to_yield
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = steps;
    }

    /// Sets what `is_idle()` reports.
    pub fn set_idle(&self, idle: bool) {
        self.is_idle
            .store(idle, std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(test)]
impl Connection for MockConnection {
    fn conversation_id(&self) -> &str {
        &self.id
    }

    fn is_idle(&self) -> bool {
        self.is_idle.load(std::sync::atomic::Ordering::SeqCst)
    }

    async fn wait_for_idle(&self) {
        while !self.is_idle() {
            tokio::task::yield_now().await;
        }
    }

    fn receive_steps(&self) -> BoxStream<'static, Result<Step, anyhow::Error>> {
        let steps = self
            .steps_to_yield
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        futures_util::stream::iter(steps).map(Ok).boxed()
    }

    async fn send(&self, content: &str) -> Result<(), anyhow::Error> {
        self.sent_prompts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(content.to_string());
        Ok(())
    }

    async fn send_trigger_notification(&self, _content: &str) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn send_halt_request(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn send_tool_confirmation(
        &self,
        _trajectory_id: &str,
        _step_index: u32,
        _accepted: bool,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn send_tool_response(
        &self,
        _id: &str,
        _result: ToolResult,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn send_question_response(
        &self,
        _trajectory_id: &str,
        _step_index: u32,
        _answers: QuestionHookResult,
    ) -> Result<(), anyhow::Error> {
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), anyhow::Error> {
        Ok(())
    }
}
