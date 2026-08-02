//! Mapping of harness `StepUpdate` events onto [`ToolCall`]s.
//!
//! Shared by both transports. `src/local.rs` and `src/wasm.rs` each carried a
//! copy of this and had already drifted — the wasm copy was missing
//! `RUN_COMMAND`'s `combined_output`/`exit_code` — so it lives here instead,
//! where a wire fix lands once.

/// What the reader task puts on the step channel.
///
/// Upstream signals idle with a sentinel object on the same queue
/// (`event_processor.IDLE_SENTINEL`). This crate previously used a `Step` whose
/// `id` was the literal `"IDLE_SENTINEL"`, which meant a harness step carrying
/// that id would have been silently swallowed. Making the marker a variant
/// removes the collision by construction.
#[derive(Debug)]
pub enum StepEvent {
    /// A real step. Boxed: `Step` is large and would otherwise set the size of
    /// every value on the channel.
    Step(Box<crate::types::Step>),
    /// A failure to surface to the caller.
    Error(anyhow::Error),
    /// The trajectory reached idle. Not an end-of-stream signal on its own —
    /// steps may already be queued behind it.
    Idle,
}

use crate::proto::localharness::StepUpdate;
use crate::types::ToolCall;

#[allow(clippy::too_many_lines)]
pub fn extract_builtin_tool_call(step_update: &StepUpdate) -> Option<ToolCall> {
    let traj_id = step_update.trajectory_id.clone().unwrap_or_default();
    let step_idx = step_update.step_index.unwrap_or(0);
    let id = format!("{traj_id}_{step_idx}");

    // Each arm builds only `name` and `args`; the tail normalizes wire paths and
    // derives `canonical_path` from them, mirroring upstream's single
    // `_normalize_path_args` + first-match derivation (`hook_router.py:63-74`,
    // `:191-200`). Deriving rather than assigning per-arm keeps the two in sync.
    let (name, mut args): (&str, serde_json::Value) = if step_update.invoke_subagent.is_some() {
        (
            "START_SUBAGENT",
            serde_json::json!({
                "prompt": step_update.request_text.clone().unwrap_or_default()
            }),
        )
    } else if let Some(ref fd) = step_update.find_file {
        (
            "FIND_FILE",
            serde_json::json!({
                "directory_path": fd.directory_path,
                "query": fd.query,
            }),
        )
    } else if let Some(ref run) = step_update.run_command {
        (
            "RUN_COMMAND",
            // Only what the model asked for. `combined_output` and `exit_code`
            // are *results*: a `pre_tool_call` predicate reading them would see
            // them absent, because the command has not run yet, and a rule
            // built on that reads as "always allow". The completed results
            // reach `post_tool_call` through `tool_output::structured_result`.
            serde_json::json!({
                "command_line": run.command_line,
                "working_dir":  run.working_dir,
            }),
        )
    } else if let Some(ref view) = step_update.view_file {
        (
            "VIEW_FILE",
            serde_json::json!({
                "file_path": view.file_path,
                "start_line": view.start_line,
                "end_line": view.end_line,
            }),
        )
    } else if let Some(ref write) = step_update.create_file {
        (
            "CREATE_FILE",
            serde_json::json!({
                "file_path": write.file_path,
                "contents": write.contents,
            }),
        )
    } else if let Some(ref edit) = step_update.edit_file {
        (
            "EDIT_FILE",
            // diff_block is the edit itself. Dropping it meant a policy
            // predicate on EDIT_FILE could see which file was being changed but
            // not what the change was — so a rule like "deny edits that remove
            // a licence header" could not be written at all.
            serde_json::json!({
                "file_path": edit.file_path,
                "diff_block": edit.diff_block,
            }),
        )
    } else if let Some(ref search) = step_update.search_directory {
        // `output` and `num_results` are results, not arguments — same reason
        // as RUN_COMMAND above. They arrive on the `ToolResult` instead.
        (
            "SEARCH_DIR",
            serde_json::json!({
                "directory_path": search.directory_path,
                "query": search.query,
            }),
        )
    } else if let Some(ref list) = step_update.list_directory {
        (
            "LIST_DIR",
            serde_json::json!({
                "directory_path": list.directory_path,
            }),
        )
    } else if let Some(ref search) = step_update.search_web {
        (
            "SEARCH_WEB",
            serde_json::json!({
                "query": search.query,
                "domain": search.domain,
            }),
        )
    } else if let Some(ref read) = step_update.read_url_content {
        (
            "READ_URL_CONTENT",
            serde_json::json!({
                "url": read.url,
            }),
        )
    } else if let Some(ref finish) = step_update.finish {
        // FINISH is a built-in like any other upstream
        // (`_BUILTIN_TOOL_PROTO_FIELDS`), and leaving it out meant the one call
        // that ends a turn and emits structured output was never seen by a
        // policy or a hook. `BuiltinTools::read_only()` already includes it, so
        // the default policy sets allow it.
        (
            "FINISH",
            serde_json::json!({
                "output_string": finish.output_string,
            }),
        )
    } else {
        // Last arm: a step carrying none of these actions is not a tool call.
        let img_gen = step_update.generate_image.as_ref()?;
        (
            "GENERATE_IMAGE",
            serde_json::json!({
                "prompt": img_gen.prompt,
                "image_paths": img_gen.image_paths,
                "image_name": img_gen.image_name,
            }),
        )
    };

    crate::wire_path::normalize_path_args(&mut args);
    let canonical_path = crate::wire_path::canonical_path_from_args(&args);

    Some(ToolCall {
        id,
        name: name.to_string(),
        args,
        canonical_path,
        server_name: None,
    })
}

/// Maps a replayed `StepUpdate` onto a [`Step`].
///
/// Used for the history the harness returns in its handshake reply. This is a
/// narrower mapping than the live reader performs: replayed steps carry no
/// deltas, no in-flight tool state and no usage rollup, so only the fields that
/// survive a round trip are populated.
pub fn step_from_update(step_update: &StepUpdate) -> Option<crate::types::Step> {
    use crate::types::{Step, StepSource, StepStatus, StepTarget, StepType};

    let trajectory_id = step_update.trajectory_id.clone().unwrap_or_default();
    let step_index = step_update.step_index.unwrap_or(0);

    // A finished model step addressed to the user is a complete response. It is
    // what `Conversation::last_response()` looks for, so without this a resumed
    // session reports no last response even with its full history replayed.
    let is_complete_response = step_update.state == Some(2)
        && step_update.source == Some(3)
        && step_update.target == Some(1);

    Some(Step {
        is_complete_response: Some(is_complete_response),
        id: format!("{trajectory_id}_{step_index}"),
        step_index,
        r#type: if step_update.finish.is_some() {
            StepType::Finish
        } else {
            StepType::TextResponse
        },
        source: match step_update.source {
            Some(1) => StepSource::System,
            Some(2) => StepSource::User,
            Some(3) => StepSource::Model,
            _ => StepSource::Unknown,
        },
        target: match step_update.target {
            Some(1) => StepTarget::User,
            _ => StepTarget::Unknown,
        },
        status: match step_update.state {
            Some(1) => StepStatus::Active,
            Some(2) => StepStatus::Done,
            Some(3) => StepStatus::WaitingForUser,
            Some(4) => StepStatus::TerminalError,
            _ => StepStatus::Unknown,
        },
        content: step_update.text.clone().unwrap_or_default(),
        thinking: step_update.thinking.clone().unwrap_or_default(),
        error: step_update.error_message.clone().unwrap_or_default(),
        cascade_id: step_update.cascade_id.clone().unwrap_or_default(),
        trajectory_id,
        ..Default::default()
    })
}

/// Releases a connection's single-consumer claim on the step stream when the
/// stream is dropped.
///
/// `receive_steps()` is called once per turn, so the claim cannot simply be
/// permanent — it has to be handed back when the caller stops reading.
#[derive(Debug)]
pub struct ConsumerGuard(std::sync::Arc<std::sync::atomic::AtomicBool>);

impl ConsumerGuard {
    /// Claims the stream, or returns `None` if another consumer holds it.
    pub fn claim(flag: &std::sync::Arc<std::sync::atomic::AtomicBool>) -> Option<Self> {
        if flag.swap(true, std::sync::atomic::Ordering::SeqCst) {
            None
        } else {
            Some(Self(flag.clone()))
        }
    }
}

impl Drop for ConsumerGuard {
    fn drop(&mut self) {
        self.0.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}

#[cfg(test)]
mod consumer_guard_tests {
    #![allow(clippy::unwrap_used)]
    use super::ConsumerGuard;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn a_second_claim_is_refused_while_the_first_is_alive() {
        let flag = Arc::new(AtomicBool::new(false));
        let first = ConsumerGuard::claim(&flag);
        assert!(first.is_some());
        assert!(ConsumerGuard::claim(&flag).is_none());
        drop(first);
        // Released on drop — this is what lets `receive_steps()` be called once
        // per turn rather than once per connection.
        assert!(ConsumerGuard::claim(&flag).is_some());
    }
}

#[cfg(test)]
mod extractor_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::extract_builtin_tool_call;
    use crate::proto::localharness::{ActionFinish, StepUpdate};

    /// FINISH is the call that ends a turn and emits structured output. It was
    /// the one built-in no policy or hook could see.
    #[test]
    fn finish_is_a_tool_call() {
        let update = StepUpdate {
            trajectory_id: Some("t".to_string()),
            step_index: Some(3),
            finish: Some(ActionFinish {
                output_string: Some("{\"answer\":42}".to_string()),
            }),
            ..Default::default()
        };
        let tc = extract_builtin_tool_call(&update).expect("FINISH should classify as a tool call");
        assert_eq!(tc.name, "FINISH");
        assert_eq!(
            tc.args.get("output_string").and_then(|v| v.as_str()),
            Some("{\"answer\":42}")
        );
        // Nothing path-shaped, so nothing to scope.
        assert!(tc.canonical_path.is_none());
    }

    #[test]
    fn a_step_with_no_action_is_not_a_tool_call() {
        let update = StepUpdate {
            trajectory_id: Some("t".to_string()),
            step_index: Some(1),
            text: Some("just talking".to_string()),
            ..Default::default()
        };
        assert!(extract_builtin_tool_call(&update).is_none());
    }
}
