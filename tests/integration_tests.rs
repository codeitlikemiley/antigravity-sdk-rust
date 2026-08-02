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
    config.conversation_id = Some("test_conv_123".to_string());
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
    assert_eq!(conversation.conversation_id(), "test_conv_123");

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
    config.conversation_id = Some("test_conv_err".to_string());

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
