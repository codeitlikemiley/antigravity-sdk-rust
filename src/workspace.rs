//! Workspace-root resolution shared by every transport.
//!
//! The client-side policy layer and the harness must be told about the same
//! workspace roots. Resolving them in one place is what keeps the two halves
//! from disagreeing — see the module tests for the contract.

/// Returns the default workspace list used when the caller configured none.
///
/// Mirrors upstream `local_connection_config.py:54`, whose field default is
/// `default_factory=lambda: [os.getcwd()]`.
///
/// Returns an empty `Vec` when the current directory cannot be read (a deleted
/// cwd, or `wasm32-unknown-unknown` where there is no filesystem).
#[must_use]
pub fn default_workspaces() -> Vec<String> {
    std::env::current_dir().map_or_else(
        |_| Vec::new(),
        |cwd| vec![cwd.to_string_lossy().into_owned()],
    )
}

/// Resolves the effective workspace roots for an agent configuration.
///
/// `None` (never configured) yields `[cwd]`. `Some(v)` yields exactly `v` —
/// including `Some(vec![])`, which upstream documents as the way to ask for
/// unrestricted access (`local_connection_config.py:142`).
#[must_use]
pub fn resolve(configured: Option<&Vec<String>>) -> Vec<String> {
    configured
        .map_or_else(default_workspaces, Clone::clone)
        .iter()
        .map(|w| crate::wire_path::normalize_wire_path(w))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_workspaces_returns_cwd() {
        let Ok(cwd) = std::env::current_dir() else {
            // No readable cwd (deleted directory, or a target without a
            // filesystem): the documented fail-soft is an empty list.
            assert!(default_workspaces().is_empty());
            return;
        };
        assert_eq!(
            default_workspaces(),
            vec![cwd.to_string_lossy().into_owned()]
        );
    }

    #[test]
    fn resolve_none_is_cwd() {
        assert_eq!(resolve(None), default_workspaces());
    }

    /// Upstream `local_connection_config.py:142` documents `workspaces=[]` as
    /// the opt-out for unrestricted access, so an explicit empty list must
    /// survive resolution rather than being treated as "unconfigured".
    #[test]
    fn resolve_some_empty_stays_empty() {
        let configured = Vec::new();
        assert!(resolve(Some(&configured)).is_empty());
    }

    #[test]
    fn resolve_some_explicit_is_passthrough() {
        let configured = vec!["/a".to_string(), "/b".to_string()];
        assert_eq!(resolve(Some(&configured)), configured);
    }
}
