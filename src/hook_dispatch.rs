//! Hook plumbing shared by both transports.
//!
//! `src/local.rs` and `src/wasm.rs` are forks of each other, and every piece of
//! hook wiring added to one has historically been missing from the other. The
//! target-neutral half lives here so a dispatch site exists once.

use crate::hooks::HookRunner;
use anyhow::anyhow;

/// Which lifecycle hooks a [`Hook`](crate::hooks::Hook) implementation wants.
///
/// The `Hook` trait gives every method a default, so there is no way to tell
/// from the type which ones an implementation actually overrode. Declaring is
/// the prerequisite for `HarnessConfig.enabled_hooks` (field 16): the harness
/// blocks its turn waiting for a `CallHookResponse` for every kind it is told
/// about, so the list must name only what this side will really answer.
///
/// The default is [`NONE`](Self::NONE) — declaring is **opt-in**. Local
/// dispatch is unaffected either way: a hook that declares nothing still has
/// every method called by the runner, exactly as before.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HookKinds(u16);

impl HookKinds {
    /// Declares nothing. The default.
    pub const NONE: Self = Self(0);
    /// `on_session_start`.
    pub const SESSION_START: Self = Self(1 << 0);
    /// `on_session_end`.
    pub const SESSION_END: Self = Self(1 << 1);
    /// `pre_turn`.
    pub const PRE_TURN: Self = Self(1 << 2);
    /// `post_turn`.
    pub const POST_TURN: Self = Self(1 << 3);
    /// `pre_tool_call`.
    pub const PRE_TOOL: Self = Self(1 << 4);
    /// `post_tool_call`.
    pub const POST_TOOL: Self = Self(1 << 5);
    /// `on_tool_error`.
    pub const ON_TOOL_ERROR: Self = Self(1 << 6);

    /// Every kind the harness can dispatch.
    ///
    /// `on_interaction` and `on_compaction` are absent deliberately: the 0.1.9
    /// `LifecycleHook` enum has no member for either, so they are dispatched
    /// locally only.
    pub const ALL: Self = Self(0b0111_1111);

    /// Whether `other`'s kinds are all present.
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Whether nothing is declared.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The proto enum values, ascending, for `HarnessConfig.enabled_hooks`.
    #[must_use]
    pub fn to_proto(self) -> Vec<i32> {
        [
            (Self::SESSION_START, 1),
            (Self::SESSION_END, 2),
            (Self::PRE_TURN, 3),
            (Self::POST_TURN, 4),
            (Self::PRE_TOOL, 5),
            (Self::POST_TOOL, 6),
            (Self::ON_TOOL_ERROR, 7),
        ]
        .into_iter()
        .filter(|(kind, _)| self.contains(*kind))
        .map(|(_, value)| value)
        .collect()
    }
}

impl std::ops::BitOr for HookKinds {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self {
        Self(self.0 | rhs.0)
    }
}

impl std::ops::BitOrAssign for HookKinds {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

/// Decides whether a turn may start, failing **closed**.
///
/// Dispatched from each transport's `send()`, which is the only place every
/// turn passes through — `Conversation::send`, trigger notifications and
/// `chat()` all funnel into it.
///
/// Deny semantics mirror upstream: the prompt is **not** sent, and the caller
/// gets the hook's message rather than a turn that silently produces nothing.
/// A hook that errors denies too, for the same reason
/// [`HookRunner::gate_tool_call`] denies: a gate that cannot decide must not
/// fall open.
///
/// `None` means no hooks are registered, which is not a failure.
///
/// # Errors
///
/// Returns an error when a hook denies the turn or fails to decide.
pub async fn gate_turn(runner: Option<&HookRunner>) -> Result<(), anyhow::Error> {
    let Some(runner) = runner else {
        return Ok(());
    };
    match runner.dispatch_pre_turn().await {
        Ok(res) if res.allow => Ok(()),
        Ok(res) if res.message.is_empty() => Err(anyhow!("the turn was denied by a pre_turn hook")),
        Ok(res) => Err(anyhow!("{}", res.message)),
        Err(e) => Err(anyhow!(
            "the pre_turn gate could not decide, so the turn was denied: {e}"
        )),
    }
}

#[cfg(test)]
mod kind_tests {
    use super::HookKinds;

    #[test]
    fn nothing_is_declared_by_default() {
        assert!(HookKinds::default().is_empty());
        assert!(HookKinds::default().to_proto().is_empty());
    }

    #[test]
    fn kinds_compose_and_map_to_proto_values() {
        let kinds = HookKinds::PRE_TOOL | HookKinds::SESSION_START;
        assert!(kinds.contains(HookKinds::PRE_TOOL));
        assert!(!kinds.contains(HookKinds::POST_TURN));
        // Ascending proto order, whatever order they were combined in.
        assert_eq!(kinds.to_proto(), vec![1, 5]);
    }

    /// `on_interaction` and `on_compaction` have no `LifecycleHook` member in
    /// 0.1.9, so ALL must not invent values for them.
    #[test]
    fn all_covers_exactly_the_seven_wire_kinds() {
        assert_eq!(HookKinds::ALL.to_proto(), vec![1, 2, 3, 4, 5, 6, 7]);
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::gate_turn;
    use crate::hooks::{Hook, HookRunner};
    use crate::types::HookResult;
    use std::sync::Arc;

    struct DenyingHook(&'static str);

    impl Hook for DenyingHook {
        async fn pre_turn(&self) -> Result<HookResult, anyhow::Error> {
            Ok(HookResult {
                allow: false,
                message: self.0.to_string(),
            })
        }
    }

    struct BrokenHook;

    impl Hook for BrokenHook {
        async fn pre_turn(&self) -> Result<HookResult, anyhow::Error> {
            Err(anyhow::anyhow!("quota lookup failed"))
        }
    }

    #[tokio::test]
    async fn no_runner_allows() {
        assert!(gate_turn(None).await.is_ok());
    }

    #[tokio::test]
    async fn a_denying_hook_reports_its_reason() {
        let runner = HookRunner::new();
        runner
            .register(Arc::new(DenyingHook("out of budget")))
            .await;
        let err = gate_turn(Some(&runner)).await.unwrap_err().to_string();
        assert!(err.contains("out of budget"), "{err}");
    }

    #[tokio::test]
    async fn a_broken_hook_denies() {
        let runner = HookRunner::new();
        runner.register(Arc::new(BrokenHook)).await;
        let err = gate_turn(Some(&runner)).await.unwrap_err().to_string();
        assert!(err.contains("quota lookup failed"), "{err}");
    }
}
