//! The one place a `ToolResponse` is built.
//!
//! Both transports constructed this frame in three places each, and the six
//! had already drifted: some wrapped a non-object result, some did not, and
//! none of them ever set `error_message`, so a tool failure reached the harness
//! looking like a successful call whose payload happened to contain the word
//! "error".

use crate::proto::localharness::{InputEvent, ToolResponse};
use crate::types::ToolResult;
use serde_json::Value;

/// Builds the `ToolResponse` frame for a finished tool call.
///
/// - A successful result is sent as a JSON **object**; a bare string, number or
///   array is wrapped under `"result"`, because the harness rejects anything
///   else.
/// - A failure sets `error_message` as well as the payload. The harness uses
///   that field to mark the call failed; without it a failed tool was recorded
///   as a success whose output mentioned an error.
#[must_use]
pub fn tool_response(id: Option<String>, result: &ToolResult) -> ToolResponse {
    let (response_json, error_message) = match (&result.error, &result.result) {
        (Some(error), _) => (
            serde_json::json!({ "error": error }).to_string(),
            Some(error.clone()),
        ),
        (None, Some(value)) if value.is_object() => (
            serde_json::to_string(value).unwrap_or_else(|_| "{}".to_string()),
            None,
        ),
        (None, Some(value)) => (serde_json::json!({ "result": value }).to_string(), None),
        (None, None) => ("{}".to_string(), None),
    };

    ToolResponse {
        id,
        response_json: Some(response_json),
        error_message,
        supplemental_media: Vec::new(),
    }
}

/// The same frame, wrapped and serialized ready for the socket.
#[must_use]
pub fn tool_response_frame(id: Option<String>, result: &ToolResult) -> Option<String> {
    let event = InputEvent {
        event: Some(
            crate::proto::localharness::input_event::Event::ToolResponse(tool_response(id, result)),
        ),
    };
    serde_json::to_string(&event).ok()
}

/// The response for a call that was refused before it ran.
#[must_use]
pub fn denied_response(id: Option<String>, reason: &str) -> ToolResponse {
    tool_response(
        id,
        &ToolResult {
            error: Some(reason.to_string()),
            ..Default::default()
        },
    )
}

/// Parses the arguments the model supplied.
///
/// Absent or empty is an empty object, not null: upstream does
/// `json.loads(arguments_json or "{}")`, and a tool reading `args["x"]` got a
/// type error rather than a missing key.
#[must_use]
pub fn parse_arguments(arguments_json: Option<&str>) -> Value {
    let raw = arguments_json.unwrap_or("").trim();
    if raw.is_empty() {
        return Value::Object(serde_json::Map::new());
    }
    serde_json::from_str(raw).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::{denied_response, parse_arguments, tool_response};
    use crate::types::ToolResult;

    fn ok_result(value: serde_json::Value) -> ToolResult {
        ToolResult {
            name: "lookup".to_string(),
            result: Some(value),
            ..Default::default()
        }
    }

    #[test]
    fn a_failure_sets_error_message() {
        let failed = ToolResult {
            name: "lookup".to_string(),
            error: Some("row not found".to_string()),
            ..Default::default()
        };
        let response = tool_response(Some("1".to_string()), &failed);
        assert_eq!(response.error_message.as_deref(), Some("row not found"));
        assert!(response.response_json.unwrap().contains("row not found"));
    }

    #[test]
    fn a_bare_value_is_wrapped_in_an_object() {
        let response = tool_response(None, &ok_result(serde_json::json!(42)));
        assert_eq!(response.response_json.as_deref(), Some(r#"{"result":42}"#));
        assert!(response.error_message.is_none());
    }

    #[test]
    fn an_object_result_is_sent_as_is() {
        let response = tool_response(None, &ok_result(serde_json::json!({"rows": 3})));
        assert_eq!(response.response_json.as_deref(), Some(r#"{"rows":3}"#));
    }

    #[test]
    fn an_empty_result_is_an_empty_object() {
        let response = tool_response(None, &ToolResult::default());
        assert_eq!(response.response_json.as_deref(), Some("{}"));
    }

    #[test]
    fn a_denial_reads_as_a_failure() {
        let response = denied_response(Some("7".to_string()), "denied by policy");
        assert_eq!(response.error_message.as_deref(), Some("denied by policy"));
    }

    #[test]
    fn absent_arguments_parse_to_an_empty_object() {
        assert_eq!(parse_arguments(None), serde_json::json!({}));
        assert_eq!(parse_arguments(Some("")), serde_json::json!({}));
        assert_eq!(parse_arguments(Some("  ")), serde_json::json!({}));
        assert_eq!(
            parse_arguments(Some(r#"{"a":1}"#)),
            serde_json::json!({"a":1})
        );
    }
}
