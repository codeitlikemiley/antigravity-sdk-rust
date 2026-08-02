//! Hook plumbing shared by both transports.
//!
//! `src/local.rs` and `src/wasm.rs` are forks of each other, and every piece of
//! hook wiring added to one has historically been missing from the other. The
//! target-neutral half lives here so a dispatch site exists once.

use crate::hooks::HookRunner;
use anyhow::anyhow;

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
