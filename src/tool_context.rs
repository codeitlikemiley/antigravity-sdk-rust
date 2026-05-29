//! Conversation-aware context injected into tools that request it.
//!
//! `ToolContext` provides access to session state, conversation metadata,
//! and the ability to send messages to the agent. State is scoped to the
//! session and is independent of `HookContext`.

use crate::connection::AnyConnection;
use crate::connection::Connection;
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
    connection: AnyConnection,
    state: Mutex<HashMap<String, Value>>,
}

impl ToolContext {
    /// Creates a new `ToolContext` wrapping the given connection.
    pub fn new(connection: AnyConnection) -> Self {
        Self {
            connection,
            state: Mutex::new(HashMap::new()),
        }
    }

    /// Returns the conversation ID for the current session.
    pub fn conversation_id(&self) -> &str {
        self.connection.conversation_id()
    }

    /// Returns whether the agent is currently idle (not processing).
    pub fn is_idle(&self) -> bool {
        self.connection.is_idle()
    }

    /// Sends a trigger notification message to the agent.
    pub async fn send(&self, message: &str) -> Result<()> {
        self.connection.send_trigger_notification(message).await
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
}
