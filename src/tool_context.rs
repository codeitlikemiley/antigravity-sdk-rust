//! Conversation-aware context injected into tools that request it.
//!
//! `ToolContext` provides access to session state, conversation metadata,
//! and the ability to send messages to the agent. State is scoped to the
//! session and is independent of `HookContext`.

use crate::connection::Connection;
use crate::connection::WeakConnection;
use anyhow::Result;
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

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
    state: Mutex<HashMap<String, Value>>,
}

impl ToolContext {
    /// Creates a new `ToolContext` holding a non-owning handle to the session.
    ///
    /// Weak by construction: the connection owns the tool runner, and a strong
    /// handle back would keep the session alive forever.
    pub fn new(connection: WeakConnection) -> Self {
        Self {
            connection,
            state: Mutex::new(HashMap::new()),
        }
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
        self.state
            .lock()
            .ok()
            .and_then(|store| store.get(key).cloned())
            .and_then(|v| serde_json::from_value(v).ok())
    }

    /// Stores a value by key in the session-scoped state store.
    #[allow(clippy::collapsible_if)]
    pub fn set_state<T: Serialize>(&self, key: &str, value: T) {
        if let Ok(mut store) = self.state.lock() {
            if let Ok(v) = serde_json::to_value(value) {
                store.insert(key.to_string(), v);
            }
        }
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
        update_locked(&self.state, key, transform);
    }
}

/// The read-modify-write half of [`ToolContext::update_state`], separated so it
/// can be tested without a live connection.
fn update_locked<T, F>(state: &Mutex<HashMap<String, Value>>, key: &str, transform: F)
where
    T: Serialize + DeserializeOwned,
    F: FnOnce(Option<T>) -> Option<T>,
{
    let Ok(mut store) = state.lock() else {
        return;
    };
    let current = store
        .get(key)
        .cloned()
        .and_then(|v| serde_json::from_value(v).ok());
    if let Some(next) = transform(current)
        && let Ok(v) = serde_json::to_value(next)
    {
        store.insert(key.to_string(), v);
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::significant_drop_tightening
    )]
    use super::*;
    use std::sync::Arc;

    // ToolContext tests require a mock connection which is only available
    // via the full test harness. Unit tests here validate the state store.
    #[test]
    fn test_state_set_and_get() {
        let state: Mutex<HashMap<String, Value>> = Mutex::new(HashMap::new());
        state
            .lock()
            .unwrap()
            .insert("key".to_string(), serde_json::to_value("value").unwrap());
        let val: String =
            serde_json::from_value(state.lock().unwrap().get("key").cloned().unwrap()).unwrap();
        assert_eq!(val, "value");
    }

    #[test]
    fn test_state_overwrite() {
        let state: Mutex<HashMap<String, Value>> = Mutex::new(HashMap::new());
        {
            let mut store = state.lock().unwrap();
            store.insert("key".to_string(), serde_json::to_value(1i32).unwrap());
            store.insert("key".to_string(), serde_json::to_value(2i32).unwrap());
        }
        let val: i32 =
            serde_json::from_value(state.lock().unwrap().get("key").cloned().unwrap()).unwrap();
        assert_eq!(val, 2);
    }

    /// The point of the method: a read-modify-write that cannot interleave.
    /// A `get_state` + `set_state` pair releases the lock in between, so a
    /// concurrent increment is lost — 800 here would come out lower.
    #[test]
    fn update_locked_is_atomic_across_threads() {
        let state: Arc<Mutex<HashMap<String, Value>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut handles = Vec::new();
        for _ in 0..8 {
            let state = state.clone();
            handles.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    update_locked::<u32, _>(&state, "n", |c| Some(c.unwrap_or(0) + 1));
                }
            }));
        }
        for h in handles {
            h.join().ok();
        }
        let stored: u32 =
            serde_json::from_value(state.lock().unwrap().get("n").cloned().unwrap()).unwrap();
        assert_eq!(stored, 800);
    }

    #[test]
    fn update_locked_returning_none_leaves_the_entry() {
        let state: Mutex<HashMap<String, Value>> = Mutex::new(HashMap::new());
        update_locked::<u32, _>(&state, "k", |_| Some(7));
        update_locked::<u32, _>(&state, "k", |_| None);
        let stored: u32 =
            serde_json::from_value(state.lock().unwrap().get("k").cloned().unwrap()).unwrap();
        assert_eq!(stored, 7);
    }
}
