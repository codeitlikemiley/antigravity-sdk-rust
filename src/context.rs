//! Hierarchical context system for hook state management.
//!
//! Provides a parent-chaining key-value store where `get()` walks the parent chain
//! and `set()` writes only to the local store. This enables state sharing across
//! hook lifecycle events (session → turn → operation scope).

use crate::state::StateStore;
use serde::{Serialize, de::DeserializeOwned};
use std::sync::Arc;

/// A hierarchical key-value store for sharing state across hook invocations.
///
/// Contexts form a chain: `OperationContext` → `TurnContext` → `SessionContext`.
/// - `get()` searches the local store first, then walks up the parent chain.
/// - `set()` writes only to the local store (shadowing, not mutating parents).
#[derive(Debug, Clone)]
pub struct HookContext {
    parent: Option<Arc<Self>>,
    store: StateStore,
}

impl HookContext {
    /// Creates a root context (no parent). Used as the session-level context.
    pub fn new() -> Self {
        Self {
            parent: None,
            store: StateStore::new(),
        }
    }

    /// Creates a child context with the given parent.
    /// Used to create turn-level (parent=session) or operation-level (parent=turn) contexts.
    pub fn child(parent: Arc<Self>) -> Self {
        Self {
            parent: Some(parent),
            store: StateStore::new(),
        }
    }

    /// Retrieves a value by key, walking up the parent chain if not found locally.
    /// Returns `None` if the key is not found in any context in the hierarchy.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.store
            .get(key)
            .or_else(|| self.parent.as_ref().and_then(|parent| parent.get(key)))
    }

    /// Sets a value in the **local** store only (does not write to parents).
    /// If the key already exists locally, it is overwritten.
    pub fn set<T: Serialize>(&self, key: &str, value: T) {
        self.store.set(key, value);
    }

    /// Atomically reads, transforms and writes a **local** entry.
    ///
    /// Does not walk the parent chain: a read-modify-write that fell through to
    /// a parent would write the result locally and leave the parent stale,
    /// which reads as a lost update.
    pub fn update<T, F>(&self, key: &str, transform: F)
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce(Option<T>) -> Option<T>,
    {
        self.store.update(key, transform);
    }

    /// Returns `true` if this context has a parent (i.e., is not a root/session context).
    pub const fn has_parent(&self) -> bool {
        self.parent.is_some()
    }
}

impl Default for HookContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_root_context_get_set() {
        let ctx = HookContext::new();
        ctx.set("key1", "value1");
        assert_eq!(ctx.get::<String>("key1"), Some("value1".to_string()));
        assert_eq!(ctx.get::<String>("missing"), None);
    }

    #[test]
    fn test_child_inherits_parent() {
        let session = Arc::new(HookContext::new());
        session.set("session_key", "session_value");

        let turn = HookContext::child(session);
        // Child can read parent values
        assert_eq!(
            turn.get::<String>("session_key"),
            Some("session_value".to_string())
        );
    }

    #[test]
    fn test_child_shadows_parent() {
        let session = Arc::new(HookContext::new());
        session.set("key", "parent_value");

        let turn = HookContext::child(session.clone());
        turn.set("key", "child_value");

        // Child sees its own value
        assert_eq!(turn.get::<String>("key"), Some("child_value".to_string()));
        // Parent is unchanged
        assert_eq!(
            session.get::<String>("key"),
            Some("parent_value".to_string())
        );
    }

    #[test]
    fn test_three_level_hierarchy() {
        let session = Arc::new(HookContext::new());
        session.set("level", "session");
        session.set("session_only", true);

        let turn = Arc::new(HookContext::child(session));
        turn.set("level", "turn");
        turn.set("turn_only", 42i32);

        let operation = HookContext::child(turn);
        operation.set("level", "operation");

        // Operation sees its own, then turn, then session
        assert_eq!(
            operation.get::<String>("level"),
            Some("operation".to_string())
        );
        assert_eq!(operation.get::<i32>("turn_only"), Some(42));
        assert_eq!(operation.get::<bool>("session_only"), Some(true));
    }

    #[test]
    fn test_no_parent_for_root() {
        let ctx = HookContext::new();
        assert!(!ctx.has_parent());
    }

    #[test]
    fn test_child_has_parent() {
        let parent = Arc::new(HookContext::new());
        let child = HookContext::child(parent);
        assert!(child.has_parent());
    }

    #[test]
    fn test_default_is_root() {
        let ctx = HookContext::default();
        assert!(!ctx.has_parent());
    }
}
