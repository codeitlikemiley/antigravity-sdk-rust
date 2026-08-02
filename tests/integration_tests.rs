#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::field_reassign_with_default
)]

use antigravity_sdk_rust::agent::{Agent, AgentConfig};
use antigravity_sdk_rust::policy;
use antigravity_sdk_rust::types::{
    BuiltinTools, CapabilitiesConfig, GeminiConfig, GenerationConfig, ModelConfig, ModelEntry,
};

#[tokio::test]
async fn test_agent_chat_integration() {
    let _ = tracing_subscriber::fmt::try_init();

    let mut config = AgentConfig::default();

    // Set up mock harness path (absolute path to compiled Rust binary)
    let harness_path = std::env::var("CARGO_BIN_EXE_mock_localharness")
        .expect("CARGO_BIN_EXE_mock_localharness not set — run via `cargo test`");

    config.binary_path = Some(harness_path);
    config.gemini_config = GeminiConfig {
        api_key: Some("test_api_key".to_string()),
        models: ModelConfig {
            default: ModelEntry {
                name: "gemini-3.5-flash".to_string(),
                api_key: None,
                generation: GenerationConfig {
                    thinking_level: None,
                },
            },
            image_generation: ModelEntry::default(),
        },
        ..Default::default()
    };

    // Disable write tools to avoid policy assertion issues, or register allow_all policy
    config.capabilities = CapabilitiesConfig {
        enabled_tools: Some(vec![BuiltinTools::ViewFile]), // Only read-only
        disabled_tools: None,
        compaction_threshold: None,
        image_model: None,
        finish_tool_schema_json: None,
    };

    config.policies = Some(vec![policy::allow_all()]);
    config.conversation_id = Some("test-conv-0123456789abcdef0123456789".to_string());
    config.workspaces = Some(vec![
        std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
    ]);

    let agent = Agent::new(config);

    // 1. Start agent
    let agent = agent.start().await.expect("Failed to start agent");

    // 2. Chat with agent
    let response = agent
        .chat("hello")
        .await
        .expect("Failed to chat with agent");

    // 3. Verify response
    assert!(
        response
            .text
            .contains("Client info language: rust, version:")
    );
    assert!(response.text.contains("How can I help you today?"));
    assert_eq!(response.steps.len(), 2);

    // 4. Verify conversation metadata
    let conversation = agent.conversation();
    assert_eq!(
        conversation.conversation_id(),
        "test-conv-0123456789abcdef0123456789"
    );

    // 5. Stop agent
    agent.stop().await.expect("Failed to stop agent");
}

#[tokio::test]
async fn test_agent_start_mutually_exclusive_capabilities() {
    let mut config = AgentConfig::default();
    config.binary_path = Some("some_path".to_string());
    config.capabilities = CapabilitiesConfig {
        enabled_tools: Some(vec![BuiltinTools::ViewFile]),
        disabled_tools: Some(vec![BuiltinTools::RunCommand]),
        compaction_threshold: None,
        image_model: None,
        finish_tool_schema_json: None,
    };

    let agent = Agent::new(config);
    let result = agent.start().await;
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("mutually exclusive"));
}

#[tokio::test]
async fn test_agent_real_chat_integration() {
    let _ = tracing_subscriber::fmt::try_init();

    // Load environment variables from .env if present
    dotenvy::dotenv().ok();

    // Check if GEMINI_API_KEY is present
    let api_key = match std::env::var("GEMINI_API_KEY") {
        Ok(key) if !key.trim().is_empty() => key,
        _ => {
            println!("Skipping real integration test: GEMINI_API_KEY is not set.");
            return;
        }
    };

    let mut config = AgentConfig::default();

    // Try to get binary path from environment, or let it fall back
    if let Ok(harness_path) = std::env::var("ANTIGRAVITY_HARNESS_PATH") {
        config.binary_path = Some(harness_path);
    }

    config.gemini_config = GeminiConfig {
        api_key: Some(api_key),
        models: ModelConfig {
            default: ModelEntry {
                name: "gemini-3.5-flash".to_string(),
                api_key: None,
                generation: GenerationConfig {
                    thinking_level: None,
                },
            },
            image_generation: ModelEntry::default(),
        },
        ..Default::default()
    };

    let tmp_dir = std::env::temp_dir().join(format!(
        "antigravity_test_{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
    ));
    std::fs::create_dir_all(&tmp_dir).unwrap();
    config.save_dir = Some(tmp_dir.to_string_lossy().into_owned());

    config.policies = Some(vec![policy::allow_all()]);
    config.workspaces = Some(vec![
        std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
    ]);

    let agent = Agent::new(config);

    // 1. Start agent
    let agent = agent.start().await.expect("Failed to start agent");

    // 2. Chat with agent
    let response = agent
        .chat("Say 'Hello from integration test!'")
        .await
        .expect("Failed to chat with agent");

    // 3. Verify response contains hello or integration
    let response_lower = response.text.to_lowercase();
    assert!(
        response_lower.contains("hello") || response_lower.contains("integration"),
        "Expected response to contain hello/integration, got: {}",
        response.text
    );

    // 4. Verify conversation metadata
    let conversation = agent.conversation();
    assert!(!conversation.conversation_id().is_empty());

    // 5. Stop agent
    agent.stop().await.expect("Failed to stop agent");
}

#[tokio::test]
async fn test_agent_terminal_error_propagation() {
    let mut config = AgentConfig::default();

    // Set up mock harness path
    let harness_path = std::env::var("CARGO_BIN_EXE_mock_localharness")
        .expect("CARGO_BIN_EXE_mock_localharness not set — run via `cargo test`");

    config.binary_path = Some(harness_path);
    config.gemini_config = GeminiConfig {
        api_key: Some("test_api_key".to_string()),
        models: ModelConfig {
            default: ModelEntry {
                name: "gemini-3.5-flash".to_string(),
                api_key: None,
                generation: GenerationConfig {
                    thinking_level: None,
                },
            },
            image_generation: ModelEntry::default(),
        },
        ..Default::default()
    };

    config.capabilities = CapabilitiesConfig {
        enabled_tools: Some(vec![BuiltinTools::ViewFile]),
        disabled_tools: None,
        compaction_threshold: None,
        image_model: None,
        finish_tool_schema_json: None,
    };

    config.policies = Some(vec![policy::allow_all()]);
    config.conversation_id = Some("test-conv-err-0123456789abcdef0123".to_string());

    let agent = Agent::new(config);
    let agent = agent.start().await.expect("Failed to start agent");

    // Chat with agent triggering terminal error
    let res = agent.chat("trigger_terminal_error").await;
    assert!(res.is_err());
    let err_msg = res.unwrap_err().to_string();
    assert!(err_msg.contains("Terminal execution error: Terminal error triggered by prompt"));

    agent.stop().await.expect("Failed to stop agent");
}

/// End-to-end proof that the workspace sandbox is wired, not merely composed.
///
/// The unit tests in `src/agent.rs` show `compose_policies` produces the right
/// policy list. They cannot show that the resulting enforcer is registered on
/// the hook runner and consulted when the harness asks to run a tool. This
/// drives a real `tool_confirmation_request` through the mock and reads back
/// the `accepted` flag the SDK returns, which *is* the policy decision.
///
/// Covers the confirmation path only — the sole pre-tool gate the 0.1.1 wire
/// has. Gating built-ins without a confirmation request needs the harness-side
/// hook channel (WP-8).
async fn confirmation_decision_for(path: &str) -> String {
    let mut config = AgentConfig::default();
    config.binary_path = Some(
        std::env::var("CARGO_BIN_EXE_mock_localharness")
            .expect("CARGO_BIN_EXE_mock_localharness not set — run via `cargo test`"),
    );
    config.gemini_config = GeminiConfig {
        api_key: Some("test_api_key".to_string()),
        ..Default::default()
    };
    config.capabilities = CapabilitiesConfig {
        enabled_tools: Some(vec![BuiltinTools::ViewFile]),
        ..Default::default()
    };
    // allow_all() used to switch the workspace sandbox off entirely. It must
    // not any more — that is the regression this asserts end to end.
    config.policies = Some(vec![policy::allow_all()]);
    config.workspaces = Some(vec![
        std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .into_owned(),
    ]);

    let agent = Agent::new(config)
        .start()
        .await
        .expect("Failed to start agent");
    let response = agent
        .chat(&format!("trigger_tool_confirmation:{path}"))
        .await
        .expect("chat failed");
    agent.stop().await.expect("Failed to stop agent");
    response.text
}

#[tokio::test]
async fn test_workspace_policy_denies_path_outside_workspace() {
    let decision = confirmation_decision_for("/etc/passwd").await;
    assert!(
        decision.contains("accepted=false"),
        "a file outside the workspace must be denied, got: {decision}"
    );
}

#[tokio::test]
async fn test_workspace_policy_allows_path_inside_workspace() {
    let inside = std::env::current_dir().unwrap().join("Cargo.toml");
    let decision = confirmation_decision_for(&inside.to_string_lossy()).await;
    assert!(
        decision.contains("accepted=true"),
        "a file inside the workspace must be allowed, got: {decision}"
    );
}

/// The traversal escape, end to end. `Path::starts_with` reported this as
/// inside the workspace, so `view_file` on /etc/passwd was confirmed.
#[tokio::test]
async fn test_workspace_policy_denies_parent_traversal() {
    let escape = std::env::current_dir()
        .unwrap()
        .join("../../etc/passwd")
        .to_string_lossy()
        .into_owned();
    let decision = confirmation_decision_for(&escape).await;
    assert!(
        decision.contains("accepted=false"),
        "a `..` escape must be denied, got: {decision}"
    );
}

/// A caller-initiated halt must be distinguishable from a turn that simply
/// finished (A3). The harness answers a halt with a plain `STATE_FULLY_IDLE`,
/// so without the client-side flag the stream would just end normally.
#[tokio::test]
async fn test_cancel_surfaces_cancelled_error() {
    use futures_util::StreamExt;

    let mut config = AgentConfig::default();
    config.binary_path = Some(
        std::env::var("CARGO_BIN_EXE_mock_localharness")
            .expect("CARGO_BIN_EXE_mock_localharness not set — run via `cargo test`"),
    );
    config.gemini_config = GeminiConfig {
        api_key: Some("test_api_key".to_string()),
        ..Default::default()
    };
    config.policies = Some(vec![policy::allow_all()]);
    config.conversation_id = Some("test-conv-cancel-0123456789abcdef".to_string());

    let agent = Agent::new(config).start().await.expect("start");
    let conversation = agent.conversation();

    conversation.send("trigger_cancel").await.expect("send");

    let mut stream = conversation.receive_steps();
    // The mock emits one step before stalling; draining it proves the turn is
    // under way, so the halt below lands mid-turn rather than before it starts.
    let first = stream.next().await.expect("a step").expect("not an error");
    assert_eq!(first.content, "Working...");

    conversation.cancel().await.expect("cancel");

    let mut saw_cancelled = false;
    while let Some(item) = stream.next().await {
        if let Err(e) = item {
            saw_cancelled = e
                .downcast_ref::<antigravity_sdk_rust::error::AntigravityError>()
                .is_some_and(|e| {
                    matches!(
                        e,
                        antigravity_sdk_rust::error::AntigravityError::Cancelled(_)
                    )
                });
            if saw_cancelled {
                break;
            }
        }
    }
    assert!(saw_cancelled, "cancelled turn ended as if it had completed");

    agent.stop().await.expect("stop");
}

/// A harness that dies mid-turn must surface an error carrying what it printed
/// on the way out, not end the step stream as though the turn had completed
/// (`harness-crash-diagnostics`).
#[tokio::test]
async fn test_harness_crash_surfaces_stderr_tail() {
    use futures_util::StreamExt;

    let mut config = AgentConfig::default();
    config.binary_path = Some(
        std::env::var("CARGO_BIN_EXE_mock_localharness")
            .expect("CARGO_BIN_EXE_mock_localharness not set — run via `cargo test`"),
    );
    config.gemini_config = GeminiConfig {
        api_key: Some("test_api_key".to_string()),
        ..Default::default()
    };
    config.policies = Some(vec![policy::allow_all()]);
    config.conversation_id = Some("test-conv-crash-0123456789abcdef0".to_string());

    let agent = Agent::new(config).start().await.expect("start");
    let conversation = agent.conversation();
    conversation.send("trigger_crash").await.expect("send");

    let mut stream = conversation.receive_steps();
    let mut errors = Vec::new();
    while let Some(item) = stream.next().await {
        if let Err(e) = item {
            errors.push(e.to_string());
        }
    }

    let joined = errors.join("\n");
    assert!(
        joined.contains("closed before the turn finished"),
        "a crash ended the stream silently; saw: {joined}"
    );
    assert!(
        joined.contains("mock harness exploded"),
        "the crash was reported without the harness's own diagnostics; saw: {joined}"
    );
}

/// A subagent finishing is how a `START_SUBAGENT` call completes — the harness
/// sends no tool response for it. Before H12 a `post_tool_call` hook saw the
/// `pre_tool_call` and never a matching completion, so
/// `examples/subagents.rs`'s "Subagent Finished" branch never fired.
#[tokio::test]
async fn test_post_tool_call_fires_on_subagent_completion() {
    use antigravity_sdk_rust::hooks::Hook;
    use antigravity_sdk_rust::types::ToolResult;
    use futures_util::StreamExt;
    use std::sync::{Arc, Mutex};

    struct CaptureHook(Arc<Mutex<Vec<ToolResult>>>);

    impl Hook for CaptureHook {
        async fn post_tool_call(&self, result: &ToolResult) -> Result<(), anyhow::Error> {
            self.0.lock().expect("lock").push(result.clone());
            Ok(())
        }
    }

    let captured = Arc::new(Mutex::new(Vec::new()));

    let mut config = AgentConfig::default();
    config.binary_path = Some(
        std::env::var("CARGO_BIN_EXE_mock_localharness")
            .expect("CARGO_BIN_EXE_mock_localharness not set — run via `cargo test`"),
    );
    config.gemini_config = GeminiConfig {
        api_key: Some("test_api_key".to_string()),
        ..Default::default()
    };
    config.policies = Some(vec![policy::allow_all()]);
    config.conversation_id = Some("test-conv-subagent-0123456789abc".to_string());
    config.hooks = vec![Arc::new(CaptureHook(captured.clone()))];

    let agent = Agent::new(config).start().await.expect("start");
    let conversation = agent.conversation();
    conversation.send("trigger_subagent").await.expect("send");

    let mut stream = conversation.receive_steps();
    while stream.next().await.is_some() {}

    // The dispatch is spawned, so give it a moment to land rather than racing it.
    for _ in 0..50 {
        if !captured.lock().expect("lock").is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }

    let results = captured.lock().expect("lock").clone();
    let subagent = results
        .iter()
        .find(|r| r.name == "START_SUBAGENT")
        .expect("no post_tool_call for the finished subagent");
    assert_eq!(
        subagent.result,
        Some(serde_json::json!("Here is a poem about nature.")),
        "the completion should carry what the subagent produced"
    );

    agent.stop().await.expect("stop");
}

/// `post_turn` was defined and dispatched from nowhere (H1b). It fires at the
/// terminal user-facing model step, carrying that step's text.
#[tokio::test]
async fn test_post_turn_fires_with_the_final_text() {
    use antigravity_sdk_rust::hooks::Hook;
    use futures_util::StreamExt;
    use std::sync::{Arc, Mutex};

    struct CaptureTurn(Arc<Mutex<Vec<String>>>);

    impl Hook for CaptureTurn {
        async fn post_turn(&self, response: &str) -> Result<(), anyhow::Error> {
            self.0.lock().expect("lock").push(response.to_string());
            Ok(())
        }
    }

    let seen = Arc::new(Mutex::new(Vec::new()));

    let mut config = AgentConfig::default();
    config.binary_path = Some(
        std::env::var("CARGO_BIN_EXE_mock_localharness")
            .expect("CARGO_BIN_EXE_mock_localharness not set — run via `cargo test`"),
    );
    config.gemini_config = GeminiConfig {
        api_key: Some("test_api_key".to_string()),
        ..Default::default()
    };
    config.policies = Some(vec![policy::allow_all()]);
    config.conversation_id = Some("test-conv-postturn-0123456789abc".to_string());
    config.hooks = vec![Arc::new(CaptureTurn(seen.clone()))];

    let agent = Agent::new(config).start().await.expect("start");
    let conversation = agent.conversation();
    conversation.send("hello").await.expect("send");

    let mut stream = conversation.receive_steps();
    while stream.next().await.is_some() {}

    // Dispatch is spawned; give it a moment rather than racing it.
    for _ in 0..50 {
        if !seen.lock().expect("lock").is_empty() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }

    let texts = seen.lock().expect("lock").clone();
    assert!(
        texts
            .iter()
            .any(|t| t.contains("How can I help you today?")),
        "post_turn never fired with the turn's final text; saw {texts:?}"
    );

    agent.stop().await.expect("stop");
}

/// The harness blocks its turn until a `CallHookResponse` comes back. Before
/// the router existed this arm only logged a warning, so a harness that sent
/// one would have stalled — which is why `enabled_hooks` could not be emitted.
#[tokio::test]
async fn test_harness_hook_request_is_answered() {
    let mut config = AgentConfig::default();
    config.binary_path = Some(
        std::env::var("CARGO_BIN_EXE_mock_localharness")
            .expect("CARGO_BIN_EXE_mock_localharness not set — run via `cargo test`"),
    );
    config.gemini_config = GeminiConfig {
        api_key: Some("test_api_key".to_string()),
        ..Default::default()
    };
    // A policy the router must consult: RUN_COMMAND is denied.
    config.policies = Some(vec![policy::deny("RUN_COMMAND"), policy::allow_all()]);
    config.conversation_id = Some("test-conv-hookreq-0123456789abcd".to_string());

    let agent = Agent::new(config).start().await.expect("start");
    let response = tokio::time::timeout(
        std::time::Duration::from_secs(20),
        agent.chat("trigger_hook_request"),
    )
    .await
    .expect("the harness stalled waiting for a CallHookResponse")
    .expect("chat failed");

    assert!(
        response.text.contains("decision=DENY"),
        "the router did not consult the policy; saw {:?}",
        response.text
    );

    agent.stop().await.expect("stop");
}
