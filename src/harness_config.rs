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
