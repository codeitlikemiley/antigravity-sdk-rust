//! Shared construction of the harness `HarnessConfig`.
//!
//! `src/local.rs` and `src/wasm.rs` each built this independently and drifted
//! apart. Anything both transports must agree on belongs here — the module is
//! ungated, unlike `local` (non-wasm) and `wasm` (wasm or test).

/// Builds `HarnessConfig.models` (field 15) from the crate's `GeminiConfig`.
///
/// Implements upstream's `_merge_models_list`
/// (`local_connection_config.py:268-296`): explicit `model_targets` first, then
/// the shorthand model, then the defaults — and a default is appended **only**
/// if none of its model types is already covered. Deduplication is by
/// [`ModelType`], never by name: two text models are legal.
///
/// The shorthand endpoint (Vertex when `vertex` is set, otherwise the Gemini
/// API) attaches to the shorthand model and to the defaults, never to an
/// explicit target — an explicit target must carry its own.
///
/// # Errors
///
/// Returns an error if an explicit target has no endpoint.
pub fn build_models_proto(
    gemini_config: &crate::types::GeminiConfig,
    image_model: Option<&str>,
) -> Result<Vec<crate::proto::localharness::ModelConfig>, anyhow::Error> {
    use crate::types::{GeminiModelOptions, ModelEndpoint, ModelTarget, ModelType};

    let options = gemini_config
        .models
        .default
        .generation
        .thinking_level
        .map(|thinking_level| GeminiModelOptions {
            thinking_level: Some(thinking_level),
        })
        .filter(|o| !o.is_empty());

    // Never the environment: upstream treats `GEMINI_API_KEY` as a presence
    // check and lets the harness read it from the environment it inherits, so
    // the key stays out of the config frame.
    let api_key = gemini_config
        .models
        .default
        .api_key
        .clone()
        .or_else(|| gemini_config.api_key.clone());

    let shorthand_endpoint = |options: Option<GeminiModelOptions>| {
        if gemini_config.vertex {
            ModelEndpoint::Vertex {
                base_url: None,
                http_headers: std::collections::HashMap::new(),
                project: gemini_config
                    .project
                    .clone()
                    .or_else(|| std::env::var("GOOGLE_CLOUD_PROJECT").ok()),
                location: gemini_config
                    .location
                    .clone()
                    .or_else(|| std::env::var("GOOGLE_CLOUD_LOCATION").ok()),
                options,
            }
        } else {
            ModelEndpoint::GeminiApi {
                base_url: None,
                http_headers: std::collections::HashMap::new(),
                api_key: api_key.clone(),
                options,
            }
        }
    };

    let mut merged: Vec<ModelTarget> = Vec::new();
    for target in &gemini_config.model_targets {
        if target.endpoint.is_none() {
            return Err(anyhow::anyhow!(
                "the model target `{}` has no endpoint; an explicitly supplied target must carry \
                 one, because the api_key/vertex shorthand only attaches to the shorthand and \
                 default models",
                target.name.as_deref().unwrap_or("<unnamed>")
            ));
        }
        merged.push(target.clone());
    }

    // The shorthand text model.
    merged.push(ModelTarget {
        name: Some(gemini_config.models.default.name.clone()),
        types: vec![ModelType::Text],
        endpoint: Some(shorthand_endpoint(options)),
    });

    // Defaults fill only the types nothing above covers.
    let image_name = image_model.map_or_else(
        || gemini_config.models.image_generation.name.clone(),
        ToString::to_string,
    );
    for default in [
        ModelTarget {
            name: Some(gemini_config.models.default.name.clone()),
            types: vec![ModelType::Text],
            endpoint: Some(shorthand_endpoint(None)),
        },
        ModelTarget {
            name: Some(image_name),
            types: vec![ModelType::Image],
            endpoint: Some(shorthand_endpoint(None)),
        },
    ] {
        let covered: std::collections::HashSet<ModelType> = merged
            .iter()
            .flat_map(|t| t.types.iter().copied())
            .collect();
        if default.types.iter().any(|t| covered.contains(t)) {
            continue;
        }
        merged.push(default);
    }

    Ok(merged.iter().map(to_proto).collect())
}

/// Maps one [`ModelTarget`](crate::types::ModelTarget) onto its proto form.
fn to_proto(target: &crate::types::ModelTarget) -> crate::proto::localharness::ModelConfig {
    use crate::proto::localharness::{
        GeminiApiEndpoint, GeminiModelOptions as ProtoOptions, GemmaEndpoint,
        ModelConfig as ProtoModelConfig, VertexEndpoint, model_config::Endpoint,
    };
    use crate::types::ModelEndpoint;

    let proto_options = |options: &Option<crate::types::GeminiModelOptions>| {
        options
            .as_ref()
            .filter(|o| !o.is_empty())
            .map(|o| ProtoOptions {
                thinking_level: o.thinking_level.map(|l| l.as_str().to_string()),
            })
    };

    let endpoint = target.endpoint.as_ref().map(|endpoint| match endpoint {
        ModelEndpoint::GeminiApi {
            base_url,
            http_headers,
            api_key,
            options,
        } => Endpoint::GeminiApiEndpoint(GeminiApiEndpoint {
            base_url: base_url.clone(),
            http_headers: http_headers.clone(),
            api_key: api_key.clone(),
            options: proto_options(options),
        }),
        ModelEndpoint::Vertex {
            base_url,
            http_headers,
            project,
            location,
            options,
        } => Endpoint::VertexEndpoint(VertexEndpoint {
            base_url: base_url.clone(),
            http_headers: http_headers.clone(),
            project: project.clone(),
            location: location.clone(),
            options: proto_options(options),
        }),
        ModelEndpoint::Gemma { base_url } => Endpoint::GemmaEndpoint(GemmaEndpoint {
            base_url: Some(base_url.clone()),
        }),
    });

    ProtoModelConfig {
        name: Some(target.name.clone().unwrap_or_default()),
        types: target.types.iter().map(|t| t.as_proto()).collect(),
        endpoint,
    }
}

/// Whether the environment asks for the Vertex backend.
///
/// Upstream reads both names and accepts `"true"` or `"1"`
/// (`local_connection_config.py:206-211`, 0.1.7). This crate read neither, so a
/// caller whose environment selected Vertex silently got the Gemini API.
#[must_use]
pub fn vertex_from_env() -> bool {
    ["GOOGLE_GENAI_USE_VERTEXAI", "GOOGLE_GENAI_USE_ENTERPRISE"]
        .iter()
        .filter_map(|name| std::env::var(name).ok())
        .any(|value| {
            let value = value.trim().to_ascii_lowercase();
            value == "true" || value == "1"
        })
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

#[cfg(test)]
mod model_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::build_models_proto;
    use crate::proto::localharness::model_config::Endpoint;
    use crate::types::{
        GeminiConfig, GenerationConfig, ModelEndpoint, ModelTarget, ModelType, ThinkingLevel,
    };

    fn types_of(config: &crate::proto::localharness::ModelConfig) -> Vec<i32> {
        config.types.clone()
    }

    #[test]
    fn a_default_config_emits_one_text_and_one_image_model() {
        let models = build_models_proto(&GeminiConfig::default(), None).unwrap();
        assert_eq!(models.len(), 2);
        assert_eq!(types_of(&models[0]), vec![ModelType::Text.as_proto()]);
        assert_eq!(types_of(&models[1]), vec![ModelType::Image.as_proto()]);
        // The endpoint is present but empty: the harness reads GEMINI_API_KEY
        // from the environment it inherits.
        match models[0].endpoint.as_ref().unwrap() {
            Endpoint::GeminiApiEndpoint(e) => assert!(e.api_key.is_none()),
            other => panic!("unexpected endpoint {other:?}"),
        }
    }

    #[test]
    fn an_explicit_key_reaches_both_entries() {
        let config = GeminiConfig {
            api_key: Some("k".to_string()),
            ..Default::default()
        };
        let models = build_models_proto(&config, None).unwrap();
        for model in &models {
            match model.endpoint.as_ref().unwrap() {
                Endpoint::GeminiApiEndpoint(e) => {
                    // Only the text model carries the shorthand's options; the
                    // key is on both.
                    assert_eq!(e.api_key.as_deref(), Some("k"));
                }
                other => panic!("unexpected endpoint {other:?}"),
            }
        }
    }

    #[test]
    fn vertex_selects_the_vertex_endpoint() {
        let config = GeminiConfig {
            vertex: true,
            project: Some("p".to_string()),
            location: Some("l".to_string()),
            ..Default::default()
        };
        let models = build_models_proto(&config, None).unwrap();
        match models[0].endpoint.as_ref().unwrap() {
            Endpoint::VertexEndpoint(e) => {
                assert_eq!(e.project.as_deref(), Some("p"));
                assert_eq!(e.location.as_deref(), Some("l"));
            }
            other => panic!("unexpected endpoint {other:?}"),
        }
    }

    /// `options` is omitted entirely when every field is unset, and carries the
    /// per-variant spelling — `extra_high`, not `extrahigh`.
    #[test]
    fn thinking_level_rides_on_options() {
        let mut config = GeminiConfig::default();
        config.models.default.generation = GenerationConfig {
            thinking_level: Some(ThinkingLevel::ExtraHigh),
        };
        let models = build_models_proto(&config, None).unwrap();
        match models[0].endpoint.as_ref().unwrap() {
            Endpoint::GeminiApiEndpoint(e) => assert_eq!(
                e.options.as_ref().unwrap().thinking_level.as_deref(),
                Some("extra_high")
            ),
            other => panic!("unexpected endpoint {other:?}"),
        }
        // The image entry has no options at all.
        match models[1].endpoint.as_ref().unwrap() {
            Endpoint::GeminiApiEndpoint(e) => assert!(e.options.is_none()),
            other => panic!("unexpected endpoint {other:?}"),
        }
    }

    /// An explicit IMAGE target suppresses the default image model, and the
    /// text default is appended after it.
    #[test]
    fn an_explicit_target_covers_its_type() {
        let config = GeminiConfig {
            model_targets: vec![ModelTarget {
                name: Some("my-image-model".to_string()),
                types: vec![ModelType::Image],
                endpoint: Some(ModelEndpoint::Gemma {
                    base_url: "http://localhost:11434".to_string(),
                }),
            }],
            ..Default::default()
        };
        let models = build_models_proto(&config, None).unwrap();
        assert_eq!(models.len(), 2, "no default image model is appended");
        assert_eq!(models[0].name.as_deref(), Some("my-image-model"));
        assert_eq!(types_of(&models[1]), vec![ModelType::Text.as_proto()]);
        assert!(matches!(
            models[0].endpoint.as_ref().unwrap(),
            Endpoint::GemmaEndpoint(_)
        ));
    }

    /// Dedupe is by model type, never by name: two text models are legal.
    #[test]
    fn two_text_models_are_legal() {
        let config = GeminiConfig {
            model_targets: vec![ModelTarget {
                name: Some("gemini-3.6-flash".to_string()),
                types: vec![ModelType::Text],
                endpoint: Some(ModelEndpoint::GeminiApi {
                    base_url: None,
                    http_headers: std::collections::HashMap::new(),
                    api_key: None,
                    options: None,
                }),
            }],
            ..Default::default()
        };
        let models = build_models_proto(&config, None).unwrap();
        let text_models = models
            .iter()
            .filter(|m| m.types.contains(&ModelType::Text.as_proto()))
            .count();
        assert_eq!(text_models, 2, "the shorthand text model is kept too");
    }

    #[test]
    fn an_explicit_target_without_an_endpoint_is_an_error() {
        let config = GeminiConfig {
            model_targets: vec![ModelTarget {
                name: Some("orphan".to_string()),
                ..Default::default()
            }],
            ..Default::default()
        };
        let err = build_models_proto(&config, None).expect_err("an endpoint is required");
        assert!(err.to_string().contains("no endpoint"), "{err}");
    }
}

/// Maps the crate's MCP server configs onto `HarnessConfig.mcp_servers`
/// (field 14).
///
/// The builder accepted MCP servers, both strategies stored them, and nothing
/// ever put them on the wire — so `mcp_server(...)` was a no-op and the model
/// never saw a single MCP tool (audit CT-011).
///
/// SSE and HTTP both map to `McpHttpTransport`: the 0.1.9 proto has one HTTP
/// transport, and the harness negotiates the streaming style itself.
#[must_use]
pub fn build_mcp_servers_proto(
    servers: &[crate::types::McpServerConfig],
) -> Vec<crate::proto::localharness::McpServerConfig> {
    use crate::proto::localharness::{
        McpHttpTransport, McpServerConfig as ProtoMcp, McpStdioTransport, mcp_server_config,
    };
    use crate::types::McpServerConfig;

    servers
        .iter()
        .map(|server| match server {
            McpServerConfig::Stdio {
                name,
                command,
                args,
                enabled_tools,
                disabled_tools,
                env,
                timeout_seconds,
            } => ProtoMcp {
                name: Some(name.clone()),
                enabled_tools: enabled_tools.clone().unwrap_or_default(),
                disabled_tools: disabled_tools.clone().unwrap_or_default(),
                auth_provider_type: None,
                timeout_seconds: *timeout_seconds,
                transport: Some(mcp_server_config::Transport::Stdio(McpStdioTransport {
                    command: Some(command.clone()),
                    args: args.clone(),
                    env: env.clone(),
                })),
            },
            McpServerConfig::Sse {
                name,
                url,
                headers,
                enabled_tools,
                disabled_tools,
                timeout_seconds,
            } => ProtoMcp {
                name: Some(name.clone()),
                enabled_tools: enabled_tools.clone().unwrap_or_default(),
                disabled_tools: disabled_tools.clone().unwrap_or_default(),
                auth_provider_type: None,
                timeout_seconds: *timeout_seconds,
                transport: Some(mcp_server_config::Transport::Http(McpHttpTransport {
                    url: Some(url.clone()),
                    headers: headers.clone().unwrap_or_default(),
                })),
            },
            McpServerConfig::Http {
                name,
                url,
                headers,
                enabled_tools,
                disabled_tools,
                timeout,
                ..
            } => ProtoMcp {
                name: Some(name.clone()),
                enabled_tools: enabled_tools.clone().unwrap_or_default(),
                disabled_tools: disabled_tools.clone().unwrap_or_default(),
                auth_provider_type: None,
                // The proto's granularity is whole seconds; the crate's HTTP
                // variant has carried a float since before this field existed.
                #[allow(clippy::cast_possible_truncation)]
                timeout_seconds: Some(*timeout as i32),
                transport: Some(mcp_server_config::Transport::Http(McpHttpTransport {
                    url: Some(url.clone()),
                    headers: headers.clone().unwrap_or_default(),
                })),
            },
        })
        .collect()
}

#[cfg(test)]
mod mcp_tests {
    #![allow(clippy::unwrap_used, clippy::panic)]
    use super::build_mcp_servers_proto;
    use crate::proto::localharness::mcp_server_config::Transport;
    use crate::types::McpServerConfig;

    #[test]
    fn a_stdio_server_carries_its_command_env_and_timeout() {
        let servers = vec![McpServerConfig::Stdio {
            name: "github".to_string(),
            command: "mcp-github".to_string(),
            args: vec!["--stdio".to_string()],
            enabled_tools: Some(vec!["create_issue".to_string()]),
            disabled_tools: None,
            env: std::iter::once(("TOKEN".to_string(), "abc".to_string())).collect(),
            timeout_seconds: Some(30),
        }];
        let proto = build_mcp_servers_proto(&servers);
        assert_eq!(proto.len(), 1);
        assert_eq!(proto[0].name.as_deref(), Some("github"));
        assert_eq!(proto[0].enabled_tools, vec!["create_issue".to_string()]);
        assert_eq!(proto[0].timeout_seconds, Some(30));
        match proto[0].transport.as_ref().unwrap() {
            Transport::Stdio(t) => {
                assert_eq!(t.command.as_deref(), Some("mcp-github"));
                assert_eq!(t.env.get("TOKEN").map(String::as_str), Some("abc"));
            }
            other @ Transport::Http(_) => panic!("unexpected transport {other:?}"),
        }
    }

    /// The 0.1.9 proto has one HTTP transport; the harness negotiates the
    /// streaming style, so SSE and HTTP map to the same frame.
    #[test]
    fn sse_and_http_both_map_to_the_http_transport() {
        for server in [
            McpServerConfig::Sse {
                name: "s".to_string(),
                url: "https://example.test/sse".to_string(),
                headers: None,
                enabled_tools: None,
                disabled_tools: None,
                timeout_seconds: None,
            },
            McpServerConfig::Http {
                name: "h".to_string(),
                url: "https://example.test/mcp".to_string(),
                headers: None,
                enabled_tools: None,
                disabled_tools: None,
                timeout: 30.0,
                sse_read_timeout: 300.0,
                terminate_on_close: true,
            },
        ] {
            let proto = build_mcp_servers_proto(&[server]);
            assert!(matches!(
                proto[0].transport.as_ref().unwrap(),
                Transport::Http(_)
            ));
        }
    }
}

/// Builds `HarnessConfig.custom_subagents` (field 17).
///
/// Ports upstream's three validations (`local_connection.py:884-936`):
///
/// 1. capabilities default to [`BuiltinTools::read_only`] when neither list is
///    given — a subagent that inherits everything is not what "default" should
///    mean;
/// 2. `START_SUBAGENT` is dropped with a warning, because a subagent spawning
///    subagents is not supported by the harness;
/// 3. naming a client-side tool the main agent has not registered is an error,
///    not a subagent that silently cannot call it.
///
/// # Errors
///
/// Returns an error if both capability lists are set on one subagent, or if a
/// named tool is not registered.
pub fn build_custom_subagents_proto(
    subagents: &[crate::types::SubagentConfig],
    registered_tools: &[String],
) -> Result<Vec<crate::proto::localharness::CustomAgent>, anyhow::Error> {
    use crate::proto::localharness::{CustomAgent, SystemInstructions as ProtoSystem};
    use crate::types::BuiltinTools;

    let mut built = Vec::with_capacity(subagents.len());
    for subagent in subagents {
        let capabilities = &subagent.capabilities;
        if capabilities.enabled_tools.is_some() && capabilities.disabled_tools.is_some() {
            return Err(anyhow::anyhow!(
                "subagent `{}` sets both enabled_tools and disabled_tools; they are mutually \
                 exclusive",
                subagent.name
            ));
        }

        let mut tools: Vec<BuiltinTools> =
            match (&capabilities.enabled_tools, &capabilities.disabled_tools) {
                (Some(enabled), _) => enabled.clone(),
                (None, Some(disabled)) => BuiltinTools::all_tools()
                    .into_iter()
                    .filter(|t| !disabled.contains(t))
                    .collect(),
                (None, None) => BuiltinTools::read_only(),
            };

        if tools.contains(&BuiltinTools::StartSubagent) {
            tracing::warn!(
                "subagent `{}` requested START_SUBAGENT; nested subagents are not supported and \
                 the tool has been dropped",
                subagent.name
            );
            tools.retain(|t| *t != BuiltinTools::StartSubagent);
        }

        for tool in &subagent.tools {
            if !registered_tools.iter().any(|name| name == tool) {
                return Err(anyhow::anyhow!(
                    "subagent `{}` names the tool `{tool}`, which is not registered on the agent",
                    subagent.name
                ));
            }
        }

        built.push(CustomAgent {
            name: Some(subagent.name.clone()),
            description: Some(subagent.description.clone()),
            system_instructions: subagent.system_instructions.as_ref().map(|text| ProtoSystem {
                r#type: Some(
                    crate::proto::localharness::system_instructions::Type::Custom(
                        crate::proto::localharness::CustomSystemInstructions {
                            part: vec![crate::proto::localharness::custom_system_instructions::Part {
                                part: Some(
                                    crate::proto::localharness::custom_system_instructions::part::Part::Text(text.clone()),
                                ),
                            }],
                        },
                    ),
                ),
            }),
            harness_side_tools: Some(harness_side_tools_for(&tools)),
            // Client-side tools are declared once on the main agent; the
            // harness routes a subagent's call back through the same channel.
            tools: Vec::new(),
        });
    }
    Ok(built)
}

/// The `HarnessSideTools` toggles for a given set of built-ins.
///
/// `subagents` is always off here: nested subagents are not supported, which is
/// validation 2 of [`build_custom_subagents_proto`] expressed on the wire.
fn harness_side_tools_for(
    tools: &[crate::types::BuiltinTools],
) -> crate::proto::localharness::HarnessSideTools {
    use crate::proto::localharness::{
        FileEditToolConfig, FindToolConfig, GenerateImageToolConfig, GrepSearchToolConfig,
        HarnessSideTools, ListDirToolConfig, ReadUrlContentToolConfig, RunCommandToolConfig,
        SearchWebToolConfig, SubagentsConfig, UserQuestionsConfig, ViewFileToolConfig,
        WriteToFileToolConfig,
    };
    use crate::types::BuiltinTools;

    let on = |tool: BuiltinTools| Some(tools.contains(&tool));

    HarnessSideTools {
        find: Some(FindToolConfig {
            enabled: on(BuiltinTools::FindFile),
        }),
        run_command: Some(RunCommandToolConfig {
            enabled: on(BuiltinTools::RunCommand),
        }),
        subagents: Some(SubagentsConfig {
            enabled: Some(false),
        }),
        user_questions: Some(UserQuestionsConfig {
            enabled: on(BuiltinTools::AskQuestion),
        }),
        file_edit: Some(FileEditToolConfig {
            enabled: on(BuiltinTools::EditFile),
        }),
        view_file: Some(ViewFileToolConfig {
            enabled: on(BuiltinTools::ViewFile),
        }),
        write_to_file: Some(WriteToFileToolConfig {
            enabled: on(BuiltinTools::CreateFile),
        }),
        grep_search: Some(GrepSearchToolConfig {
            enabled: on(BuiltinTools::SearchDir),
        }),
        list_dir: Some(ListDirToolConfig {
            enabled: on(BuiltinTools::ListDir),
        }),
        permissions: None,
        generate_image: Some(GenerateImageToolConfig {
            enabled: on(BuiltinTools::GenerateImage),
        }),
        search_web: Some(SearchWebToolConfig {
            enabled: on(BuiltinTools::SearchWeb),
        }),
        read_url_content: Some(ReadUrlContentToolConfig {
            enabled: on(BuiltinTools::ReadUrlContent),
        }),
        tool_search_config: None,
    }
}

#[cfg(test)]
mod subagent_tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::build_custom_subagents_proto;
    use crate::types::{BuiltinTools, SubagentCapabilities, SubagentConfig};

    fn reviewer() -> SubagentConfig {
        SubagentConfig {
            name: "reviewer".to_string(),
            description: "reviews a diff".to_string(),
            ..Default::default()
        }
    }

    /// Capabilities default to the read-only built-ins. A subagent that
    /// inherited everything is not what "default" should mean.
    #[test]
    fn capabilities_default_to_read_only() {
        let built = build_custom_subagents_proto(&[reviewer()], &[]).unwrap();
        let tools = built[0].harness_side_tools.as_ref().unwrap();
        assert_eq!(tools.view_file.as_ref().unwrap().enabled, Some(true));
        assert_eq!(tools.run_command.as_ref().unwrap().enabled, Some(false));
        assert_eq!(tools.file_edit.as_ref().unwrap().enabled, Some(false));
    }

    /// A subagent spawning subagents is not supported by the harness.
    #[test]
    fn start_subagent_is_dropped() {
        let mut subagent = reviewer();
        subagent.capabilities = SubagentCapabilities {
            enabled_tools: Some(vec![BuiltinTools::StartSubagent, BuiltinTools::ViewFile]),
            disabled_tools: None,
        };
        let built = build_custom_subagents_proto(&[subagent], &[]).unwrap();
        let tools = built[0].harness_side_tools.as_ref().unwrap();
        assert_eq!(tools.subagents.as_ref().unwrap().enabled, Some(false));
        assert_eq!(tools.view_file.as_ref().unwrap().enabled, Some(true));
    }

    #[test]
    fn both_capability_lists_is_an_error() {
        let mut subagent = reviewer();
        subagent.capabilities = SubagentCapabilities {
            enabled_tools: Some(vec![BuiltinTools::ViewFile]),
            disabled_tools: Some(vec![BuiltinTools::RunCommand]),
        };
        let err = build_custom_subagents_proto(&[subagent], &[]).expect_err("mutually exclusive");
        assert!(err.to_string().contains("mutually exclusive"), "{err}");
    }

    /// A subagent cannot call a tool that does not exist.
    #[test]
    fn an_unregistered_tool_is_an_error() {
        let mut subagent = reviewer();
        subagent.tools = vec!["lookup".to_string()];
        let err = build_custom_subagents_proto(&[subagent.clone()], &[]).expect_err("unregistered");
        assert!(err.to_string().contains("not registered"), "{err}");

        // Registered on the agent: fine.
        assert!(build_custom_subagents_proto(&[subagent], &["lookup".to_string()]).is_ok());
    }

    #[test]
    fn a_disabled_list_subtracts_from_all_tools() {
        let mut subagent = reviewer();
        subagent.capabilities = SubagentCapabilities {
            enabled_tools: None,
            disabled_tools: Some(vec![BuiltinTools::RunCommand]),
        };
        let built = build_custom_subagents_proto(&[subagent], &[]).unwrap();
        let tools = built[0].harness_side_tools.as_ref().unwrap();
        assert_eq!(tools.run_command.as_ref().unwrap().enabled, Some(false));
        assert_eq!(tools.file_edit.as_ref().unwrap().enabled, Some(true));
    }
}
