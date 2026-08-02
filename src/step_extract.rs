//! Mapping of harness `StepUpdate` events onto [`ToolCall`]s.
//!
//! Shared by both transports. `src/local.rs` and `src/wasm.rs` each carried a
//! copy of this and had already drifted — the wasm copy was missing
//! `RUN_COMMAND`'s `combined_output`/`exit_code` — so it lives here instead,
//! where a wire fix lands once.

/// Id of the synthetic step that marks an idle transition on the step channel.
///
/// A sentinel value on the same channel as real steps is upstream's design
/// (`IDLE_SENTINEL`), but keying it on the id means a harness step with this
/// literal id would be swallowed. Replacing the channel payload with an enum is
/// the durable fix and is tracked as the remainder of batch A2.
pub const IDLE_SENTINEL_ID: &str = "IDLE_SENTINEL";

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
            serde_json::json!({
                "command_line":    run.command_line,
                "working_dir":     run.working_dir,
                // Include the execution result fields so the frontend
                // can display stdout/stderr instead of "(no output)".
                "combined_output": run.combined_output,
                "exit_code":       run.exit_code,
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
            serde_json::json!({
                "file_path": edit.file_path,
            }),
        )
    } else if let Some(ref search) = step_update.search_directory {
        // The harness puts grep/search results into `step_update.text`.
        // Pack them into `args.output` so the frontend can display them,
        // mirroring how RUN_COMMAND packs `combined_output`.
        (
            "SEARCH_DIR",
            serde_json::json!({
                "directory_path": search.directory_path,
                "query": search.query,
                "num_results": search.num_results,
                // Actual grep results from the harness
                "output": step_update.text,
            }),
        )
    } else if let Some(ref list) = step_update.list_directory {
        (
            "LIST_DIR",
            serde_json::json!({
                "directory_path": list.directory_path,
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

    Some(Step {
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
