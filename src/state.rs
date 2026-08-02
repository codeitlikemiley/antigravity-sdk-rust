//! The session-scoped key-value store shared by hook and tool contexts.
//!
//! `HookContext` and `ToolContext` each carried their own `Mutex<HashMap<..>>`
//! and their own copy of the read-modify-write logic. The two drifted — only
//! one of them had an atomic `update_state` — and neither could be tested
//! without constructing the context that owned it.
//!
//! The store lives here so there is one implementation. Hook state and tool
//! state remain **separate stores**, deliberately: upstream keeps them apart so
//! a hook cannot silently depend on a tool's bookkeeping. Sharing the type is
//! not sharing the data.

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// A cloneable handle to one session-scoped store.
///
/// Cloning shares the underlying map, so a store handed to two callers is one
/// store.
#[derive(Debug, Clone, Default)]
pub struct StateStore {
    entries: Arc<Mutex<HashMap<String, Value>>>,
}

impl StateStore {
    /// Creates an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Retrieves a value by key.
    ///
    /// `None` if the key is unset, or if the stored value does not deserialize
    /// into `T`.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.entries
            .lock()
            .ok()
            .and_then(|entries| entries.get(key).cloned())
            .and_then(|value| serde_json::from_value(value).ok())
    }

    /// Stores a value by key.
    ///
    /// A value that cannot be serialized is dropped rather than panicking:
    /// bookkeeping must not take down the turn.
    pub fn set<T: Serialize>(&self, key: &str, value: T) {
        if let Ok(mut entries) = self.entries.lock()
            && let Ok(value) = serde_json::to_value(value)
        {
            entries.insert(key.to_string(), value);
        }
    }

    /// Atomically reads, transforms and writes an entry.
    ///
    /// `get` followed by `set` releases the lock in between, so two callers can
    /// both read the old value and one write is lost. This holds the lock
    /// across the transform, which is the only safe way to do read-modify-write
    /// on shared state. Mirrors upstream's `update_state` (`utils/state.py`,
    /// added 0.1.7).
    ///
    /// The closure receives the current value, or `None` when the key is unset.
    /// Returning `None` leaves the entry untouched.
    pub fn update<T, F>(&self, key: &str, transform: F)
    where
        T: Serialize + DeserializeOwned,
        F: FnOnce(Option<T>) -> Option<T>,
    {
        let Ok(mut entries) = self.entries.lock() else {
            return;
        };
        let current = entries
            .get(key)
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok());
        if let Some(next) = transform(current)
            && let Ok(value) = serde_json::to_value(next)
        {
            entries.insert(key.to_string(), value);
        }
    }

    /// Removes an entry, returning whether it was there.
    pub fn remove(&self, key: &str) -> bool {
        self.entries
            .lock()
            .is_ok_and(|mut entries| entries.remove(key).is_some())
    }

    /// Empties the store.
    pub fn clear(&self) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.clear();
        }
    }

    /// How many entries are stored.
    pub fn len(&self) -> usize {
        self.entries.lock().map_or(0, |entries| entries.len())
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::StateStore;

    #[test]
    fn set_get_and_overwrite() {
        let store = StateStore::new();
        assert!(store.is_empty());
        store.set("key", "value");
        assert_eq!(store.get::<String>("key").as_deref(), Some("value"));
        store.set("key", 2i32);
        assert_eq!(store.get::<i32>("key"), Some(2));
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn a_mistyped_read_is_none_not_a_panic() {
        let store = StateStore::new();
        store.set("key", "not a number");
        assert_eq!(store.get::<i32>("key"), None);
    }

    /// The point of `update`: a read-modify-write that cannot interleave. A
    /// `get` + `set` pair releases the lock in between, so a concurrent
    /// increment is lost and this would come out under 800.
    #[test]
    fn update_is_atomic_across_threads() {
        let store = StateStore::new();
        let mut handles = Vec::new();
        for _ in 0..8 {
            let store = store.clone();
            handles.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    store.update::<u32, _>("n", |current| Some(current.unwrap_or(0) + 1));
                }
            }));
        }
        for handle in handles {
            handle.join().ok();
        }
        assert_eq!(store.get::<u32>("n"), Some(800));
    }

    #[test]
    fn update_returning_none_leaves_the_entry() {
        let store = StateStore::new();
        store.update::<u32, _>("k", |_| Some(7));
        store.update::<u32, _>("k", |_| None);
        assert_eq!(store.get::<u32>("k"), Some(7));
    }

    /// A clone is the same store, which is what makes it shareable between a
    /// context and whatever created it.
    #[test]
    fn a_clone_shares_the_entries() {
        let store = StateStore::new();
        let other = store.clone();
        other.set("k", 1i32);
        assert_eq!(store.get::<i32>("k"), Some(1));
        assert!(store.remove("k"));
        assert!(other.is_empty());
    }
}
