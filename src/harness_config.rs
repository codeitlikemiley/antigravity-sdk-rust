//! Shared construction of the harness `HarnessConfig`.
//!
//! `src/local.rs` and `src/wasm.rs` each built this independently and drifted
//! apart. Anything both transports must agree on belongs here — the module is
//! ungated, unlike `local` (non-wasm) and `wasm` (wasm or test).

/// Builds `HarnessConfig.models` (field 15) from the crate's `GeminiConfig`.
///
/// Replaces the `gemini_config` oneof that upstream deleted in 0.1.4. Emits two
/// entries — one TEXT, one IMAGE — which is what upstream produces for a
/// default configuration (`local_connection_test.py:3523-3532`). The endpoint
/// sub-message is present even when empty: an env-only setup sends
/// `"geminiApiEndpoint": {}` and the harness reads `GEMINI_API_KEY` from the
/// environment it inherits.
///
/// This keeps the crate's existing public `GeminiConfig` shape. Replacing it
/// with upstream's `ModelTarget` / `ModelEndpoint` graph is WP-4 proper; this
/// gets the *wire* right without a public API break.
pub fn build_models_proto(
    gemini_config: &crate::types::GeminiConfig,
    image_model: Option<&str>,
) -> Vec<crate::proto::localharness::ModelConfig> {
    use crate::proto::localharness::{
        GeminiApiEndpoint, GeminiModelOptions, ModelConfig as ProtoModelConfig, ModelType,
        VertexEndpoint, model_config::Endpoint,
    };

    let thinking_level = gemini_config
        .models
        .default
        .generation
        .thinking_level
        .map(|l| match l {
            crate::types::ThinkingLevel::Minimal => "minimal".to_string(),
            crate::types::ThinkingLevel::Low => "low".to_string(),
            crate::types::ThinkingLevel::Medium => "medium".to_string(),
            crate::types::ThinkingLevel::High => "high".to_string(),
        });
    // Upstream omits `options` entirely when every field is None
    // (local_connection.py:140-146).
    let options = thinking_level.map(|level| GeminiModelOptions {
        thinking_level: Some(level),
    });

    let api_key = gemini_config
        .models
        .default
        .api_key
        .clone()
        .or_else(|| gemini_config.api_key.clone());

    let endpoint = |options: Option<GeminiModelOptions>| {
        if gemini_config.vertex {
            Endpoint::VertexEndpoint(VertexEndpoint {
                base_url: None,
                project: gemini_config.project.clone(),
                location: gemini_config.location.clone(),
                options,
                http_headers: std::collections::HashMap::new(),
            })
        } else {
            Endpoint::GeminiApiEndpoint(GeminiApiEndpoint {
                base_url: None,
                api_key: api_key.clone(),
                options,
                http_headers: std::collections::HashMap::new(),
            })
        }
    };

    vec![
        ProtoModelConfig {
            name: Some(gemini_config.models.default.name.clone()),
            types: vec![ModelType::Text as i32],
            endpoint: Some(endpoint(options)),
        },
        ProtoModelConfig {
            name: Some(image_model.map_or_else(
                || gemini_config.models.image_generation.name.clone(),
                ToString::to_string,
            )),
            types: vec![ModelType::Image as i32],
            endpoint: Some(endpoint(None)),
        },
    ]
}

/// A best-effort OS version string for `ClientInfo.os_version` (proto field 5).
///
/// Upstream sends `platform.release()`. There is no portable equivalent in std,
/// so this reads `uname -r` on unix and falls back to the empty string, which
/// is what the field defaults to anyway.
#[must_use]
pub fn os_version() -> String {
    #[cfg(all(unix, not(target_arch = "wasm32")))]
    {
        std::process::Command::new("uname")
            .arg("-r")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_default()
    }
    #[cfg(not(all(unix, not(target_arch = "wasm32"))))]
    {
        String::new()
    }
}

/// Strips control characters a harness will reject from a user prompt.
///
/// Mirrors upstream `_sanitize_prompt` (`local_connection.py:219-229`, added
/// 0.1.8). Tab, newline and carriage return are kept — they are meaningful in a
/// prompt; the rest of C0, DEL and the C1 range are not.
#[must_use]
pub fn sanitize_prompt(text: &str) -> String {
    text.chars()
        .filter(|c| {
            !matches!(*c, '\u{0}'..='\u{8}' | '\u{b}' | '\u{c}' | '\u{e}'..='\u{1f}' | '\u{7f}'..='\u{9f}')
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_prompt_keeps_meaningful_whitespace() {
        assert_eq!(sanitize_prompt("a\tb\nc\r\nd"), "a\tb\nc\r\nd");
    }

    #[test]
    fn sanitize_prompt_strips_control_characters() {
        assert_eq!(sanitize_prompt("a\u{0}b\u{7}c\u{1f}d\u{7f}e"), "abcde");
        // C1 range, which arrives from mis-decoded input rather than a user.
        assert_eq!(sanitize_prompt("x\u{85}y\u{9f}z"), "xyz");
    }

    #[test]
    fn sanitize_prompt_leaves_ordinary_text_alone() {
        let text = "Hello — こんにちは 🌍";
        assert_eq!(sanitize_prompt(text), text);
    }
}
