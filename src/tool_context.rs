//! Conversation-aware context injected into tools that request it.
//!
//! `ToolContext` provides access to session state, conversation metadata,
//! and the ability to send messages to the agent. State is scoped to the
//! session and is independent of `HookContext`.

use crate::connection::Connection;
use crate::connection::WeakConnection;
use crate::state::StateStore;
use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};

/// Session-scoped context injected into tools that need conversation awareness.
///
/// Provides:
/// - `conversation_id()` — current session identifier
/// - `is_idle()` — whether the agent is idle
/// - `send()` — push a trigger notification to the agent
/// - `get_state()` / `set_state()` — per-session key-value store
///
/// State set by tools is **not** visible to hooks, and vice versa.
/// This separation is intentional (see hooks/README.md in the Python SDK).
#[derive(Debug)]
pub struct ToolContext {
    connection: WeakConnection,
    state: StateStore,
}

impl ToolContext {
    /// Creates a new `ToolContext` holding a non-owning handle to the session.
    ///
    /// Weak by construction: the connection owns the tool runner, and a strong
    /// handle back would keep the session alive forever.
    pub fn new(connection: WeakConnection) -> Self {
        Self {
            connection,
            state: StateStore::new(),
        }
    }

    /// The session store backing this context.
    ///
    /// Cloning it shares the entries, so a caller can read what a tool wrote.
    #[must_use]
    pub fn state(&self) -> StateStore {
        self.state.clone()
    }

    /// Returns the conversation ID, or `None` once the session has ended.
    pub fn conversation_id(&self) -> Option<String> {
        self.connection
            .upgrade()
            .map(|c| c.conversation_id().to_string())
    }

    /// Returns whether the agent is currently idle, or `None` once the session
    /// has ended.
    pub fn is_idle(&self) -> Option<bool> {
        self.connection.upgrade().map(|c| c.is_idle())
    }

    /// Sends a trigger notification message to the agent.
    ///
    /// # Errors
    ///
    /// Returns an error if the session has ended or the message cannot be sent.
    pub async fn send(&self, message: &str) -> Result<()> {
        let connection = self
            .connection
            .upgrade()
            .ok_or_else(|| anyhow::anyhow!("the session has ended"))?;
        connection.send_trigger_notification(message).await
    }

    /// Retrieves a previously stored value by key.
    /// Returns `None` if the key doesn't exist or deserialization fails.
    pub fn get_state<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.state.get(key)
    }

    /// Stores a value by key in the session-scoped state store.
    pub fn set_state<T: Serialize>(&self, key: &str, value: T) {
        self.state.set(key, value);
    }

    /// Atomically reads, transforms and writes a state entry.
    ///
    /// `get_state` followed by `set_state` releases the lock in between, so two
    /// tools running concurrently can both read the old value and one write is
    /// lost. This holds the lock across the transform, which is the only safe
    /// way to do read-modify-write on shared state. Mirrors upstream's
    /// `update_state` (`utils/state.py`, added 0.1.7).
    ///
    /// The closure receives the current value, or `None` when the key is unset.
    /// Returning `None` leaves the entry untouched.
    ///
    /// ```
    /// # use antigravity_sdk_rust::tool_context::ToolContext;
    /// # fn demo(ctx: &ToolContext) {
    /// ctx.update_state::<u32, _>("calls", |current| Some(current.unwrap_or(0) + 1));
    /// # }
    /// ```
    pub fn update_state<T, F>(&self, key: &str, transform: F)
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce(Option<T>) -> Option<T>,
    {
        self.state.update(key, transform);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::ToolContext;
    use crate::connection::{AnyConnection, MockConnection};
    use std::sync::Arc;

    fn context() -> (Arc<MockConnection>, AnyConnection, ToolContext) {
        let mock = Arc::new(MockConnection::new("conv-1"));
        let any = AnyConnection::Mock(mock.clone());
        let context = ToolContext::new(any.downgrade());
        (mock, any, context)
    }

    #[test]
    fn state_round_trips_through_the_shared_store() {
        let (_mock, _any, context) = context();
        context.set_state("key", "value");
        assert_eq!(context.get_state::<String>("key").as_deref(), Some("value"));
        context.update_state::<u32, _>("calls", |c| Some(c.unwrap_or(0) + 1));
        context.update_state::<u32, _>("calls", |c| Some(c.unwrap_or(0) + 1));
        assert_eq!(context.get_state::<u32>("calls"), Some(2));
        // The handle sees the same entries.
        assert_eq!(context.state().get::<u32>("calls"), Some(2));
    }

    #[test]
    fn conversation_id_and_idle_track_the_connection() {
        let (mock, any, context) = context();
        assert_eq!(context.conversation_id().as_deref(), Some("conv-1"));
        assert_eq!(context.is_idle(), Some(true));

        // Once the session is gone the context reports nothing rather than
        // keeping it alive.
        drop(any);
        drop(mock);
        assert_eq!(context.conversation_id(), None);
        assert_eq!(context.is_idle(), None);
    }

    #[tokio::test]
    async fn send_errors_once_the_session_has_ended() {
        let (mock, any, context) = context();
        context.send("wake up").await.expect("live session");
        drop(any);
        drop(mock);
        let err = context.send("wake up").await.expect_err("session is gone");
        assert!(err.to_string().contains("session has ended"), "{err}");
    }
}
