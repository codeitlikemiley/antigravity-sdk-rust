//! Structured results for harness-executed built-ins.
//!
//! A `post_tool_call` hook used to receive whatever display text the harness
//! happened to put on the step — so a hook that wanted a command's exit code
//! had to parse prose, and one that wanted the list of files a search matched
//! could not get it at all.
//!
//! Upstream models these per tool (`connections/local/types.py`). This builds
//! the same shapes as JSON, since [`ToolResult::result`] is a
//! `serde_json::Value`: a typed struct per tool would force every hook to match
//! on an enum to reach one field.

use crate::proto::localharness::StepUpdate;
use serde_json::{Value, json};

/// The structured result for a finished built-in step, if it has one.
///
/// `None` when the step is not a recognised built-in, in which case the caller
/// should fall back to the step's display text.
#[must_use]
pub fn structured_result(step_update: &StepUpdate) -> Option<Value> {
    let text = || step_update.text.clone().unwrap_or_default();

    if let Some(ref run) = step_update.run_command {
        // The two fields a hook actually wants — "did it work, and what did it
        // print" — instead of one blob of display text.
        return Some(json!({
            "command_line": run.command_line,
            "working_dir": run.working_dir,
            "combined_output": run.combined_output,
            "exit_code": run.exit_code,
        }));
    }
    if let Some(ref search) = step_update.search_directory {
        return Some(json!({
            "directory_path": search.directory_path,
            "query": search.query,
            "num_results": search.num_results,
            "output": text(),
        }));
    }
    if let Some(ref list) = step_update.list_directory {
        return Some(json!({
            "directory_path": list.directory_path,
            "output": text(),
        }));
    }
    if let Some(ref find) = step_update.find_file {
        return Some(json!({
            "directory_path": find.directory_path,
            "query": find.query,
            "output": text(),
        }));
    }
    if let Some(ref view) = step_update.view_file {
        return Some(json!({
            "file_path": view.file_path,
            "start_line": view.start_line,
            "end_line": view.end_line,
            "contents": text(),
        }));
    }
    if let Some(ref create) = step_update.create_file {
        return Some(json!({ "file_path": create.file_path }));
    }
    if let Some(ref edit) = step_update.edit_file {
        return Some(json!({
            "file_path": edit.file_path,
            "diff_block": edit.diff_block,
        }));
    }
    if let Some(ref search) = step_update.search_web {
        return Some(json!({
            "query": search.query,
            "domain": search.domain,
            "summary": search.summary,
        }));
    }
    if let Some(ref read) = step_update.read_url_content {
        return Some(json!({
            "url": read.url,
            "title": read.title,
            "summary": read.summary,
            "content_path": read.content_path,
        }));
    }
    if let Some(ref image) = step_update.generate_image {
        return Some(json!({
            "prompt": image.prompt,
            "image_paths": image.image_paths,
            "image_name": image.image_name,
        }));
    }
    if let Some(ref finish) = step_update.finish {
        return Some(json!({ "output_string": finish.output_string }));
    }
    None
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::structured_result;
    use crate::proto::localharness::{
        ActionListDirectory, ActionReadUrlContent, ActionRunCommand, StepUpdate,
    };

    #[test]
    fn a_command_reports_its_exit_code_not_prose() {
        let update = StepUpdate {
            run_command: Some(ActionRunCommand {
                command_line: Some("ls -l".to_string()),
                working_dir: Some("/srv".to_string()),
                combined_output: Some("total 0".to_string()),
                exit_code: Some(0),
            }),
            text: Some("ran a command".to_string()),
            ..Default::default()
        };
        let result = structured_result(&update).unwrap();
        assert_eq!(result["exit_code"], 0);
        assert_eq!(result["combined_output"], "total 0");
        assert_eq!(result["command_line"], "ls -l");
    }

    /// Tools whose payload the harness only puts in the display text still get
    /// it, under a named key rather than as the whole result.
    #[test]
    fn display_text_becomes_a_named_field() {
        let update = StepUpdate {
            list_directory: Some(ActionListDirectory {
                directory_path: Some("/srv".to_string()),
                ..Default::default()
            }),
            text: Some("a\nb\nc".to_string()),
            ..Default::default()
        };
        let result = structured_result(&update).unwrap();
        assert_eq!(result["directory_path"], "/srv");
        assert_eq!(result["output"], "a\nb\nc");
    }

    #[test]
    fn a_fetched_url_carries_where_its_content_landed() {
        let update = StepUpdate {
            read_url_content: Some(ActionReadUrlContent {
                url: Some("https://example.test".to_string()),
                title: Some("Example".to_string()),
                summary: Some("a page".to_string()),
                content_path: Some("/tmp/page.md".to_string()),
            }),
            ..Default::default()
        };
        let result = structured_result(&update).unwrap();
        assert_eq!(result["content_path"], "/tmp/page.md");
        assert_eq!(result["title"], "Example");
    }

    #[test]
    fn a_step_with_no_action_has_no_structured_result() {
        let update = StepUpdate {
            text: Some("just talking".to_string()),
            ..Default::default()
        };
        assert!(structured_result(&update).is_none());
    }
}
