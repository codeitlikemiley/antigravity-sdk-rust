//! HTTP Sidecar Server for the Antigravity SDK.
//!
//! This binary wraps the full `antigravity-sdk-rust` Agent (including localharness,
//! tools, hooks, and policies) and exposes it via a simple HTTP REST API.
//!
//! Spin/WASI components cannot make outbound TCP/WebSocket connections — only `wasi::http`.
//! This sidecar bridges that gap: it runs natively, maintains the WebSocket connection
//! to localharness, and exposes streaming SSE for the Spin component to call.
//!
//! # Key Design
//!
//! Each chat session gets its OWN agent process (harness subprocess) and its own
//! workspace directory:
//!
//! - Default:  `/tmp/antigravity/sessions/<session_id>/workspace/`  (created automatically)
//! - Override: any absolute path the user provides via `POST /workspace`
//!
//! This means `list_dir`, `create_file`, `view_file`, etc. always work — there
//! is always a real directory for the harness to operate in.
//!
//! # Usage
//!
//! ```bash
//! # Terminal 1: Start the sidecar
//! cd examples/agent_server
//! GEMINI_API_KEY=your-key cargo run
//!
//! # Terminal 2: Start Spin
//! cd examples/leptos_ssr_axum
//! spin build --up
//! ```

use antigravity_sdk_rust::agent::{Agent, Started};
use antigravity_sdk_rust::connection::Connection;
use antigravity_sdk_rust::conversation::Conversation;
use antigravity_sdk_rust::hooks::Hook;
use antigravity_sdk_rust::types::{
    AskQuestionEntry, CapabilitiesConfig, CustomSystemInstructions,
    GenerationConfig, GeminiConfig, ModelConfig, ModelEntry,
    HookResult, QuestionHookResult, QuestionResponse, StepSource, StepStatus, StepType,
    SystemInstructions, ThinkingLevel, ToolCall,
};
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::routing::{get, post};
use axum::{Json, Router};
use futures_util::StreamExt;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

// ---------------------------------------------------------------------------
// Confirm hook — intercepts write tools and waits for user approval
// ---------------------------------------------------------------------------

/// Payload sent from ConfirmHook to the SSE stream handler when a tool needs user approval.
struct ConfirmHookRequest {
    /// Snapshot of the tool call requiring approval.
    tool_call: ToolCall,
    /// Send `true` to approve, `false` to deny.
    result_tx: tokio::sync::oneshot::Sender<bool>,
}

/// Write tools that require explicit user confirmation on each new session.
const WRITE_TOOLS: &[&str] = &[
    "CREATE_FILE",
    "EDIT_FILE",
    "RUN_COMMAND",
    "FIND_FILE",
];

/// Custom [`Hook`] that blocks write-tool execution until the user approves/denies
/// via the frontend's confirmation dialog.
struct ConfirmHook {
    /// Channel to send pending confirmation requests to the SSE stream handler.
    confirm_tx: tokio::sync::mpsc::Sender<ConfirmHookRequest>,
    /// Tools that are auto-approved for this session (populated by "Allow for Session").
    auto_allowed: Arc<RwLock<HashSet<String>>>,
}

impl Hook for ConfirmHook {
    async fn pre_tool_call<'a>(
        &'a self,
        tool_call: &'a ToolCall,
    ) -> Result<HookResult, anyhow::Error> {
        // Always approve non-write tools immediately.
        if !WRITE_TOOLS.contains(&tool_call.name.as_str()) {
            return Ok(HookResult { allow: true, message: String::new() });
        }
        // Check if this tool has been auto-allowed for the session.
        {
            let allowed = self.auto_allowed.read().await;
            if allowed.contains(&tool_call.name) {
                tracing::debug!("ConfirmHook: auto-approving '{}'", tool_call.name);
                return Ok(HookResult { allow: true, message: String::new() });
            }
        }
        // Send a confirmation request to the stream handler and await the user's decision.
        let (result_tx, result_rx) = tokio::sync::oneshot::channel::<bool>();
        let req = ConfirmHookRequest {
            tool_call: tool_call.clone(),
            result_tx,
        };
        if self.confirm_tx.send(req).await.is_err() {
            // Stream handler has gone away — deny for safety.
            tracing::warn!("ConfirmHook: stream handler gone, denying '{}'", tool_call.name);
            return Ok(HookResult { allow: false, message: "Stream handler unavailable".to_string() });
        }
        // Block until the user decides.
        match result_rx.await {
            Ok(true) => {
                tracing::info!("ConfirmHook: user approved '{}'", tool_call.name);
                Ok(HookResult { allow: true, message: String::new() })
            }
            Ok(false) => {
                tracing::info!("ConfirmHook: user denied '{}'", tool_call.name);
                Ok(HookResult { allow: false, message: format!("User denied execution of '{}'", tool_call.name) })
            }
            Err(_) => {
                // Receiver dropped — stream handler died, deny for safety.
                Ok(HookResult { allow: false, message: "Confirmation channel closed".to_string() })
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Per-session state
// ---------------------------------------------------------------------------

/// All mutable state belonging to a single chat session.
struct SessionEntry {
    conversation: Arc<Conversation>,
    pending_question: Arc<Mutex<Option<tokio::sync::oneshot::Sender<QuestionHookResult>>>>,
    /// Shared auto-allowed tools set (also held by ConfirmHook for real-time updates).
    auto_allowed: Arc<RwLock<HashSet<String>>>,
    /// Absolute path of the workspace directory this session's harness operates in.
    workspace_path: String,
    /// Receiver for confirmation requests from ConfirmHook.
    /// Wrapped in Arc<Mutex> so it survives across multiple conversation turns
    /// without being moved — each turn's confirm-task locks it for that turn only.
    confirm_rx: Arc<tokio::sync::Mutex<tokio::sync::mpsc::Receiver<ConfirmHookRequest>>>,
    /// Holds the oneshot TX from ConfirmHookRequest so /confirm can fire it.
    confirm_result_tx: Arc<Mutex<Option<tokio::sync::oneshot::Sender<bool>>>>,
}

// ---------------------------------------------------------------------------
// Immutable server config (read once at startup)
// ---------------------------------------------------------------------------

struct ServerConfig {
    harness_path: Option<String>,
    api_key: Option<String>,
    model: String,
    app_data_dir: String,
    /// Root under which per-session workspace subdirs are created.
    base_sessions_dir: PathBuf,
    /// Optional env-level workspace override (WORKSPACE_ROOT).
    /// When set, ALL sessions use this path and filesystem tools are fully enabled.
    env_workspace: Option<String>,
}

// ---------------------------------------------------------------------------
// Shared application state
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct AppState {
    sessions: Arc<Mutex<HashMap<String, Arc<Mutex<SessionEntry>>>>>,
    config: Arc<ServerConfig>,
}

// ---------------------------------------------------------------------------
// Request / response bodies
// ---------------------------------------------------------------------------

/// Query parameters for the streaming `GET /chat/stream` endpoint.
#[derive(Debug, Deserialize)]
struct ChatStreamParams {
    message: String,
    session_id: String,
}

/// Request body for `POST /answer`.
#[derive(Debug, Deserialize)]
struct AnswerRequest {
    session_id: String,
    trajectory_id: String,
    step_index: u32,
    responses: Vec<QuestionResponse>,
    #[serde(default)]
    cancelled: bool,
}

/// Request body for `POST /confirm`.
#[derive(Debug, Deserialize)]
struct ConfirmRequest {
    session_id: String,
    trajectory_id: String,
    step_index: u32,
    accepted: bool,
    /// If true, approve all future requests for this tool in this session.
    #[serde(default)]
    allow_for_session: bool,
    /// Name of the tool being confirmed (needed for auto-allow tracking).
    tool_name: Option<String>,
}

/// Request body for `POST /halt`.
#[derive(Debug, Deserialize)]
struct HaltRequest {
    session_id: String,
}

/// Request body for `POST /workspace` — set or change a session's workspace.
#[derive(Debug, Deserialize)]
struct SetWorkspaceRequest {
    session_id: String,
    /// Absolute path to the project folder to use as workspace.
    path: String,
}

/// Query params for `GET /workspace`.
#[derive(Debug, Deserialize)]
struct GetWorkspaceParams {
    session_id: String,
}

// ---------------------------------------------------------------------------
// Helper: build a single SSE Event from name + JSON value
// ---------------------------------------------------------------------------

fn sse_event(event_name: &str, data: serde_json::Value) -> Event {
    Event::default()
        .event(event_name)
        .json_data(data)
        .unwrap_or_else(|_| Event::default().event(event_name).data("{}"))
}

// ---------------------------------------------------------------------------
// Helper: create a new agent session
// ---------------------------------------------------------------------------

/// Builds and starts a new agent for `session_id`, using `workspace_override` if
/// provided, otherwise the env-level workspace, otherwise a freshly-created
/// per-session temp directory.
async fn build_session(
    config: &ServerConfig,
    session_id: &str,
    workspace_override: Option<String>,
) -> Result<SessionEntry, anyhow::Error> {
    // 1. Resolve workspace path
    let workspace = workspace_override
        .or_else(|| config.env_workspace.clone())
        .unwrap_or_else(|| {
            config
                .base_sessions_dir
                .join(session_id)
                .join("workspace")
                .to_string_lossy()
                .into_owned()
        });

    // 2. Ensure the directory exists
    std::fs::create_dir_all(&workspace).map_err(|e| {
        anyhow::anyhow!("Failed to create workspace dir {:?}: {e}", workspace)
    })?;
    tracing::info!("Session {session_id} workspace: {workspace}");

    // 3. Build Agent
    // Build base builder with optional harness path + api key
    let mut base = Agent::builder();
    if let Some(ref path) = config.harness_path {
        base = base.binary_path(path.clone());
    }
    if let Some(ref key) = config.api_key {
        base = base.api_key(key.clone());
    }

    let model_name = config.model.clone();

    // All tools enabled — workspace always exists
    let capabilities = CapabilitiesConfig {
        enabled_tools: None, // default = all
        disabled_tools: None,
        compaction_threshold: None,
        image_model: None,
        finish_tool_schema_json: None,
    };

    let system_text = format!(
        "You are an expert AI coding assistant. \
         Your active workspace is: {workspace}\n\
         You can freely read, create, edit, and run code within that directory. \
         CRITICAL PATH RULE: When specifying file paths in tool arguments, you MUST use plain \
         absolute filesystem paths that start with '/' (e.g. {workspace}/src/main.rs). \
         NEVER use file:// or file:/// URI prefixes — they are not valid filesystem paths and \
         will cause tool failures. Always write paths as: /absolute/path/to/file, never as \
         file:///absolute/path/to/file. \
         Only use filesystem tools (like listing directory contents, reading/searching files, and running commands) when relevant to the user request. Do not access the workspace filesystem or run directory listings for queries that only need web search or general information. \
         Think step-by-step and show your reasoning. \
         Use markdown formatting for all responses including code blocks."
    );

    // Build the shared auto-allowed set and confirm channel.
    // ConfirmHook holds the Sender and auto_allowed Arc.
    // SessionEntry holds the Receiver (moved into stream handler on first chat).
    let auto_allowed: Arc<RwLock<HashSet<String>>> = Arc::new(RwLock::new(HashSet::new()));
    let (confirm_tx, confirm_rx) = tokio::sync::mpsc::channel::<ConfirmHookRequest>(4);
    let hook = Arc::new(ConfirmHook {
        confirm_tx,
        auto_allowed: auto_allowed.clone(),
    });

    // NOTE: We intentionally do NOT call .workspaces() here.
    // When .workspaces() is set, the SDK automatically prepends workspace_only policies
    // BEFORE allow_all() in the hook chain. That workspace_only gate then denies any
    // file operation that doesn't match the allowed paths — even after the user has
    // explicitly approved via ConfirmHook. Since we manage all access control through
    // ConfirmHook + allow_all(), we skip workspace registration to avoid this conflict.
    // The workspace path is already communicated to the agent via the system prompt.
    //
    // Builder type-state: all configuration methods (.hook, .capabilities, etc.) must
    // come BEFORE .allow_all() which transitions NoPolicies → HasPolicies. After that,
    // only .build() is available.
    //
    // gemini-3.5-flash supports Thinking (confirmed at ai.google.dev).
    // Set ThinkingLevel::High so the agent reasons through every turn.
    let gemini_cfg = GeminiConfig {
        api_key: config.api_key.clone(),
        models: ModelConfig {
            default: ModelEntry {
                name: model_name.clone(),
                api_key: None,
                generation: GenerationConfig {
                    thinking_level: Some(ThinkingLevel::High),
                },
            },
            image_generation: ModelEntry::default(),
        },
        enable_google_search: Some(true),
        enable_url_context: Some(true),
        ..Default::default()
    };
    let agent: Agent<Started> = base
        .app_data_dir(&config.app_data_dir)
        .default_model(model_name)
        .gemini_config(gemini_cfg)
        .hook(hook)             // register ConfirmHook before allow_all()
        .capabilities(capabilities)
        .system_instructions(SystemInstructions::Custom(CustomSystemInstructions {
            text: system_text,
        }))
        .allow_all()            // type-state: NoPolicies → HasPolicies
        .build()
        .start()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to start agent for session {session_id}: {e}"))?;

    let conversation = agent.conversation();

    Ok(SessionEntry {
        conversation,
        pending_question: Arc::new(Mutex::new(None)),
        auto_allowed,
        workspace_path: workspace,
        confirm_rx: Arc::new(tokio::sync::Mutex::new(confirm_rx)),
        confirm_result_tx: Arc::new(Mutex::new(None)),
    })
}

/// Look up or lazily create a session entry. Returns a clone of the Arc<Mutex<SessionEntry>>.
async fn get_or_create_session(
    state: &AppState,
    session_id: &str,
) -> Result<Arc<Mutex<SessionEntry>>, anyhow::Error> {
    let sessions = state.sessions.lock().await;
    if let Some(entry) = sessions.get(session_id) {
        return Ok(entry.clone());
    }
    // Not found — create fresh
    drop(sessions); // release lock while we do the async build
    let entry = build_session(&state.config, session_id, None).await?;
    let entry_arc = Arc::new(Mutex::new(entry));
    let mut sessions = state.sessions.lock().await;
    // Double-check after re-acquiring (another request may have beaten us)
    sessions
        .entry(session_id.to_string())
        .or_insert_with(|| entry_arc.clone());
    Ok(sessions[session_id].clone())
}

// ---------------------------------------------------------------------------
// GET /chat/stream — SSE streaming
// ---------------------------------------------------------------------------

async fn chat_stream_handler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<ChatStreamParams>,
) -> impl IntoResponse {
    let (tx, rx) = tokio::sync::mpsc::channel::<Result<Event, Infallible>>(2048);

    let session_id = params.session_id.clone();

    // Get (or lazily create) the session — on failure emit error events via the same channel
    match get_or_create_session(&state, &session_id).await {
        Err(e) => {
            tracing::error!("Failed to create session {session_id}: {e:?}");
            let tx2 = tx.clone();
            tokio::spawn(async move {
                let _ = tx2.send(Ok(sse_event("error", serde_json::json!({
                    "message": format!("Failed to start agent: {e}"),
                    "http_code": 500,
                })))).await;
                let _ = tx2.send(Ok(sse_event("idle", serde_json::json!({})))).await;
                let _ = tx2.send(Ok(sse_event("done", serde_json::json!({})))).await;
            });
        }
        Ok(session_arc) => {
            tokio::spawn(async move {
                let (agent, pending_question, confirm_rx_arc, confirm_result_tx) = {
                    let session = session_arc.lock().await;
                    (
                        session.conversation.clone(),
                        session.pending_question.clone(),
                        session.confirm_rx.clone(), // Arc clone — receiver stays in session
                        session.confirm_result_tx.clone(),
                    )
                };

                // ── Concurrent confirm-request handler ─────────────────────────────
                // Spawned as its own task so it NEVER blocks the step-stream loop.
                // Locks the shared confirm_rx for this turn only; the lock is released
                // when this task exits (turn ends), allowing the next turn to reuse it.
                let tx_confirm = tx.clone();
                let confirm_result_tx2 = confirm_result_tx.clone();
                // watch channel: step-stream stores the LATEST WaitingForUser context.
                // Using watch instead of mpsc so that:
                //   - send() never blocks (no capacity to fill up)
                //   - confirm-task always reads the most-recent (trajectory_id, step_index)
                //   - auto-approved WaitingForUser steps don't accumulate in a buffer and
                //     block the step-stream loop when the confirm-task is idle.
                let (ctx_watch_tx, ctx_watch_rx) =
                    tokio::sync::watch::channel((String::new(), 0u32));
                let confirm_task = tokio::spawn(async move {
                    let mut rx = confirm_rx_arc.lock().await;
                    tracing::debug!("confirm-task: started, waiting for ConfirmHookRequests");
                    while let Some(req) = rx.recv().await {
                        tracing::info!(
                            tool = %req.tool_call.name,
                            "confirm-task: received request — emitting SSE"
                        );
                        // Read the latest WaitingForUser context written by the step-stream.
                        // watch::Sender::borrow() never blocks — just reads the current value.
                        let (trajectory_id, step_index) = ctx_watch_rx.borrow().clone();
                        let _ = tx_confirm.send(Ok(sse_event("confirm", serde_json::json!({
                            "trajectory_id": trajectory_id,
                            "step_index":    step_index,
                            "tool_call": {
                                "id":             req.tool_call.id,
                                "name":           req.tool_call.name,
                                "args":           req.tool_call.args,
                                "canonical_path": req.tool_call.canonical_path,
                            },
                        })))).await;
                        let mut crt = confirm_result_tx2.lock().await;
                        *crt = Some(req.result_tx);
                        tracing::debug!("confirm-task: result_tx stored, waiting for /confirm");
                    }
                    tracing::debug!("confirm-task: channel closed — exiting");
                });

                // ── Send the prompt ────────────────────────────────────────────────
                tracing::debug!(msg = %params.message, "stream-handler: sending prompt");
                if let Err(e) = agent.send(&params.message).await {
                    tracing::error!("stream-handler: send error: {e}");
                    let _ = tx.send(Ok(sse_event("error", serde_json::json!({
                        "message": format!("{e}"),
                        "http_code": 500,
                    })))).await;
                    let _ = tx.send(Ok(sse_event("idle", serde_json::json!({})))).await;
                    let _ = tx.send(Ok(sse_event("done", serde_json::json!({})))).await;
                    return;
                }

                let mut step_stream = agent.receive_steps();
                let conn = agent.connection();

                // Track which tool call IDs we've already emitted tool_start for.
                // The SDK sends multiple Active status updates for the same step while
                // it runs — we only want to emit tool_start once per unique call ID.
                let mut seen_tool_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

                while let Some(step_res) = step_stream.next().await {
                    match step_res {
                        Err(e) => {
                            tracing::error!("stream-handler: step error: {e}");
                            let _ = tx.send(Ok(sse_event("error", serde_json::json!({
                                "message": format!("{e}"),
                                "http_code": 500,
                            })))).await;
                            let _ = tx.send(Ok(sse_event("idle", serde_json::json!({})))).await;
                            let _ = tx.send(Ok(sse_event("done", serde_json::json!({})))).await;
                            break;
                        }
                        Ok(step) => {
                            let step_index = step.step_index;
                            let trajectory_id = step.trajectory_id.clone();

                            tracing::debug!(
                                step_index,
                                status  = ?step.status,
                                r#type  = ?step.r#type,
                                source  = ?step.source,
                                tools   = step.tool_calls.len(),
                                "stream-handler: step received"
                            );

                            // ── thought delta ──────────────────────────────────────
                            if step.source == StepSource::Model && !step.thinking_delta.is_empty() {
                                let _ = tx.send(Ok(sse_event("thought", serde_json::json!({
                                    "step_index": step_index,
                                    "text": step.thinking_delta,
                                    "trajectory_id": &trajectory_id,
                                })))).await;
                            }

                            // ── text token delta ───────────────────────────────────
                            // Skip token emission for ToolCall steps — their content_delta
                            // is the tool label (e.g. "Workspace directory listing"), not
                            // assistant prose. The label is passed via tool_start instead.
                            if step.source == StepSource::Model && !step.content_delta.is_empty()
                                && step.r#type != StepType::ToolCall
                            {
                                let _ = tx.send(Ok(sse_event("token", serde_json::json!({
                                    "step_index": step_index,
                                    "text": step.content_delta,
                                    "trajectory_id": &trajectory_id,
                                })))).await;
                            }

                            // ── tool calls ─────────────────────────────────────────
                            if step.r#type == StepType::ToolCall {
                                for call in &step.tool_calls {
                                    if step.status == StepStatus::Active {
                                        // Only emit tool_start once per call ID.
                                        if seen_tool_ids.insert(call.id.clone()) {
                                            tracing::info!(
                                                tool = %call.name, id = %call.id,
                                                "stream-handler: tool_start"
                                            );
                                            let _ = tx.send(Ok(sse_event("tool_start", serde_json::json!({
                                                "id":             call.id,
                                                "name":           call.name,
                                                "args":           call.args,
                                                "canonical_path": call.canonical_path,
                                                // step.content is the human-readable label
                                                // the agent sets for this action (e.g. "Change Directory").
                                                "label":          step.content,
                                                "trajectory_id":  &trajectory_id,
                                            })))).await;
                                        }
                                    } else if step.status == StepStatus::Done
                                        || step.status == StepStatus::Error
                                        || step.status == StepStatus::TerminalError
                                    {
                                        // If this tool was never seen as Active (e.g. auto-approved
                                        // non-WRITE_TOOLS like LIST_DIR that go straight from
                                        // WaitingForUser → Done), emit a synthetic tool_start first
                                        // so the frontend creates a proper ToolCallView card.
                                        if seen_tool_ids.insert(call.id.clone()) {
                                            tracing::info!(
                                                tool = %call.name, id = %call.id,
                                                "stream-handler: synthetic tool_start (skipped Active)"
                                            );
                                            let _ = tx.send(Ok(sse_event("tool_start", serde_json::json!({
                                                "id":             call.id,
                                                "name":           call.name,
                                                "args":           call.args,
                                                "canonical_path": call.canonical_path,
                                                "label":          step.content,
                                                "trajectory_id":  &trajectory_id,
                                            })))).await;
                                        }
                                        tracing::info!(
                                            tool   = %call.name,
                                            status = ?step.status,
                                            "stream-handler: tool_result"
                                        );
                                        // For Done steps, call.args contains the full result
                                        // (e.g. combined_output, exit_code for RUN_COMMAND).
                                        // step.content is only the step label ("Create Rust Project"),
                                        // not the actual command stdout.
                                        let result_val = if step.status == StepStatus::Done {
                                            call.args.clone()
                                        } else {
                                            serde_json::Value::Null
                                        };
                                        let error_val = if step.status == StepStatus::Error {
                                            Some(if step.error.is_empty() {
                                                "Tool execution failed".to_string()
                                            } else {
                                                step.error.clone()
                                            })
                                        } else {
                                            None
                                        };
                                        let _ = tx.send(Ok(sse_event("tool_result", serde_json::json!({
                                            "id":             call.id,
                                            "name":           call.name,
                                            "result":         result_val,
                                            "error":          error_val,
                                            "canonical_path": call.canonical_path,
                                            "trajectory_id":  &trajectory_id,
                                        })))).await;
                                    }
                                }
                            }

                            // ── usage metadata ─────────────────────────────────────
                            if let Some(ref usage) = step.usage_metadata {
                                let _ = tx.send(Ok(sse_event("usage", serde_json::json!({
                                    "prompt_token_count":      usage.prompt_token_count,
                                    "candidates_token_count":  usage.candidates_token_count,
                                    "total_token_count":       usage.total_token_count,
                                    "thoughts_token_count":    usage.thoughts_token_count,
                                })))).await;
                            }

                            // ── compaction ─────────────────────────────────────────
                            if step.r#type == StepType::Compaction {
                                let _ = tx.send(Ok(sse_event("compaction", serde_json::json!({
                                    "step_index": step_index,
                                    "trajectory_id": &trajectory_id,
                                })))).await;
                            }

                            // ── finish (structured output) ─────────────────────────
                            if step.r#type == StepType::Finish {
                                let _ = tx.send(Ok(sse_event("finish", serde_json::json!({
                                    "structured_output": step.structured_output,
                                    "text": step.content,
                                    "trajectory_id": &trajectory_id,
                                })))).await;
                            }

                            // ── status update ──────────────────────────────────────
                            let status_str = match step.status {
                                StepStatus::Active         => Some("ACTIVE"),
                                StepStatus::Done           => Some("DONE"),
                                StepStatus::Error          => Some("ERROR"),
                                StepStatus::WaitingForUser => Some("WAITING_FOR_USER"),
                                StepStatus::Canceled       => Some("CANCELED"),
                                StepStatus::TerminalError  => Some("TERMINAL_ERROR"),
                                StepStatus::Unknown        => None,
                            };
                            if let Some(status) = status_str {
                                let _ = tx.send(Ok(sse_event("status", serde_json::json!({
                                    "step_index": step_index,
                                    "status":     status,
                                    "trajectory_id": &trajectory_id,
                                })))).await;
                            }

                            // ── question (WAITING_FOR_USER with ask_question content) ──
                            // NOTE: tool-confirm WAITING_FOR_USER is handled by the
                            // concurrent confirm-task above — no blocking needed here.
                            if step.status == StepStatus::WaitingForUser
                                && !trajectory_id.is_empty()
                            {
                                let questions: Vec<AskQuestionEntry> =
                                    serde_json::from_str(&step.content).unwrap_or_default();

                                if !questions.is_empty() {
                                    tracing::info!(
                                        count = questions.len(),
                                        "stream-handler: agent question"
                                    );
                                    let _ = tx.send(Ok(sse_event("question", serde_json::json!({
                                        "trajectory_id": trajectory_id,
                                        "step_index":    step_index,
                                        "questions":     questions,
                                    })))).await;

                                    let (oneshot_tx, oneshot_rx) =
                                        tokio::sync::oneshot::channel::<QuestionHookResult>();
                                    {
                                        let mut pq = pending_question.lock().await;
                                        *pq = Some(oneshot_tx);
                                    }
                                    if let Ok(answer) = oneshot_rx.await {
                                        if let Err(e) = conn
                                            .send_question_response(&trajectory_id, step_index, answer)
                                            .await
                                        {
                                            let _ = tx.send(Ok(sse_event("error", serde_json::json!({
                                                "message": format!("Failed to send question response: {e}"),
                                                "http_code": 500,
                                            })))).await;
                                        }
                                    }
                                } else {
                                    // Tool-confirm WaitingForUser — write latest context to
                                    // the watch slot. watch::Sender::send() never blocks.
                                    tracing::debug!(
                                        step_index,
                                        tools = step.tool_calls.len(),
                                        "stream-handler: WaitingForUser (tool confirm) — handled by confirm-task"
                                    );
                                    let _ = ctx_watch_tx.send((trajectory_id.clone(), step_index));
                                }
                            }

                            // ── error step ─────────────────────────────────────────
                            if !step.error.is_empty() {
                                tracing::warn!(err = %step.error, "stream-handler: step error field");
                                let _ = tx.send(Ok(sse_event("error", serde_json::json!({
                                    "message":   step.error,
                                    "http_code": step.http_code,
                                })))).await;
                            }
                        }
                    }
                }

                tracing::debug!("stream-handler: step stream ended — sending idle+done");
                let _ = tx.send(Ok(sse_event("idle", serde_json::json!({})))).await;
                let _ = tx.send(Ok(sse_event("done", serde_json::json!({})))).await;

                // Abort the confirm-task so it releases the Arc<Mutex<Receiver>> lock.
                // Without this the task sits in recv() forever holding the lock, and
                // the next turn's confirm-task can never acquire it → deadlock.
                confirm_task.abort();
            });
        }
    }

    let stream = futures_util::stream::unfold(rx, |mut rx| async move {
        rx.recv().await.map(|val| (val, rx))
    });

    Sse::new(stream).keep_alive(
        // Send a SSE comment (`: ping`) every 15 s while the stream is open.
        // This prevents the browser (and any intermediate proxy/WASM layer) from
        // treating a silent confirm-wait as an idle connection and dropping it.
        KeepAlive::new()
            .interval(std::time::Duration::from_secs(15))
            .text("ping"),
    )
}

// ---------------------------------------------------------------------------
// GET /resolve/folder?name=<folder-name>
// Locate directories on this machine matching the given name.
// Called after the browser's showDirectoryPicker() returns a handle.name.
// ---------------------------------------------------------------------------

async fn resolve_folder_handler(
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let name = match params.get("name") {
        Some(n) if !n.trim().is_empty() => n.trim().to_string(),
        _ => return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "name parameter is required" })),
        ),
    };

    let home_dir = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());

    // Search under $HOME, max depth 6, skip hidden dirs, take first 10 matches
    let output = tokio::process::Command::new("find")
        .args([
            &home_dir,
            "-maxdepth", "6",
            "-type", "d",
            "-name", &name,
            // Prune hidden directories for speed
            "!", "-path", "*/.*",
        ])
        .output()
        .await
        .unwrap_or_else(|_| std::process::Output {
            status: Default::default(),
            stdout: vec![],
            stderr: vec![],
        });

    let paths: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|l| !l.is_empty())
        .map(String::from)
        .take(10)
        .collect();

    tracing::info!("resolve_folder: '{}' -> {} match(es)", name, paths.len());

    (
        StatusCode::OK,
        Json(serde_json::json!({ "name": name, "paths": paths })),
    )
}

// ---------------------------------------------------------------------------
// GET /media — serve generated images and other local files
// ---------------------------------------------------------------------------

/// Query parameters for `GET /media`.
#[derive(Debug, Deserialize)]
struct MediaParams {
    path: String,
}

async fn media_handler(
    axum::extract::Query(params): axum::extract::Query<MediaParams>,
) -> impl IntoResponse {
    use axum::http::header;

    let path = std::path::Path::new(&params.path);

    // Security: only serve files, not directories or symlinks to directories
    if !path.is_file() {
        return (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain")],
            Vec::new(),
        )
            .into_response();
    }

    let content_type = match path.extension().and_then(|e| e.to_str()) {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    };

    match tokio::fs::read(path).await {
        Ok(bytes) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, content_type)],
            bytes,
        )
            .into_response(),
        Err(e) => {
            tracing::error!("media_handler: failed to read {:?}: {e}", path);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                [(header::CONTENT_TYPE, "text/plain")],
                Vec::new(),
            )
                .into_response()
        }
    }
}

// ---------------------------------------------------------------------------
// POST /halt
// ---------------------------------------------------------------------------

async fn halt_handler(
    State(state): State<AppState>,
    Json(req): Json<HaltRequest>,
) -> impl IntoResponse {
    let sessions = state.sessions.lock().await;
    let Some(entry_arc) = sessions.get(&req.session_id).cloned() else {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "ok": false, "error": "Session not found" })));
    };
    drop(sessions);
    let session = entry_arc.lock().await;
    match session.conversation.connection().send_halt_request().await {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))),
        Err(e) => {
            tracing::error!("Halt error: {e:?}");
            (StatusCode::CONFLICT, Json(serde_json::json!({ "ok": false, "error": format!("{e}") })))
        }
    }
}

// ---------------------------------------------------------------------------
// POST /answer
// ---------------------------------------------------------------------------

async fn answer_handler(
    State(state): State<AppState>,
    Json(req): Json<AnswerRequest>,
) -> impl IntoResponse {
    let sessions = state.sessions.lock().await;
    let Some(entry_arc) = sessions.get(&req.session_id).cloned() else {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "ok": false, "error": "Session not found" })));
    };
    drop(sessions);
    let session = entry_arc.lock().await;
    let sender = {
        let mut pq = session.pending_question.lock().await;
        pq.take()
    };

    if let Some(tx) = sender {
        let result = QuestionHookResult {
            responses: req.responses,
            cancelled: req.cancelled,
        };
        let _ = tx.send(result);
        return (StatusCode::OK, Json(serde_json::json!({ "ok": true })));
    }

    // Fallback: direct connection call
    let result = QuestionHookResult {
        responses: req.responses,
        cancelled: req.cancelled,
    };
    match session
        .conversation
        .connection()
        .send_question_response(&req.trajectory_id, req.step_index, result)
        .await
    {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))),
        Err(e) => {
            tracing::error!("Answer error: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "ok": false, "error": format!("{e}") })))
        }
    }
}

// ---------------------------------------------------------------------------
// POST /confirm
// ---------------------------------------------------------------------------

async fn confirm_handler(
    State(state): State<AppState>,
    Json(req): Json<ConfirmRequest>,
) -> impl IntoResponse {
    let sessions = state.sessions.lock().await;
    let Some(entry_arc) = sessions.get(&req.session_id).cloned() else {
        return (StatusCode::NOT_FOUND, Json(serde_json::json!({ "ok": false, "error": "Session not found" })));
    };
    drop(sessions);

    // If user chose "Allow for Session", add tool to auto_allowed so future calls skip the dialog.
    if req.accepted && req.allow_for_session {
        if let Some(ref name) = req.tool_name {
            let session = entry_arc.lock().await;
            session.auto_allowed.write().await.insert(name.clone());
            tracing::info!("Session {}: auto-allowing tool '{name}' for session", req.session_id);
        }
    }

    // Take the oneshot TX that the stream handler stored from the ConfirmHookRequest.
    // Sending on it unblocks ConfirmHook::pre_tool_call which then returns allow/deny to harness.
    let sender = {
        let session = entry_arc.lock().await;
        let mut crt = session.confirm_result_tx.lock().await;
        crt.take()
    };

    if let Some(tx) = sender {
        let _ = tx.send(req.accepted);
        tracing::info!(
            "Session {}: user {} tool '{}'",
            req.session_id,
            if req.accepted { "approved" } else { "denied" },
            req.tool_name.as_deref().unwrap_or("?"),
        );
        return (StatusCode::OK, Json(serde_json::json!({ "ok": true })));
    }

    // Fallback: if no oneshot TX (e.g. non-WRITE_TOOLS that bypass the hook),
    // send directly via the connection API.
    let result = {
        let session = entry_arc.lock().await;
        session
            .conversation
            .connection()
            .send_tool_confirmation(&req.trajectory_id, req.step_index, req.accepted)
            .await
    };
    match result {
        Ok(()) => (StatusCode::OK, Json(serde_json::json!({ "ok": true }))),
        Err(e) => {
            tracing::error!("Confirm error: {e:?}");
            (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "ok": false, "error": format!("{e}") })))
        }
    }
}



// ---------------------------------------------------------------------------
// POST /workspace — set or change a session's workspace folder
// ---------------------------------------------------------------------------

async fn set_workspace_handler(
    State(state): State<AppState>,
    Json(req): Json<SetWorkspaceRequest>,
) -> impl IntoResponse {
    // Validate the requested path
    let path = std::path::Path::new(&req.path);
    if !path.is_dir() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "ok": false,
                "error": format!("Directory not found: {}", req.path),
            })),
        );
    }

    let canonical = match path.canonicalize() {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({ "ok": false, "error": format!("{e}") })),
            );
        }
    };

    tracing::info!("Setting workspace for session {} → {canonical}", req.session_id);

    // Disconnect existing session agent (if any)
    {
        let mut sessions = state.sessions.lock().await;
        if let Some(old_entry) = sessions.remove(&req.session_id) {
            let session = old_entry.lock().await;
            let _ = session.conversation.connection().disconnect().await;
            tracing::info!("Disconnected old agent for session {}", req.session_id);
        }
    }

    // Build new agent with the chosen workspace
    match build_session(&state.config, &req.session_id, Some(canonical.clone())).await {
        Ok(entry) => {
            let entry_arc = Arc::new(Mutex::new(entry));
            let mut sessions = state.sessions.lock().await;
            sessions.insert(req.session_id.clone(), entry_arc);
            (
                StatusCode::OK,
                Json(serde_json::json!({ "ok": true, "workspace": canonical })),
            )
        }
        Err(e) => {
            tracing::error!("Failed to create session with workspace {canonical}: {e:?}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "ok": false, "error": format!("{e}") })),
            )
        }
    }
}

// ---------------------------------------------------------------------------
// GET /workspace — get current workspace for a session
// ---------------------------------------------------------------------------

async fn get_workspace_handler(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<GetWorkspaceParams>,
) -> impl IntoResponse {
    let sessions = state.sessions.lock().await;
    if let Some(entry_arc) = sessions.get(&params.session_id).cloned() {
        drop(sessions);
        let session = entry_arc.lock().await;
        (
            StatusCode::OK,
            Json(serde_json::json!({ "workspace": session.workspace_path })),
        )
    } else {
        // Return what the default would be (even if not yet created)
        let default = state
            .config
            .base_sessions_dir
            .join(&params.session_id)
            .join("workspace")
            .to_string_lossy()
            .into_owned();
        (
            StatusCode::OK,
            Json(serde_json::json!({ "workspace": default, "default": true })),
        )
    }
}

// ---------------------------------------------------------------------------
// GET /health
// ---------------------------------------------------------------------------

async fn health_handler() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({ "status": "ok" })))
}

// ---------------------------------------------------------------------------
// main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    // 1. Load .env for GEMINI_API_KEY
    dotenvy::dotenv().ok();

    // 2. Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    // 3. Read config from environment
    let port: u16 = std::env::var("AGENT_SERVER_PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let home_dir = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "/tmp".to_string());

    let app_data_dir = std::env::var("ANTIGRAVITY_APP_DATA_DIR")
        .unwrap_or_else(|_| format!("{home_dir}/.gemini/antigravity"));

    let base_sessions_dir = std::path::PathBuf::from(&app_data_dir).join("sessions");
    if let Err(e) = std::fs::create_dir_all(&base_sessions_dir) {
        tracing::warn!("Could not create sessions dir {base_sessions_dir:?}: {e}");
    }
    tracing::info!("Sessions base dir (in app_data): {base_sessions_dir:?}");

    let env_workspace = std::env::var("WORKSPACE_ROOT").ok();
    if let Some(ref ws) = env_workspace {
        tracing::info!("Global workspace override (WORKSPACE_ROOT): {ws}");
    } else {
        tracing::info!("No WORKSPACE_ROOT set — each session gets its own temp workspace");
    }

    let config = Arc::new(ServerConfig {
        harness_path: std::env::var("ANTIGRAVITY_HARNESS_PATH").ok(),
        api_key: std::env::var("GEMINI_API_KEY").ok(),
        model: std::env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-3.5-flash".to_string()),
        app_data_dir,
        base_sessions_dir: base_sessions_dir.clone(),
        env_workspace,
    });

    let app_state = AppState {
        sessions: Arc::new(Mutex::new(HashMap::new())),
        config,
    };

    // 4. Build Axum router with CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/chat/stream", get(chat_stream_handler))
        .route("/halt", post(halt_handler))
        .route("/answer", post(answer_handler))
        .route("/confirm", post(confirm_handler))
        .route("/workspace", get(get_workspace_handler).post(set_workspace_handler))
        .route("/resolve/folder", get(resolve_folder_handler))
        .route("/media", get(media_handler))
        .route("/health", get(health_handler))
        .layer(cors)
        .with_state(app_state.clone());

    // 5. Serve
    let addr = format!("127.0.0.1:{port}");
    tracing::info!("Agent sidecar listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind");
    let server = axum::serve(listener, app.into_make_service());

    // Shutdown hook: disconnect all sessions and clean up temp dirs
    let cleanup_sessions_dir = base_sessions_dir;
    let cleanup_state = app_state;
    tokio::spawn(async move {
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};
            let mut sigterm = signal(SignalKind::terminate()).expect("sigterm handler");
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = sigterm.recv() => {}
            }
        }
        #[cfg(not(unix))]
        {
            let _ = tokio::signal::ctrl_c().await;
        }
        tracing::info!("Shutdown — disconnecting all sessions...");
        let sessions = cleanup_state.sessions.lock().await;
        for (id, entry_arc) in sessions.iter() {
            let session = entry_arc.lock().await;
            let _ = session.conversation.connection().disconnect().await;
            tracing::info!("  Disconnected session {id}");
        }
        drop(sessions);
        if let Err(e) = std::fs::remove_dir_all(&cleanup_sessions_dir) {
            tracing::warn!("Could not remove sessions dir {cleanup_sessions_dir:?}: {e}");
        }
        std::process::exit(0);
    });

    server.await.expect("Server error");
}
