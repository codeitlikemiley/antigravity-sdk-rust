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

/// Answers a harness-side `CallHookRequest`.
///
/// The harness **blocks its turn** until it gets a `CallHookResponse` carrying
/// the matching `request_id`. Every path through this function therefore
/// produces one — including the unreachable ones. A request this side does not
/// understand is answered with `error_message`, which the harness treats as a
/// hook failure; not answering at all is a deadlock.
///
/// Deny semantics match the local gates: a hook that errors refuses, because a
/// gate that cannot decide must not fall open.
pub async fn answer_hook_request(
    runner: Option<&HookRunner>,
    request: &crate::proto::localharness::CallHookRequest,
) -> crate::proto::localharness::CallHookResponse {
    use crate::proto::localharness::{
        CallHookResponse, EmptyResult, OnToolErrorResult, PreToolResult, PreTurnResult,
        call_hook_request::Args, call_hook_response::Result as ResponseResult, pre_tool_result,
        pre_turn_result,
    };

    let request_id = request.request_id.clone();
    let answer = |result: ResponseResult| CallHookResponse {
        request_id: request_id.clone(),
        result: Some(result),
    };

    let Some(runner) = runner else {
        // No hooks registered at all: nothing to object, and the turn must not
        // stall waiting for an opinion that does not exist.
        return answer(ResponseResult::EmptyResult(EmptyResult {}));
    };

    match request.args.as_ref() {
        Some(Args::PreTurnArgs(_)) => {
            let (decision, reason) = match gate_turn(Some(runner)).await {
                Ok(()) => (pre_turn_result::Decision::Allow, String::new()),
                Err(e) => (pre_turn_result::Decision::Deny, e.to_string()),
            };
            answer(ResponseResult::PreTurnResult(PreTurnResult {
                decision: Some(decision as i32),
                reason: Some(reason),
            }))
        }
        Some(Args::PreToolArgs(args)) => {
            let tool_call = crate::types::ToolCall {
                id: request.request_id.clone().unwrap_or_default(),
                name: args.tool_name.clone().unwrap_or_default(),
                args: crate::tool_wire::parse_arguments(args.arguments_json.as_deref()),
                canonical_path: None,
                server_name: args.server_name.clone(),
            };
            let (allow, reason) = HookRunner::gate_tool_call(Some(runner), &tool_call).await;
            answer(ResponseResult::PreToolResult(PreToolResult {
                decision: Some(if allow {
                    pre_tool_result::Decision::Allow as i32
                } else {
                    pre_tool_result::Decision::Deny as i32
                }),
                reason: Some(reason),
                // Rewriting the model's arguments is a capability this side does
                // not offer; sending the field back unchanged would be a lie
                // about having considered it.
                modified_arguments_json: None,
            }))
        }
        Some(Args::PostToolArgs(args)) => {
            let result = crate::types::ToolResult {
                name: args.tool_name.clone().unwrap_or_default(),
                id: request.request_id.clone(),
                result: args.result.clone().map(serde_json::Value::String),
                error: args.error.clone().filter(|e| !e.is_empty()),
                server_name: args.server_name.clone(),
                exception: None,
            };
            if let Err(e) = runner.dispatch_post_tool_call(&result).await {
                tracing::error!("post_tool_call hook failed: {e:?}");
            }
            answer(ResponseResult::EmptyResult(EmptyResult {}))
        }
        Some(Args::PostTurnArgs(args)) => {
            let text = args.response_text.clone().unwrap_or_default();
            if let Err(e) = runner.dispatch_post_turn(&text).await {
                tracing::error!("post_turn hook failed: {e:?}");
            }
            answer(ResponseResult::EmptyResult(EmptyResult {}))
        }
        Some(Args::OnToolErrorArgs(args)) => {
            let error = anyhow::anyhow!(
                "{}",
                args.error_message
                    .clone()
                    .unwrap_or_else(|| "tool failed".to_string())
            );
            let replacement = runner.dispatch_on_tool_error(&error).await;
            answer(ResponseResult::OnToolErrorResult(OnToolErrorResult {
                custom_error_message: replacement,
            }))
        }
        None => answer(ResponseResult::ErrorMessage(format!(
            "hook request {:?} carried no arguments this SDK understands",
            request.r#type
        ))),
    }
}

#[cfg(test)]
mod router_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::answer_hook_request;
    use crate::hooks::{Hook, HookRunner};
    use crate::proto::localharness::{
        CallHookRequest, OnToolErrorArgs, PostTurnArgs, PreToolArgs, PreTurnArgs,
        call_hook_request::Args, call_hook_response::Result as ResponseResult, pre_tool_result,
        pre_turn_result,
    };
    use crate::types::{HookResult, ToolCall};
    use std::sync::Arc;

    fn request(args: Args) -> CallHookRequest {
        CallHookRequest {
            request_id: Some("r1".to_string()),
            name: Some("h".to_string()),
            r#type: None,
            args: Some(args),
        }
    }

    struct Denier;

    impl Hook for Denier {
        async fn pre_tool_call(&self, _tool_call: &ToolCall) -> Result<HookResult, anyhow::Error> {
            Ok(HookResult {
                allow: false,
                message: "not on my watch".to_string(),
            })
        }
        async fn pre_turn(&self) -> Result<HookResult, anyhow::Error> {
            Err(anyhow::anyhow!("cannot decide"))
        }
        async fn on_tool_error(
            &self,
            _error: &anyhow::Error,
        ) -> Result<Option<String>, anyhow::Error> {
            Ok(Some("try fewer rows".to_string()))
        }
    }

    /// Every path must answer, and every answer must carry the request id the
    /// harness is blocking on.
    #[tokio::test]
    async fn every_request_is_answered_with_its_id() {
        let runner = HookRunner::new();
        runner.register(Arc::new(Denier)).await;

        for args in [
            Args::PreTurnArgs(PreTurnArgs { user_input: None }),
            Args::PreToolArgs(PreToolArgs {
                tool_name: Some("RUN_COMMAND".to_string()),
                arguments_json: None,
                server_name: None,
            }),
            Args::PostTurnArgs(PostTurnArgs {
                response_text: Some("done".to_string()),
            }),
            Args::OnToolErrorArgs(OnToolErrorArgs {
                tool_name: Some("lookup".to_string()),
                error_message: Some("boom".to_string()),
                server_name: None,
            }),
        ] {
            let response = answer_hook_request(Some(&runner), &request(args)).await;
            assert_eq!(response.request_id.as_deref(), Some("r1"));
            assert!(response.result.is_some(), "an unanswered request deadlocks");
        }
    }

    #[tokio::test]
    async fn a_denied_tool_call_comes_back_as_deny_with_its_reason() {
        let runner = HookRunner::new();
        runner.register(Arc::new(Denier)).await;
        let response = answer_hook_request(
            Some(&runner),
            &request(Args::PreToolArgs(PreToolArgs {
                tool_name: Some("RUN_COMMAND".to_string()),
                arguments_json: None,
                server_name: None,
            })),
        )
        .await;
        match response.result.unwrap() {
            ResponseResult::PreToolResult(r) => {
                assert_eq!(r.decision, Some(pre_tool_result::Decision::Deny as i32));
                assert_eq!(r.reason.as_deref(), Some("not on my watch"));
            }
            other => panic!("unexpected result {other:?}"),
        }
    }

    /// A gate that cannot decide refuses, matching the local gates.
    #[tokio::test]
    async fn a_failing_pre_turn_hook_denies_the_turn() {
        let runner = HookRunner::new();
        runner.register(Arc::new(Denier)).await;
        let response = answer_hook_request(
            Some(&runner),
            &request(Args::PreTurnArgs(PreTurnArgs { user_input: None })),
        )
        .await;
        match response.result.unwrap() {
            ResponseResult::PreTurnResult(r) => {
                assert_eq!(r.decision, Some(pre_turn_result::Decision::Deny as i32));
                assert!(r.reason.unwrap().contains("cannot decide"));
            }
            other => panic!("unexpected result {other:?}"),
        }
    }

    #[tokio::test]
    async fn an_unrecognised_request_is_answered_with_an_error() {
        let runner = HookRunner::new();
        let response = answer_hook_request(
            Some(&runner),
            &CallHookRequest {
                request_id: Some("r9".to_string()),
                name: None,
                r#type: None,
                args: None,
            },
        )
        .await;
        assert_eq!(response.request_id.as_deref(), Some("r9"));
        assert!(matches!(
            response.result.unwrap(),
            ResponseResult::ErrorMessage(_)
        ));
    }

    /// No hooks registered is not a failure: answer empty rather than stall.
    #[tokio::test]
    async fn no_runner_still_answers() {
        let response = answer_hook_request(
            None,
            &request(Args::PreTurnArgs(PreTurnArgs { user_input: None })),
        )
        .await;
        assert!(matches!(
            response.result.unwrap(),
            ResponseResult::EmptyResult(_)
        ));
    }
}
