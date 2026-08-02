//! Local subprocess connection and negotiation.
//!
//! This module provides [`LocalConnectionStrategy`] to initialize and spawn the local
//! agent subprocess, perform the initial handshake, and transition to a WebSocket session
//! wrapped by [`LocalConnection`].

/// How long to wait for the harness's handshake reply. A pre-0.1.4 harness
/// never sends one, so this bounds that case rather than failing it.
const HANDSHAKE_TIMEOUT_SECONDS: u64 = 10;

/// How long to wait for the harness to exit after stdin closes, before
/// escalating. Upstream uses the same three minutes (`local_connection.py:53`).
const PROCESS_WAIT_TIMEOUT_SECONDS: u64 = 3 * 60;

/// How many trailing harness stderr lines to retain for crash diagnostics.
const STDERR_TAIL_LINES: usize = 20;

use crate::connection::Connection;
use crate::hooks::HookRunner;
use crate::proto::localharness::{
    ClientInfo as ProtoClientInfo, FileEditToolConfig, FilesystemWorkspace, FindToolConfig,
    GenerateImageToolConfig, GrepSearchToolConfig, HarnessConfig, HarnessSideTools,
    InitializeConversationEvent, InputConfig, InputEvent, ListDirToolConfig, MultipleChoiceAnswer,
    OutputConfig, OutputEvent, ReadUrlContentToolConfig, RunCommandToolConfig, SearchWebToolConfig,
    SubagentsConfig, SystemInstructions as ProtoSystemInstructions, Tool as ProtoTool,
    ToolConfirmation, UserQuestionAnswer, UserQuestionsConfig, UserQuestionsResponse,
    ViewFileToolConfig, Workspace as ProtoWorkspace, WriteToFileToolConfig,
    appended_system_instructions::Section, custom_system_instructions::Part,
    user_questions_response::QuestionsResponse, workspace::WorkspaceType,
};
use crate::tools::ToolRunner;
use crate::types::{
    AntigravityExecutionError, AskQuestionEntry, AskQuestionOption, BuiltinTools,
    CapabilitiesConfig, GeminiConfig, McpServerConfig, QuestionHookResult, Step, StepSource,
    StepStatus, StepTarget, StepType, SystemInstructions, ToolCall, ToolResult, UsageMetadata,
};

use anyhow::anyhow;
use futures_util::stream::{self, BoxStream};
use futures_util::{SinkExt, StreamExt};
use prost::Message;
use serde_json::Value;
use std::collections::{HashMap, HashSet, VecDeque};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_tungstenite::connect_async_with_config;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;

/// Connection strategy implementation communicating with a local subprocess harness.
///
/// `LocalConnection` handles launching and lifecycle tracking of the `localharness` binary,
/// reading standard error output, upgrading communication to standard `WebSockets`, and managing
/// subagent and tool invocation state.
#[allow(dead_code)]
pub struct LocalConnection {
    conversation_id: String,
    learned_id: Arc<std::sync::OnceLock<String>>,
    process: Arc<Mutex<tokio::process::Child>>,
    child_stdin: Arc<Mutex<Option<tokio::process::ChildStdin>>>,
    is_idle: Arc<AtomicBool>,
    step_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<crate::step_extract::StepEvent>>>>,
    ws_tx: UnboundedSender<String>,
    tool_runner: Option<ToolRunner>,
    hook_runner: Option<HookRunner>,
    step_trackers: Arc<Mutex<HashMap<(String, u32), StepTracker>>>,
    /// The trajectory whose idle transitions end a turn. Learned from the first
    /// `StepUpdate` of each turn and cleared by `send()`, mirroring upstream's
    /// `reset_for_turn()` (`event_processor.py:379-386`).
    main_trajectory_id: Arc<Mutex<Option<String>>>,
    /// Set by [`Connection::send_halt_request`], cleared by the next `send()`
    /// or by the idle transition that consumes it.
    ///
    /// The harness answers a caller-initiated halt with a plain
    /// `STATE_FULLY_IDLE`, not `STATE_CANCELLED` — so without this flag a
    /// cancelled turn is indistinguishable from a completed one, and a caller
    /// that halts mid-turn sees the stream end as if the model had finished.
    cancel_requested: Arc<AtomicBool>,
    /// Whether a `receive_steps()` stream is currently live. See that method.
    steps_consumed: Arc<AtomicBool>,
    /// Mirrors `is_idle` for [`Connection::wait_for_idle`].
    idle_tx: tokio::sync::watch::Sender<bool>,
    /// Last model text per subagent trajectory; see the capture site in the
    /// reader loop. Cleared per turn.
    subagent_responses: Arc<Mutex<HashMap<String, String>>>,
    /// Steps the harness replayed in its handshake reply, for a resumed
    /// conversation. Seeding `Conversation` with these is the remaining
    /// half of WP-6.
    initial_history: Vec<Step>,
}

impl LocalConnection {
    /// Steps the harness replayed when the conversation was resumed.
    ///
    /// Empty for a new conversation, and for any harness older than 0.1.4.
    #[must_use]
    pub fn initial_history(&self) -> &[Step] {
        &self.initial_history
    }
}

impl std::fmt::Debug for LocalConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalConnection")
            .field("conversation_id", &self.conversation_id())
            .field("is_idle", &self.is_idle)
            .field("tool_runner", &self.tool_runner)
            .field("hook_runner", &self.hook_runner)
            .field("step_trackers", &self.step_trackers)
            .finish_non_exhaustive()
    }
}

impl Connection for LocalConnection {
    fn conversation_id(&self) -> &str {
        if self.conversation_id.is_empty() {
            self.learned_id.get().map_or("", String::as_str)
        } else {
            &self.conversation_id
        }
    }

    fn is_idle(&self) -> bool {
        self.is_idle.load(Ordering::SeqCst)
    }

    async fn wait_for_idle(&self) {
        if self.is_idle() {
            return;
        }
        // Watch rather than poll: the reader sets this the moment the harness
        // reports idle, so a caller learns immediately instead of on the next
        // tick of a sleep loop.
        let mut rx = self.idle_tx.subscribe();
        let _ = rx.wait_for(|idle| *idle).await;
    }

    fn receive_steps(&self) -> BoxStream<'static, Result<Step, anyhow::Error>> {
        // One consumer at a time. Two live streams share a single receiver, so
        // each would take roughly half the steps and neither caller would see a
        // complete turn — silently. Refusing is the only honest answer; the
        // claim is released when the first stream is dropped, which is what
        // makes the per-turn `receive_steps()` call still work.
        let Some(claim) = crate::step_extract::ConsumerGuard::claim(&self.steps_consumed) else {
            return stream::once(async {
                Err(anyhow!(
                    "receive_steps() is single-consumer and a stream is already active; \
                     drop it before subscribing again"
                ))
            })
            .boxed();
        };
        let step_rx = self.step_rx.clone();
        let is_idle = self.is_idle.clone();
        stream::unfold(claim, move |claim| {
            let step_rx = step_rx.clone();
            let is_idle = is_idle.clone();
            async move {
                loop {
                    // Head condition, upstream local_connection.py:339-341: the
                    // stream ends only when the connection is idle AND nothing
                    // is queued behind the idle event. Returning on the idle
                    // event itself drops every step queued after it.
                    let mut guard = step_rx.lock().await;
                    let Some(rx) = &mut *guard else {
                        drop(guard);
                        return None;
                    };
                    if is_idle.load(Ordering::SeqCst) && rx.is_empty() {
                        drop(guard);
                        return None;
                    }
                    let received = rx.recv().await;
                    drop(guard);

                    match received {
                        None => return None,
                        // Falls through to re-evaluate the head condition
                        // rather than ending the stream: more steps may already
                        // be queued behind the idle marker.
                        Some(crate::step_extract::StepEvent::Idle) => {}
                        Some(crate::step_extract::StepEvent::Step(step)) => {
                            return Some((Ok(*step), claim));
                        }
                        Some(crate::step_extract::StepEvent::Error(e)) => {
                            return Some((Err(e), claim));
                        }
                    }
                }
            }
        })
        .boxed()
    }

    async fn send(&self, content: &str) -> Result<(), anyhow::Error> {
        // Before any state is touched: a denied turn must leave the connection
        // exactly as it was, not half-reset with a cleared trajectory.
        crate::hook_dispatch::gate_turn(self.hook_runner.as_ref()).await?;

        self.is_idle.store(false, Ordering::SeqCst);
        let _ = self.idle_tx.send(false);
        // A halt applies to the turn it interrupted. Leaving the flag set would
        // make the *next* turn report itself cancelled the moment it went idle.
        self.cancel_requested.store(false, Ordering::SeqCst);
        {
            // A new turn may run on a new trajectory; relearn it rather than
            // judging this turn against the last one's (upstream
            // reset_for_turn(), event_processor.py:379-386).
            let mut main_id = self.main_trajectory_id.lock().await;
            *main_id = None;
        }
        {
            // Last turn's subagent text must not be attributed to this turn's
            // subagents (upstream clears the same map in send()).
            self.subagent_responses.lock().await.clear();
        }
        {
            let mut guard = self.step_rx.lock().await;
            if let Some(rx) = &mut *guard {
                while rx.try_recv().is_ok() {}
            }
        }

        let input_event = InputEvent {
            event: Some(crate::proto::localharness::input_event::Event::UserInput(
                crate::harness_config::sanitize_prompt(content),
            )),
        };
        let raw_json = serde_json::to_string(&input_event)?;
        self.ws_tx.send(raw_json)?;
        Ok(())
    }

    async fn send_trigger_notification(&self, content: &str) -> Result<(), anyhow::Error> {
        let input_event = InputEvent {
            event: Some(
                crate::proto::localharness::input_event::Event::AutomatedTrigger(
                    content.to_string(),
                ),
            ),
        };
        let raw_json = serde_json::to_string(&input_event)?;
        self.ws_tx.send(raw_json)?;
        Ok(())
    }

    async fn send_halt_request(&self) -> Result<(), anyhow::Error> {
        self.cancel_requested.store(true, Ordering::SeqCst);
        let input_event = InputEvent {
            event: Some(crate::proto::localharness::input_event::Event::HaltRequest(
                true,
            )),
        };
        let raw_json = serde_json::to_string(&input_event)?;
        self.ws_tx.send(raw_json)?;
        Ok(())
    }

    async fn send_tool_confirmation(
        &self,
        trajectory_id: &str,
        step_index: u32,
        accepted: bool,
    ) -> Result<(), anyhow::Error> {
        let conf = ToolConfirmation {
            trajectory_id: Some(trajectory_id.to_string()),
            step_index: Some(step_index),
            accepted: Some(accepted),
        };
        let input_event = InputEvent {
            event: Some(crate::proto::localharness::input_event::Event::ToolConfirmation(conf)),
        };
        let raw_json = serde_json::to_string(&input_event)?;
        self.ws_tx.send(raw_json)?;
        Ok(())
    }

    async fn send_tool_response(&self, id: &str, result: ToolResult) -> Result<(), anyhow::Error> {
        let resp = crate::tool_wire::tool_response(Some(id.to_string()), &result);
        let input_event = InputEvent {
            event: Some(crate::proto::localharness::input_event::Event::ToolResponse(resp)),
        };
        let raw_json = serde_json::to_string(&input_event)?;
        self.ws_tx.send(raw_json)?;
        Ok(())
    }

    async fn send_question_response(
        &self,
        trajectory_id: &str,
        step_index: u32,
        answers: QuestionHookResult,
    ) -> Result<(), anyhow::Error> {
        let mut proto_answers = Vec::new();
        for r in answers.responses {
            if r.skipped {
                proto_answers.push(UserQuestionAnswer {
                    answer: Some(
                        crate::proto::localharness::user_question_answer::Answer::Unanswered(true),
                    ),
                });
            } else {
                let mut mc_ans = MultipleChoiceAnswer {
                    selected_choice_indices: Vec::new(),
                    freeform_response: Some(r.freeform_response.clone()),
                };
                if let Some(ref opts) = r.selected_option_ids {
                    for opt in opts {
                        if let Ok(idx) = opt.parse::<i32>() {
                            mc_ans.selected_choice_indices.push(idx - 1);
                        }
                    }
                }
                proto_answers.push(UserQuestionAnswer {
                    answer: Some(crate::proto::localharness::user_question_answer::Answer::MultipleChoiceAnswer(mc_ans)),
                });
            }
        }

        let resp = UserQuestionsResponse {
            trajectory_id: Some(trajectory_id.to_string()),
            step_index: Some(step_index),
            result: Some(if answers.cancelled {
                crate::proto::localharness::user_questions_response::Result::Cancelled(true)
            } else {
                crate::proto::localharness::user_questions_response::Result::Response(
                    QuestionsResponse {
                        answers: proto_answers,
                    },
                )
            }),
        };

        let input_event = InputEvent {
            event: Some(crate::proto::localharness::input_event::Event::QuestionResponse(resp)),
        };
        let raw_json = serde_json::to_string(&input_event)?;
        self.ws_tx.send(raw_json)?;
        Ok(())
    }

    async fn disconnect(&self) -> Result<(), anyhow::Error> {
        // Upstream dispatches session_end from disconnect()
        // (0.1.1 local_connection.py:686-690). This crate defined the
        // dispatcher and never called it, so on_session_end hooks silently
        // never ran. Dispatched before teardown so a hook can still observe a
        // live connection; a failing hook must not block shutdown.
        if let Some(ref runner) = self.hook_runner
            && let Err(e) = runner.dispatch_session_end().await
        {
            tracing::error!("on_session_end hook failed: {e:?}");
        }

        // Ordered shutdown, mirroring upstream local_connection.py:407-455.
        // A bare kill() runs no Go defers, so cleanupAllAgents never runs and
        // the trajectory is never written to disk -- upstream's own tests spell
        // this out (local_connection_test.py:3110-3130).
        //
        // Closing stdin is the actual signal: the harness monitors it for EOF.
        {
            let mut stdin = self.child_stdin.lock().await;
            drop(stdin.take());
        }

        let mut proc = self.process.lock().await;
        match tokio::time::timeout(
            std::time::Duration::from_secs(PROCESS_WAIT_TIMEOUT_SECONDS),
            proc.wait(),
        )
        .await
        {
            Ok(Ok(_)) => {}
            // Exited badly, or took too long: escalate rather than hang.
            _ => {
                let _ = proc.kill().await;
            }
        }
        drop(proc);
        Ok(())
    }
}

/// Where the harness stores its state when the caller named no `save_dir`.
///
/// Per-conversation so two concurrent agents do not share a directory.
fn default_save_dir(conversation_id: &str) -> String {
    let leaf = if conversation_id.is_empty() {
        "antigravity-session".to_string()
    } else {
        format!("antigravity-{conversation_id}")
    };
    std::env::temp_dir()
        .join(leaf)
        .to_string_lossy()
        .into_owned()
}

/// Configurator and builder to spawn a local helper subprocess and build a connection.
#[derive(Debug)]
pub struct LocalConnectionStrategy {
    /// Path to the `localharness` binary.
    pub binary_path: String,
    /// Gemini configuration parameters.
    pub gemini_config: GeminiConfig,
    /// Capability config specifying enabled tools.
    pub capabilities_config: CapabilitiesConfig,
    /// System instruction content or templates.
    pub system_instructions: Option<SystemInstructions>,
    /// Optional directory to store state and execution logs.
    pub save_dir: Option<String>,
    /// Workspace paths.
    pub workspaces: Vec<String>,
    /// Folders containing custom skill modules.
    pub skills_paths: Vec<String>,
    /// Optional coordinator runner to handle custom tools.
    pub tool_runner: Option<ToolRunner>,
    /// Optional coordinator runner to handle lifecycle hooks.
    pub hook_runner: Option<HookRunner>,
    /// Conversation ID for standard session resuming or tracking.
    pub conversation_id: String,
    /// How the conversation attaches to harness-side session state.
    pub session_continuation_mode: Option<crate::types::SessionContinuationMode>,
    /// MCP server configurations.
    pub mcp_servers: Vec<McpServerConfig>,
    /// Named subagents, emitted on `HarnessConfig.custom_subagents`.
    ///
    /// Not a field of [`new`](Self::new) — set it on the struct.
    pub subagents: Vec<crate::types::SubagentConfig>,
    /// Extra environment for the harness process, sent on `InputConfig.env`.
    ///
    /// Not a field of [`new`](Self::new) — set it on the struct. The harness
    /// also inherits this process's environment; these are additions on top.
    pub env: HashMap<String, String>,
}

impl LocalConnectionStrategy {
    /// Creates a new `LocalConnectionStrategy`.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        binary_path: String,
        gemini_config: GeminiConfig,
        capabilities_config: CapabilitiesConfig,
        system_instructions: Option<SystemInstructions>,
        save_dir: Option<String>,
        workspaces: Vec<String>,
        skills_paths: Vec<String>,
        tool_runner: Option<ToolRunner>,
        hook_runner: Option<HookRunner>,
        conversation_id: String,
        session_continuation_mode: Option<crate::types::SessionContinuationMode>,
        mcp_servers: Vec<McpServerConfig>,
    ) -> Self {
        Self {
            binary_path,
            gemini_config,
            capabilities_config,
            system_instructions,
            save_dir,
            workspaces,
            skills_paths,
            tool_runner,
            hook_runner,
            conversation_id,
            session_continuation_mode,
            mcp_servers,
            subagents: Vec::new(),
            env: HashMap::new(),
        }
    }

    /// Spawns the subprocess helper binary, performs length-prefixed initialization,
    /// upgrades connection to WebSocket, and builds the stateful [`LocalConnection`].
    ///
    /// # Errors
    ///
    /// Returns an error if the subprocess cannot be launched, the handshake fails,
    /// or the WebSocket upgrade fails.
    #[allow(clippy::too_many_lines)]
    pub async fn connect(&self) -> Result<LocalConnection, anyhow::Error> {
        let use_vertex = self.gemini_config.vertex;
        let api_key = self
            .gemini_config
            .models
            .default
            .api_key
            .clone()
            .or_else(|| self.gemini_config.api_key.clone())
            .or_else(|| std::env::var("GEMINI_API_KEY").ok());

        if !use_vertex && api_key.is_none() {
            return Err(anyhow!(
                "A Gemini API key is required. Set it via GeminiConfig or GEMINI_API_KEY env var."
            ));
        }

        if use_vertex {
            let has_project = self.gemini_config.project.is_some();
            let has_location = self.gemini_config.location.is_some();
            if api_key.is_none() && !(has_project && has_location) {
                return Err(anyhow!(
                    "For Vertex AI, either a GCP project and location, or an API key \
                     (Express Mode) must be set."
                ));
            }
        }

        // Resolved for validation only; the value reaches the wire through
        // build_models_proto, and the harness also reads GEMINI_API_KEY from
        // the environment it inherits.
        let _api_key = api_key.unwrap_or_default();

        // 1. Spawning localharness subprocess
        // Explicitly forward SHELL and PATH so the harness can fork /bin/sh for
        // RUN_COMMAND steps regardless of what working directory is used.
        // Without this, the harness may fail with "fork/exec /bin/sh: no such file
        // or directory" when running commands in absolute-path subdirectories.
        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let path = std::env::var("PATH")
            .unwrap_or_else(|_| "/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin".to_string());
        let mut child = Command::new(&self.binary_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .env("SHELL", &shell)
            .env("PATH", &path)
            .spawn()?;

        let mut child_stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Failed to open child stdin"))?;
        let mut child_stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to open child stdout"))?;
        let child_stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("Failed to open child stderr"))?;

        // 2. Perform Handshake via length-prefixed protocol buffer over stdin/stdout
        let client_info = ProtoClientInfo {
            os: Some(std::env::consts::OS.to_string()),
            os_version: Some(crate::harness_config::os_version()),
            language: Some("rust".to_string()),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            language_version: Some(rustc_version()),
        };

        let input_config = InputConfig {
            env: self.env.clone(),
            // A harness with nowhere to write puts its state next to whatever
            // its working directory happens to be. Defaulting to a per-session
            // temp directory keeps that out of the caller's repository.
            storage_directory: Some(
                self.save_dir
                    .clone()
                    .unwrap_or_else(|| default_save_dir(&self.conversation_id)),
            ),
            port: None,
            bind_address: None,
            client_info: Some(client_info),
        };

        let mut input_buf = Vec::new();
        input_config.encode(&mut input_buf)?;

        let size = input_buf.len() as u32;
        child_stdin.write_all(&size.to_le_bytes()).await?;
        child_stdin.write_all(&input_buf).await?;
        child_stdin.flush().await?;

        let mut size_bytes = [0u8; 4];
        child_stdout.read_exact(&mut size_bytes).await?;
        let length = u32::from_le_bytes(size_bytes) as usize;
        let mut output_buf = vec![0u8; length];
        child_stdout.read_exact(&mut output_buf).await?;
        let output_config = OutputConfig::decode(&output_buf[..])?;

        let port = output_config
            .port
            .ok_or_else(|| anyhow!("Harness OutputConfig missing port"))?;
        let harness_api_key = output_config
            .api_key
            .ok_or_else(|| anyhow!("Harness OutputConfig missing api_key"))?;

        // 3. Setup WebSocket connection
        //
        // The harness binds 127.0.0.1. On a host where `localhost` resolves to
        // ::1 first, every attempt against the name fails with connection
        // refused while the literal works — so both are tried, alternating, and
        // the error names whichever was tried last.
        let ws_urls = [
            format!("ws://localhost:{port}/"),
            format!("ws://127.0.0.1:{port}/"),
        ];
        let mut requests = Vec::with_capacity(ws_urls.len());
        for url in &ws_urls {
            let mut req = url.clone().into_client_request()?;
            req.headers_mut()
                .insert("x-goog-api-key", harness_api_key.parse()?);
            requests.push(req);
        }

        // Connect with retry/backoff
        let mut ws_stream = None;
        let mut delay = std::time::Duration::from_millis(100);
        for attempt in 0..5 {
            let ws_url = &ws_urls[attempt % ws_urls.len()];
            let req = requests[attempt % requests.len()].clone();
            // Tool results and file contents routinely exceed tungstenite's
            // default 16 MiB frame / 64 MiB message caps, and upstream sets
            // max_size=None for exactly that reason
            // (local_connection.py:1086-1092). Hitting the cap kills the
            // connection mid-turn rather than truncating.
            let ws_config = WebSocketConfig {
                max_message_size: None,
                max_frame_size: None,
                ..WebSocketConfig::default()
            };
            match connect_async_with_config(req, Some(ws_config), false).await {
                Ok((stream, _)) => {
                    ws_stream = Some(stream);
                    break;
                }
                Err(e) => {
                    if attempt == 4 {
                        let _ = child.kill().await;
                        return Err(anyhow!("Failed to connect to WS at {ws_url}: {e:?}"));
                    }
                    tokio::time::sleep(delay).await;
                    delay *= 2;
                }
            }
        }

        let ws = ws_stream.ok_or_else(|| anyhow!("Failed to connect to WS"))?;
        let (mut ws_write, mut ws_read) = ws.split();

        // 4. Build HarnessConfig proto
        let mut proto_tools = Vec::new();
        let mut registered_tool_names: Vec<String> = Vec::new();
        if let Some(ref runner) = self.tool_runner {
            let tools = runner.tools.read().await;
            for t in tools.iter() {
                registered_tool_names.push(t.name().to_string());
                proto_tools.push(ProtoTool {
                    name: Some(t.name().to_string()),
                    description: Some(t.description().to_string()),
                    parameters_json_schema: Some(t.parameters_json_schema().to_string()),
                    response_json_schema: None,
                    // Deferred tool loading is a 0.1.9 capability this crate
                    // does not use yet (audit W27).
                    defer_loading: None,
                });
            }
        }

        let proto_sys = self.system_instructions.as_ref().map(|sys| match sys {
            SystemInstructions::Custom(custom) => {
                let instr_part = Part {
                    part: Some(
                        crate::proto::localharness::custom_system_instructions::part::Part::Text(
                            custom.text.clone(),
                        ),
                    ),
                };
                ProtoSystemInstructions {
                    r#type: Some(
                        crate::proto::localharness::system_instructions::Type::Custom(
                            crate::proto::localharness::CustomSystemInstructions {
                                part: vec![instr_part],
                            },
                        ),
                    ),
                }
            }
            SystemInstructions::Appended(appended) => {
                let mut sections = Vec::new();
                for sec in &appended.appended_sections {
                    sections.push(Section {
                        title: Some(sec.title.clone()),
                        content: Some(sec.content.clone()),
                    });
                }
                ProtoSystemInstructions {
                    r#type: Some(
                        crate::proto::localharness::system_instructions::Type::Appended(
                            crate::proto::localharness::AppendedSystemInstructions {
                                custom_identity: appended.custom_identity.clone(),
                                appended_sections: sections,
                            },
                        ),
                    ),
                }
            }
        });

        let mut proto_workspaces = Vec::new();
        for w in &self.workspaces {
            proto_workspaces.push(ProtoWorkspace {
                workspace_type: Some(WorkspaceType::FilesystemWorkspace(FilesystemWorkspace {
                    // Upstream normalizes on the way out (0.1.1
                    // local_connection.py:1418), so the harness and the client-side
                    // policy layer scope the same directories.
                    directory: Some(crate::wire_path::normalize_wire_path(w)),
                })),
            });
        }

        let all_tools = vec![
            BuiltinTools::CreateFile,
            BuiltinTools::EditFile,
            BuiltinTools::FindFile,
            BuiltinTools::ListDir,
            BuiltinTools::RunCommand,
            BuiltinTools::SearchDir,
            BuiltinTools::ViewFile,
            BuiltinTools::StartSubagent,
            BuiltinTools::GenerateImage,
            BuiltinTools::Finish,
        ];
        let active_tools: HashSet<BuiltinTools> =
            self.capabilities_config.enabled_tools.as_ref().map_or_else(
                || {
                    if let Some(ref disabled) = self.capabilities_config.disabled_tools {
                        let disabled_set: HashSet<BuiltinTools> =
                            disabled.iter().copied().collect();
                        all_tools
                            .into_iter()
                            .filter(|t| !disabled_set.contains(t))
                            .collect()
                    } else {
                        all_tools.into_iter().collect()
                    }
                },
                |enabled| enabled.iter().copied().collect(),
            );

        let side_tools = HarnessSideTools {
            // A tool like any other: absent would leave the harness to guess,
            // and a caller who listed `enabled_tools` had no way to turn either
            // on or off (C6).
            search_web: Some(SearchWebToolConfig {
                enabled: Some(active_tools.contains(&BuiltinTools::SearchWeb)),
            }),
            read_url_content: Some(ReadUrlContentToolConfig {
                enabled: Some(active_tools.contains(&BuiltinTools::ReadUrlContent)),
            }),
            tool_search_config: None,
            find: Some(FindToolConfig {
                enabled: Some(active_tools.contains(&BuiltinTools::FindFile)),
            }),
            run_command: Some(RunCommandToolConfig {
                enabled: Some(active_tools.contains(&BuiltinTools::RunCommand)),
            }),
            subagents: Some(SubagentsConfig {
                enabled: Some(active_tools.contains(&BuiltinTools::StartSubagent)),
            }),
            user_questions: Some(UserQuestionsConfig {
                // Was hardcoded true, so a caller who listed `enabled_tools`
                // explicitly still got the question panel and no way to turn it
                // off. It is a tool like any other.
                enabled: Some(active_tools.contains(&BuiltinTools::AskQuestion)),
            }),
            file_edit: Some(FileEditToolConfig {
                enabled: Some(active_tools.contains(&BuiltinTools::EditFile)),
            }),
            view_file: Some(ViewFileToolConfig {
                enabled: Some(active_tools.contains(&BuiltinTools::ViewFile)),
            }),
            write_to_file: Some(WriteToFileToolConfig {
                enabled: Some(active_tools.contains(&BuiltinTools::CreateFile)),
            }),
            grep_search: Some(GrepSearchToolConfig {
                enabled: Some(active_tools.contains(&BuiltinTools::SearchDir)),
            }),
            list_dir: Some(ListDirToolConfig {
                enabled: Some(active_tools.contains(&BuiltinTools::ListDir)),
            }),
            permissions: None,
            generate_image: Some(GenerateImageToolConfig {
                enabled: Some(active_tools.contains(&BuiltinTools::GenerateImage)),
            }),
        };

        let harness_config = HarnessConfig {
            cascade_id: Some(self.conversation_id.clone()),
            // Each of these is its own work package (WP-6 session continuation
            // and retry, WP-8 hooks, WP-9 MCP and subagents). Explicitly unset
            // so `cargo build` flags them again when those land.
            session_continuation_mode: self
                .session_continuation_mode
                .map(crate::types::SessionContinuationMode::as_proto),
            retry_config: None,
            enabled_hooks: Vec::new(),
            custom_subagents: crate::harness_config::build_custom_subagents_proto(
                &self.subagents,
                &registered_tool_names,
            )?,
            mcp_servers: crate::harness_config::build_mcp_servers_proto(&self.mcp_servers),
            tool_output_truncation: None,
            models: crate::harness_config::build_models_proto(
                &self.gemini_config,
                self.capabilities_config.image_model.as_deref(),
            )?,
            system_instructions: proto_sys,
            tools: proto_tools,
            harness_side_tools: Some(side_tools),
            compaction_threshold: self.capabilities_config.compaction_threshold,
            workspaces: proto_workspaces,
            skills_paths: self.skills_paths.clone(),
            finish_tool_schema_json: self.capabilities_config.finish_tool_schema_json.clone(),
            initial_trajectory: None,
            app_data_dir: self.save_dir.clone(),
        };

        // 5. Send InitializeConversationEvent
        let init_event = InitializeConversationEvent {
            config: Some(harness_config),
        };
        let init_json = serde_json::to_string(&init_event)?;
        ws_write.send(WsMessage::Text(init_json)).await?;

        // Read the handshake reply before anything else. Since 0.1.4 the harness
        // answers InitializeConversationEvent with an OutputEvent carrying
        // initialize_conversation_response, and upstream blocks on it
        // (local_connection.py:1162-1176). Skipping it leaves the frame to be
        // picked up by the step reader, where it is not a step.
        //
        // `cascade_id` from the response is deliberately ignored: upstream takes
        // the conversation id from the first StepUpdate's trajectory_id instead
        // (event_processor.py:478-480).
        let initial_history: Vec<Step> = match tokio::time::timeout(
            std::time::Duration::from_secs(HANDSHAKE_TIMEOUT_SECONDS),
            ws_read.next(),
        )
        .await
        {
            Ok(Some(Ok(WsMessage::Text(raw)))) => {
                match serde_json::from_str::<OutputEvent>(&raw) {
                    Ok(OutputEvent {
                        event:
                            Some(crate::proto::localharness::output_event::Event::InitializeConversationResponse(
                                resp,
                            )),
                        ..
                    }) => resp
                        .history
                        .iter()
                        .filter_map(crate::step_extract::step_from_update)
                        .collect(),
                    Ok(_) => {
                        // A harness that answers with something else is not one
                        // we understand; surfacing it beats guessing.
                        tracing::warn!("first frame was not initialize_conversation_response");
                        Vec::new()
                    }
                    Err(e) => {
                        return Err(anyhow!(
                            "could not parse the harness handshake reply: {e}. This usually means \
                             the harness is a different version than proto/localharness.proto was \
                             generated from — see scripts/gen_proto.py."
                        ));
                    }
                }
            }
            Ok(Some(Err(e))) => return Err(anyhow!("harness closed during handshake: {e}")),
            Ok(None) => return Err(anyhow!("harness closed the socket during handshake")),
            // Pre-0.1.4 harnesses never answer. Continuing keeps this SDK working
            // against the version scripts/install_harness.sh still pins.
            Err(_) => {
                tracing::debug!("no handshake reply within {HANDSHAKE_TIMEOUT_SECONDS}s");
                Vec::new()
            }
            Ok(Some(Ok(_))) => Vec::new(),
        };

        // 6. Spawn Background WS Sender Loop
        let (ws_tx, mut ws_rx) = mpsc::unbounded_channel::<String>();
        tokio::spawn(async move {
            while let Some(msg) = ws_rx.recv().await {
                if let Err(e) = ws_write.send(WsMessage::Text(msg)).await {
                    tracing::error!("WS Write Loop Error: {:?}", e);
                    break;
                }
            }
        });

        // 7. Setup channels for step stream
        let (step_tx, step_rx) = mpsc::unbounded_channel::<crate::step_extract::StepEvent>();
        let client_tool_step_counter = Arc::new(AtomicU32::new(50_000));

        // NOTE: upstream starts idle here, and the loop restructure in
        // receive_steps() removed the first-poll hazard that previously blocked
        // this. It still cannot flip, for a second reason: a caller that polls
        // receive_steps() on a fresh connection — before the reader has seen the
        // harness's STATE_RUNNING — would race, see idle with an empty queue,
        // and get an empty stream. Upstream's API is send()-then-receive, which
        // hides this; ours does not promise that yet. Flipping it needs the
        // connect-time race closed first (see C2 in docs/remaining-work.md).
        let is_idle = Arc::new(AtomicBool::new(false));
        let (idle_tx, _idle_rx) = tokio::sync::watch::channel(false);
        let conn_idle_tx = idle_tx.clone();
        let cancel_requested = Arc::new(AtomicBool::new(false));
        let step_trackers = Arc::new(Mutex::new(HashMap::new()));
        // Last model text seen on each subagent trajectory, so the
        // `post_tool_call` that fires when the subagent finishes can carry what
        // it produced (upstream `_subagent_responses`).
        let subagent_responses: Arc<Mutex<HashMap<String, String>>> =
            Arc::new(Mutex::new(HashMap::new()));
        let conn_subagent_responses = subagent_responses.clone();

        let conn_ws_tx = ws_tx.clone();
        let conn_is_idle = is_idle.clone();
        let conn_is_idle_for_close = is_idle.clone();
        let conn_cancel_requested = cancel_requested.clone();
        let conn_step_trackers = step_trackers.clone();

        let tool_runner = self.tool_runner.clone();
        let hook_runner = self.hook_runner.clone();
        let conversation_id = self.conversation_id.clone();
        let learned_id = Arc::new(std::sync::OnceLock::new());
        let conn_learned_id = learned_id.clone();
        let conn_cascade_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let conn_cascade_id_for_ws = conn_cascade_id.clone();

        // 8. Spawn Stderr Reader
        //
        // The tail is retained rather than only logged: when the harness dies
        // the websocket simply closes, and the reason it died is in these lines.
        // Discarding them left a crash indistinguishable from a clean end of
        // stream (`harness-crash-diagnostics`).
        let stderr_tail: Arc<Mutex<VecDeque<String>>> =
            Arc::new(Mutex::new(VecDeque::with_capacity(STDERR_TAIL_LINES)));
        let reader_stderr_tail = stderr_tail.clone();
        let mut reader = tokio::io::BufReader::new(child_stderr);
        tokio::spawn(async move {
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 {
                    break;
                }
                let trimmed = line.trim_end().to_string();
                tracing::info!("Harness stderr: {trimmed}");
                {
                    let mut tail = reader_stderr_tail.lock().await;
                    if tail.len() == STDERR_TAIL_LINES {
                        tail.pop_front();
                    }
                    tail.push_back(trimmed);
                }
                line.clear();
            }
        });

        // 9. Spawn WS Reader Loop
        let pending_builtin_tool_calls =
            Arc::new(Mutex::new(HashMap::<(String, u32), ToolCall>::new()));
        tokio::spawn(async move {
            while let Some(msg_res) = ws_read.next().await {
                match msg_res {
                    Ok(WsMessage::Text(raw_text)) => {
                        tracing::debug!("WS raw message: {}", &raw_text[..raw_text.len().min(500)]);
                        match serde_json::from_str::<OutputEvent>(&raw_text) {
                            Ok(output_event) => {
                                if let Some(event) = output_event.event {
                                    match event {
                                        crate::proto::localharness::output_event::Event::StepUpdate(step_update) => {
                                            let traj_id = step_update.trajectory_id.clone().unwrap_or_default();
                                            let step_idx = step_update.step_index.unwrap_or(0);
                                            let key = (traj_id.clone(), step_idx);

                                            // The main trajectory is whichever one reports first,
                                            // unconditionally — upstream event_processor.py:478-480.
                                            // The previous rule also required cascade_id ==
                                            // trajectory_id, so on a resumed session, or when a
                                            // subagent reported first, nothing was ever learned and
                                            // every trajectory then counted as the main one.
                                            if !traj_id.is_empty() {
                                                let mut main_id =
                                                    conn_cascade_id_for_ws.lock().await;
                                                let unset = main_id.is_none();
                                                if unset {
                                                    *main_id = Some(traj_id.clone());
                                                }
                                                drop(main_id);
                                                if unset {
                                                    tracing::debug!("main trajectory: {traj_id}");
                                                    let _ = conn_learned_id.set(traj_id.clone());
                                                }
                                            }

                                            // A model step on a trajectory that is not the main one
                                            // came from a subagent. Keep its text: the completion
                                            // event carries no result of its own (H12).
                                            {
                                                let main_id = conn_cascade_id_for_ws.lock().await.clone();
                                                let is_subagent = !traj_id.is_empty()
                                                    && main_id.as_ref().is_some_and(|id| *id != traj_id);
                                                if is_subagent
                                                    && step_update.source == Some(3)
                                                    && let Some(text) = step_update.text.clone().filter(|t| !t.is_empty())
                                                {
                                                    conn_subagent_responses.lock().await.insert(traj_id.clone(), text);
                                                }
                                            }

                                            let (is_questions_new, is_tool_conf_new) = {
                                                let mut trackers = conn_step_trackers.lock().await;
                                                let tracker = trackers.entry(key.clone()).or_insert_with(|| StepTracker {
                                                    state: 0,
                                                    handled_requests: HashSet::new(),
                                                });
                                                if let Some(st) = step_update.state {
                                                    tracker.update_state(st);
                                                }
                                                let is_q = if step_update.questions_request.is_some() {
                                                    tracker.mark_handled("questions_request")
                                                } else {
                                                    false
                                                };
                                                let is_tc = if step_update.tool_confirmation_request.is_some() {
                                                    tracker.mark_handled("tool_confirmation_request")
                                                } else {
                                                    false
                                                };
                                                drop(trackers);
                                                (is_q, is_tc)
                                            };

                                            // Map StepUpdate proto to Step domain model
                                            let step_type = if step_update.compaction.is_some() {
                                                StepType::Compaction
                                            } else if step_update.finish.is_some() {
                                                StepType::Finish
                                            } else if step_update.list_directory.is_some()
                                                || step_update.find_file.is_some()
                                                || step_update.search_directory.is_some()
                                                || step_update.view_file.is_some()
                                                || step_update.create_file.is_some()
                                                || step_update.edit_file.is_some()
                                                || step_update.run_command.is_some()
                                                || step_update.invoke_subagent.is_some()
                                                || step_update.generate_image.is_some()
                                            {
                                                StepType::ToolCall
                                            } else if step_update.text.is_some() {
                                                StepType::TextResponse
                                            } else {
                                                StepType::Unknown
                                            };

                                            let mut tool_calls = Vec::new();
                                            if let Some(tc) = crate::step_extract::extract_builtin_tool_call(&step_update) {
                                                tool_calls.push(tc);
                                            }

                                            let source = match step_update.source {
                                                Some(1) => StepSource::System,
                                                Some(2) => StepSource::User,
                                                Some(3) => StepSource::Model,
                                                _ => StepSource::Unknown,
                                            };

                                            let status = match step_update.state {
                                                Some(1) => StepStatus::Active,
                                                Some(2) => StepStatus::Done,
                                                Some(3) => StepStatus::WaitingForUser,
                                                // STATE_TERMINAL_ERROR = 5 was removed upstream in
                                                // 0.1.3; a step that fails now reports STATE_ERROR,
                                                // and a whole turn failing arrives as
                                                // TrajectoryStateUpdate.error instead (audit W7/W23).
                                                Some(4) => StepStatus::TerminalError,
                                                _ => StepStatus::Unknown,
                                            };

                                            let target = match step_update.target {
                                                Some(1) => StepTarget::User,
                                                Some(2 | 3) => StepTarget::Environment,
                                                _ => StepTarget::Unknown,
                                            };

                                            let usage = output_event.usage_metadata.map(|u| UsageMetadata {
                                                prompt_token_count: u.prompt_token_count.unwrap_or(0),
                                                candidates_token_count: u.candidates_token_count.unwrap_or(0),
                                                total_token_count: u.total_token_count.unwrap_or(0),
                                                cached_content_token_count: u.cached_content_token_count.unwrap_or(0),
                                                thoughts_token_count: u.thoughts_token_count.unwrap_or(0),
                                            });

                                            let is_complete = Some(
                                                source == StepSource::Model
                                                    && status == StepStatus::Done
                                                    && step_update.text.is_some()
                                                    && target == StepTarget::User
                                            );

                                            let structured = step_update.finish.as_ref().and_then(|f| {
                                                f.output_string.as_ref().and_then(|s| serde_json::from_str(s).ok())
                                            });

                                            // A step can carry ActionError{error_message,
                                            // http_code} with an empty top-level message, which
                                            // reported the failure as blank (audit C14).
                                            let error_msg = step_update
                                                .error_message
                                                .clone()
                                                .or_else(|| {
                                                    step_update
                                                        .error
                                                        .as_ref()
                                                        .and_then(|e| e.error_message.clone())
                                                })
                                                .unwrap_or_default();
                                            let http_code = step_update.error.as_ref().and_then(|e| e.http_code).unwrap_or(0);

                                            let step = Step {
                                                id: format!("{traj_id}_{step_idx}"),
                                                step_index: step_idx,
                                                r#type: step_type,
                                                source,
                                                target,
                                                status,
                                                content: step_update.text.clone().unwrap_or_default(),
                                                content_delta: step_update.text_delta.clone().unwrap_or_default(),
                                                thinking: step_update.thinking.clone().unwrap_or_default(),
                                                thinking_delta: step_update.thinking_delta.clone().unwrap_or_default(),
                                                tool_calls,
                                                error: error_msg,
                                                is_complete_response: is_complete,
                                                structured_output: structured,
                                                usage_metadata: usage,
                                                cascade_id: step_update.cascade_id.clone().unwrap_or_default(),
                                                trajectory_id: traj_id.clone(),
                                                http_code,
                                            };

                                            // Turn-level hooks fire off the step that carries the
                                            // event, which is the only place either is observable
                                            // from inside the connection (H1b, H1d).
                                            if let Some(runner) = hook_runner.as_ref() {
                                                if step.is_complete_response == Some(true) {
                                                    let runner = runner.clone();
                                                    let text = step.content.clone();
                                                    tokio::spawn(async move {
                                                        if let Err(e) = runner.dispatch_post_turn(&text).await {
                                                            tracing::error!("post_turn hook failed: {e:?}");
                                                        }
                                                    });
                                                }
                                                if step.r#type == StepType::Compaction {
                                                    let runner = runner.clone();
                                                    let compacted = step.clone();
                                                    tokio::spawn(async move {
                                                        if let Err(e) = runner.dispatch_on_compaction(&compacted).await {
                                                            tracing::error!("on_compaction hook failed: {e:?}");
                                                        }
                                                    });
                                                }
                                            }

                                            let _ = step_tx.send(crate::step_extract::StepEvent::Step(Box::new(step)));

                                            // Detect platform-level errors (source=SYSTEM) and propagate them.
                                            if source == StepSource::System
                                                && status == StepStatus::Error
                                                && (http_code == 400 || http_code == 401 || http_code == 403)
                                            {
                                                let err_str = step_update.error.as_ref().and_then(|e| e.error_message.clone()).unwrap_or_else(|| "System error occurred.".to_string());
                                                let _ = step_tx.send(crate::step_extract::StepEvent::Error(anyhow!("System step error (HTTP {}): {}", http_code, err_str)));
                                                break;
                                             }

                                            // Handle terminal errors — non-recoverable agent execution failure.
                                            if status == StepStatus::TerminalError {
                                                let err_msg = step_update.error_message.clone()
                                                    .unwrap_or_else(|| "Terminal error occurred during execution".to_string());
                                                let _ = step_tx.send(crate::step_extract::StepEvent::Error(
                                                    AntigravityExecutionError { message: err_msg }.into()
                                                ));
                                                break;
                                            }

                                            // Dispatch post-tool-call or on-tool-error hooks for built-in tools
                                            let state_val = step_update.state.unwrap_or(0);
                                            if state_val == 2 || state_val == 4 || state_val == 5 {
                                                let mut pending = pending_builtin_tool_calls.lock().await;
                                                if let (Some(tc), Some(runner)) = (pending.remove(&key), hook_runner.as_ref()) {
                                                    if state_val == 2 {
                                                        let extracted = extract_tool_result(&step_update);
                                                        let tr = ToolResult {
                                                            name: tc.name.clone(),
                                                            id: Some(tc.id.clone()),
                                                            result: extracted.and_then(|r| r.result).or_else(|| step_update.text.clone().map(Value::String)),
                                                            error: None,
                                                            server_name: None,
                                                            exception: None,
                                                        };
                                                        let runner_clone = runner.clone();
                                                        tokio::spawn(async move {
                                                            let _ = runner_clone.dispatch_post_tool_call(&tr).await;
                                                        });
                                                    } else {
                                                        let err_msg = step_update.error_message.clone().unwrap_or_else(|| "Built-in tool failed".to_string());
                                                        let err = anyhow!(err_msg);
                                                        let runner_clone = runner.clone();
                                                        tokio::spawn(async move {
                                                            runner_clone.dispatch_on_tool_error(&err).await;
                                                        });
                                                    }
                                                }
                                            }

                                            if let (true, Some(q_req)) = (is_questions_new, &step_update.questions_request) {
                                                let conn_ws_tx = conn_ws_tx.clone();
                                                let hook_runner = hook_runner.clone();
                                                let q_req_clone = q_req.clone();
                                                let trajectory_id = step_update.trajectory_id.clone();
                                                let step_index = step_update.step_index;
                                                tokio::spawn(async move {
                                                    let mut questions_list = Vec::new();
                                                    // The hook only sees multiple-choice questions,
                                                    // so the response index is an index into the
                                                    // FILTERED list. Carry the original index or
                                                    // every answer after a non-multiple-choice
                                                    // question is recorded against the wrong one.
                                                    let mut original_indices: Vec<usize> = Vec::new();
                                                    for (original_index, uq) in
                                                        q_req_clone.questions.iter().enumerate()
                                                    {
                                                        if let Some(crate::proto::localharness::user_question::QuestionType::MultipleChoice(ref mc)) = uq.question_type {
                                                            let mut opts = Vec::new();
                                                            for (j, choice) in mc.choices.iter().enumerate() {
                                                                opts.push(AskQuestionOption {
                                                                    id: (j + 1).to_string(),
                                                                    text: choice.clone(),
                                                                });
                                                            }
                                                            original_indices.push(original_index);
                                                            questions_list.push(AskQuestionEntry {
                                                                question: mc.question.clone().unwrap_or_default(),
                                                                options: opts,
                                                                is_multi_select: mc.is_multi_select.unwrap_or(false),
                                                            });
                                                        }
                                                    }

                                                    let mut proto_answers = vec![
                                                        UserQuestionAnswer {
                                                            answer: Some(crate::proto::localharness::user_question_answer::Answer::Unanswered(true)),
                                                        };
                                                        q_req_clone.questions.len()
                                                    ];

                                                    if let Some(runner) = hook_runner.as_ref().filter(|_| !questions_list.is_empty()) {
                                                        let res = runner.dispatch_interaction(&questions_list).await;
                                                        if let Ok(Some(q_res)) = res {
                                                            for (filtered_idx, r) in
                                                                q_res.responses.iter().enumerate()
                                                            {
                                                                // A hook may return more responses
                                                                // than there were questions; ignore
                                                                // the extras rather than panicking.
                                                                let Some(&orig_idx) =
                                                                    original_indices.get(filtered_idx)
                                                                else {
                                                                    break;
                                                                };
                                                                if !r.skipped {
                                                                    let mut mc_ans = MultipleChoiceAnswer {
                                                                        selected_choice_indices: Vec::new(),
                                                                        freeform_response: Some(r.freeform_response.clone()),
                                                                    };
                                                                    if let Some(ref opts) = r.selected_option_ids {
                                                                        for opt in opts {
                                                                            if let Ok(idx) = opt.parse::<i32>() {
                                                                                mc_ans.selected_choice_indices.push(idx - 1);
                                                                            }
                                                                        }
                                                                    }
                                                                    proto_answers[orig_idx] = UserQuestionAnswer {
                                                                        answer: Some(crate::proto::localharness::user_question_answer::Answer::MultipleChoiceAnswer(mc_ans)),
                                                                    };
                                                                }
                                                            }
                                                        }
                                                    }

                                                    let resp = UserQuestionsResponse {
                                                        trajectory_id,
                                                        step_index,
                                                        result: Some(crate::proto::localharness::user_questions_response::Result::Response(QuestionsResponse {
                                                            answers: proto_answers,
                                                        })),
                                                    };
                                                    let input_event = InputEvent {
                                                        event: Some(crate::proto::localharness::input_event::Event::QuestionResponse(resp)),
                                                    };
                                                    if let Ok(raw_json) = serde_json::to_string(&input_event) {
                                                        let _ = conn_ws_tx.send(raw_json);
                                                    }
                                                });
                                            }

                                            if is_tool_conf_new {
                                                let conn_ws_tx = conn_ws_tx.clone();
                                                let hook_runner = hook_runner.clone();
                                                let step_update_clone = step_update.clone();
                                                let pending_calls = pending_builtin_tool_calls.clone();
                                                tokio::spawn(async move {
                                                    let mut allow = true;
                                                    let tool_call = crate::step_extract::extract_builtin_tool_call(&step_update_clone);
                                                    if let Some(ref tc) = tool_call {
                                                        // Fails closed: a hook that errors denies.
                                                        (allow, _) = crate::hooks::HookRunner::gate_tool_call(hook_runner.as_ref(), tc).await;
                                                        if allow {
                                                            let key = (step_update_clone.trajectory_id.clone().unwrap_or_default(), step_update_clone.step_index.unwrap_or(0));
                                                            pending_calls.lock().await.insert(key, tc.clone());
                                                        }
                                                    }

                                                    let conf = ToolConfirmation {
                                                        trajectory_id: step_update_clone.trajectory_id.clone(),
                                                        step_index: step_update_clone.step_index,
                                                        accepted: Some(allow),
                                                    };
                                                    let input_event = InputEvent {
                                                        event: Some(crate::proto::localharness::input_event::Event::ToolConfirmation(conf)),
                                                    };
                                                    if let Ok(raw_json) = serde_json::to_string(&input_event) {
                                                        let _ = conn_ws_tx.send(raw_json);
                                                    }
                                                });
                                            }
                                        }
                                        crate::proto::localharness::output_event::Event::TrajectoryStateUpdate(tsu) => {
                                            let traj_id = tsu.trajectory_id.clone().unwrap_or_default();
                                            let main_id = conn_cascade_id_for_ws.lock().await;
                                            // Only the main trajectory drives idle. Upstream returns
                                            // early for subagent trajectories (event_processor.py:539-542);
                                            // the previous parent_idle + active_subagent_ids
                                            // bookkeeping is the 0.1.1 shape, deleted upstream in 0.1.6.
                                            let is_main = main_id
                                                .as_ref()
                                                .is_none_or(|id| traj_id.is_empty() || traj_id == *id);
                                            tracing::debug!("TrajectoryStateUpdate: trajectory_id={traj_id:?}, state={:?}, is_main={is_main}", tsu.state);
                                            drop(main_id);

                                            if !is_main {
                                                // A subagent finishing is how a START_SUBAGENT call
                                                // completes — the harness sends no tool response for
                                                // it. Without this a `post_tool_call` hook saw the
                                                // pre_tool_call and never a matching completion.
                                                if tsu.state == Some(2) || tsu.state == Some(3) {
                                                    let response = conn_subagent_responses
                                                        .lock()
                                                        .await
                                                        .remove(&traj_id)
                                                        .unwrap_or_else(|| traj_id.clone());
                                                    if let Some(runner) = hook_runner.as_ref() {
                                                        let tr = crate::types::ToolResult {
                                                            name: crate::types::BuiltinTools::StartSubagent
                                                                .as_str()
                                                                .to_string(),
                                                            id: None,
                                                            result: Some(Value::String(response)),
                                                            error: None,
                                                            server_name: None,
                                                            exception: None,
                                                        };
                                                        let runner = runner.clone();
                                                        tokio::spawn(async move {
                                                            let _ = runner.dispatch_post_tool_call(&tr).await;
                                                        });
                                                    }
                                                }
                                                continue;
                                            }

                                            // A turn that failed server-side reports its
                                            // reason here; without this the stream just ends
                                            // (event_processor.py:554-557).
                                            if let Some(ref err) = tsu.error
                                                && !err.is_empty()
                                            {
                                                let _ = step_tx.send(
                                                    crate::step_extract::StepEvent::Error(anyhow!(
                                                        "{err}"
                                                    )),
                                                );
                                            }

                                            if tsu.state == Some(3) { // STATE_CANCELLED
                                                conn_cancel_requested.store(false, Ordering::SeqCst);
                                                let reason = tsu
                                                    .error
                                                    .clone()
                                                    .filter(|e| !e.is_empty())
                                                    .unwrap_or_else(|| "Turn cancelled".to_string());
                                                let _ = step_tx.send(
                                                    crate::step_extract::StepEvent::Error(
                                                        anyhow!(crate::error::AntigravityError::Cancelled(reason)),
                                                    ),
                                                );
                                            } else if tsu.state == Some(2) // STATE_FULLY_IDLE
                                                && conn_cancel_requested.swap(false, Ordering::SeqCst)
                                            {
                                                // A halt the caller asked for. The harness stops
                                                // the turn and reports ordinary idle, so this is
                                                // the only point at which the two can be told
                                                // apart (A3, docs/remaining-work.md).
                                                let _ = step_tx.send(
                                                    crate::step_extract::StepEvent::Error(
                                                        anyhow!(crate::error::AntigravityError::Cancelled(
                                                            "Cancelled by caller".to_string()
                                                        )),
                                                    ),
                                                );
                                            }

                                            if tsu.state == Some(2) || tsu.state == Some(3) { // STATE_FULLY_IDLE | STATE_CANCELLED
                                                conn_is_idle.store(true, Ordering::SeqCst);
                                                let _ = conn_idle_tx.send(true);
                                                tracing::debug!("Connection transitioned to IDLE, sending sentinel");
                                                let _ = step_tx.send(crate::step_extract::StepEvent::Idle);
                                            }
                                        }
                                        crate::proto::localharness::output_event::Event::InitializeConversationResponse(resp) => {
                                            // The harness's first frame since 0.1.4. Reading it
                                            // during the handshake — and seeding the conversation
                                            // with `resp.history` on a resumed session — is WP-6;
                                            // until then a resumed session silently starts empty.
                                            tracing::debug!(
                                                "initialize_conversation_response ({} history steps) — not yet consumed, see WP-6",
                                                resp.history.len()
                                            );
                                        }
                                        crate::proto::localharness::output_event::Event::CallHookRequest(req) => {
                                            // Harness-side lifecycle hooks (WP-8). The harness only
                                            // sends these for hooks named in HarnessConfig.enabled_hooks,
                                            // which this crate does not populate, so reaching here means
                                            // the two have gone out of sync. The harness blocks its turn
                                            // waiting for a CallHookResponse we cannot yet send.
                                            tracing::warn!(
                                                "unexpected call_hook_request (id={:?}, type={:?}); no hook router — the harness may stall. See WP-8",
                                                req.request_id,
                                                req.r#type
                                            );
                                        }
                                        crate::proto::localharness::output_event::Event::SessionEndResponse(_) => {
                                            // Answer to a session_end_request we do not send yet (WP-6).
                                            tracing::debug!("session_end_response");
                                        }
                                        crate::proto::localharness::output_event::Event::ToolCall(tool_call) => {
                                            let conn_ws_tx = conn_ws_tx.clone();
                                            let tool_runner = tool_runner.clone();
                                            let hook_runner = hook_runner.clone();
                                            let step_tx_clone = step_tx.clone();
                                            let learned_id_clone = conn_learned_id.clone();
                                            let counter = client_tool_step_counter.clone();
                                            tokio::spawn(async move {
                                                let args: Value = crate::tool_wire::parse_arguments(tool_call.arguments_json.as_deref());
                                                let tc = ToolCall {
                                                    id: tool_call.id.clone().unwrap_or_default(),
                                                    name: tool_call.name.clone().unwrap_or_default(),
                                                    args: args.clone(),
                                                    canonical_path: None,
                                                    server_name: None,
                                                };
                                                tracing::debug!("ToolCall event received: id={}, name={}", tc.id, tc.name);

                                                // Emit ACTIVE step so the UI can show a tool card
                                                let synth_idx = counter.fetch_add(1, Ordering::SeqCst);
                                                let traj_id = learned_id_clone.get()
                                                    .cloned()
                                                    .unwrap_or_default();
                                                let active_step = Step {
                                                    id: tc.id.clone(),
                                                    step_index: synth_idx,
                                                    r#type: StepType::ToolCall,
                                                    source: StepSource::Model,
                                                    target: StepTarget::Environment,
                                                    status: StepStatus::Active,
                                                    content: tc.name.clone(),
                                                    tool_calls: vec![tc.clone()],
                                                    trajectory_id: traj_id.clone(),
                                                    ..Default::default()
                                                };
                                                let _ = step_tx_clone.send(crate::step_extract::StepEvent::Step(Box::new(active_step)));

                                                // Fails closed: a hook that errors denies.
                                                let (allow, deny_reason) = crate::hooks::HookRunner::gate_tool_call(hook_runner.as_ref(), &tc).await;
                                                tracing::debug!("Policy decision for tool {}: allow={}", tc.name, allow);

                                                if !allow {
                                                    // Emit ERROR step for denied tool call
                                                    let denied_step = Step {
                                                        id: tc.id.clone(),
                                                        step_index: synth_idx,
                                                        r#type: StepType::ToolCall,
                                                        source: StepSource::Model,
                                                        target: StepTarget::Environment,
                                                        status: StepStatus::Error,
                                                        content: tc.name.clone(),
                                                        error: if deny_reason.is_empty() {
                                                            "Execution denied by hook policy".to_string()
                                                        } else {
                                                            deny_reason.clone()
                                                        },
                                                        tool_calls: vec![tc.clone()],
                                                        trajectory_id: traj_id,
                                                        ..Default::default()
                                                    };
                                                    let _ = step_tx_clone.send(crate::step_extract::StepEvent::Step(Box::new(denied_step)));

                                                    let resp = crate::tool_wire::denied_response(
                                                        tool_call.id.clone(),
                                                        if deny_reason.is_empty() {
                                                            "Execution denied by hook policy"
                                                        } else {
                                                            &deny_reason
                                                        },
                                                    );
                                                    let input_event = InputEvent {
                                                        event: Some(crate::proto::localharness::input_event::Event::ToolResponse(resp)),
                                                    };
                                                    if let Ok(raw_json) = serde_json::to_string(&input_event) {
                                                        let _ = conn_ws_tx.send(raw_json);
                                                    }
                                                    return;
                                                }

                                                let mut result = ToolResult {
                                                    id: tool_call.id.clone(),
                                                    name: tc.name.clone(),
                                                    result: None,
                                                    error: None,
                                                    server_name: None,
                                                    exception: None,
                                                };

                                                if let Some(ref runner) = tool_runner {
                                                    tracing::debug!("Executing tool {} with args: {:?}", tc.name, tc.args);
                                                    let results = runner.process_tool_calls(vec![tc.clone()]).await;
                                                    if let Some(r) = results.into_iter().next() {
                                                        result = r;
                                                    }
                                                } else {
                                                    result.error = Some("No tool runner registered".to_string());
                                                }

                                                if let (Some(err_str), Some(runner)) = (result.error.as_ref(), hook_runner.as_ref()) {
                                                    // The hook may reword the failure. It may not
                                                    // turn it into a success: clearing the error
                                                    // reported a tool that had failed to the model
                                                    // as having worked (H4).
                                                    if let Some(message) = runner.dispatch_on_tool_error(&anyhow!(err_str.clone())).await {
                                                        result.error = Some(message);
                                                    }
                                                } else if let Some(runner) = hook_runner.as_ref() {
                                                    let _ = runner.dispatch_post_tool_call(&result).await;
                                                }

                                                // Emit DONE or ERROR step with the execution result
                                                let (final_status, error_msg, result_args) = if result.error.is_some() {
                                                    (StepStatus::Error, result.error.clone().unwrap_or_default(), Value::Null)
                                                } else {
                                                    (StepStatus::Done, String::new(), result.result.clone().unwrap_or(Value::Null))
                                                };
                                                let done_step = Step {
                                                    id: tc.id.clone(),
                                                    step_index: synth_idx,
                                                    r#type: StepType::ToolCall,
                                                    source: StepSource::Model,
                                                    target: StepTarget::Environment,
                                                    status: final_status,
                                                    content: tc.name.clone(),
                                                    error: error_msg,
                                                    tool_calls: vec![ToolCall {
                                                        id: tc.id.clone(),
                                                        name: tc.name.clone(),
                                                        args: result_args,
                                                        canonical_path: None,
                                                        server_name: None,
                                                    }],
                                                    trajectory_id: traj_id,
                                                    ..Default::default()
                                                };
                                                let _ = step_tx_clone.send(crate::step_extract::StepEvent::Step(Box::new(done_step)));

                                                let resp = crate::tool_wire::tool_response(tool_call.id.clone(), &result);
                                                let input_event = InputEvent {
                                                    event: Some(crate::proto::localharness::input_event::Event::ToolResponse(resp)),
                                                };
                                                if let Ok(raw_json) = serde_json::to_string(&input_event) {
                                                    tracing::debug!("Sending ToolResponse input_event: {}", raw_json);
                                                    let _ = conn_ws_tx.send(raw_json);
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to deserialize OutputEvent: {:?}. Raw: {}",
                                    e,
                                    &raw_text[..raw_text.len().min(300)]
                                );
                            }
                        }
                    }
                    Ok(_) => {}
                    Err(e) => {
                        let _ = step_tx.send(crate::step_extract::StepEvent::Error(anyhow!(
                            "WS read error: {e:?}"
                        )));
                        break;
                    }
                }
            }

            // The socket is gone. If the turn had not reached idle, the harness
            // died mid-turn: say so, and quote what it printed on its way out.
            // Without this the step stream just ends and the caller sees a turn
            // that produced nothing, with no indication anything went wrong.
            if !conn_is_idle_for_close.load(Ordering::SeqCst) {
                let tail = {
                    let tail = stderr_tail.lock().await;
                    tail.iter().cloned().collect::<Vec<_>>().join("\n")
                };
                let detail = if tail.is_empty() {
                    "harness exited without writing to stderr".to_string()
                } else {
                    format!("last harness stderr:\n{tail}")
                };
                let _ = step_tx.send(crate::step_extract::StepEvent::Error(anyhow!(
                    "harness connection closed before the turn finished; {detail}"
                )));
                conn_is_idle_for_close.store(true, Ordering::SeqCst);
                let _ = conn_idle_tx.send(true);
                let _ = step_tx.send(crate::step_extract::StepEvent::Idle);
            }
        });

        // 10. Hook runners dispatch session start
        if let Some(ref runner) = self.hook_runner {
            runner.dispatch_session_start().await?;
        }

        Ok(LocalConnection {
            conversation_id,
            learned_id,
            process: Arc::new(Mutex::new(child)),
            child_stdin: Arc::new(Mutex::new(Some(child_stdin))),
            is_idle,
            step_rx: Arc::new(Mutex::new(Some(step_rx))),
            ws_tx,
            tool_runner: self.tool_runner.clone(),
            hook_runner: self.hook_runner.clone(),
            step_trackers,
            main_trajectory_id: conn_cascade_id,
            cancel_requested,
            steps_consumed: Arc::new(AtomicBool::new(false)),
            idle_tx,
            subagent_responses,
            initial_history,
        })
    }
}

/// Internal state tracker for matching `StepUpdate` payloads with active handshakes.
#[derive(Debug)]
pub struct StepTracker {
    state: i32,
    handled_requests: HashSet<String>,
}

impl StepTracker {
    /// Updates the tracked step status state.
    pub fn update_state(&mut self, state: i32) {
        // Leaving WAITING_FOR_USER ends the request round. Without this the
        // dedup set persists, so a re-asked question is never answered a second
        // time and the harness waits forever. STATE_WAITING_FOR_USER = 3.
        if self.state == 3 && state != 3 {
            self.handled_requests.clear();
        }
        self.state = state;
    }

    /// Marks a specific request payload (e.g. "`tool_confirmation_request`") as handled.
    /// Returns true if the tracker transitioned to active for the request.
    pub fn mark_handled(&mut self, request_name: &str) -> bool {
        if self.state == 3 && !self.handled_requests.contains(request_name) {
            self.handled_requests.insert(request_name.to_string());
            return true;
        }
        false
    }
}

fn extract_tool_result(step_update: &crate::proto::localharness::StepUpdate) -> Option<ToolResult> {
    let traj_id = step_update.trajectory_id.clone().unwrap_or_default();
    let step_idx = step_update.step_index.unwrap_or(0);
    let id = format!("{traj_id}_{step_idx}");

    let tool_call = crate::step_extract::extract_builtin_tool_call(step_update)?;
    let result = step_update.text.clone().map(Value::String);
    let error = step_update.error_message.clone();

    Some(ToolResult {
        id: Some(id),
        name: tool_call.name,
        result,
        error,
        server_name: None,
        exception: None,
    })
}

/// Returns the Rust compiler version used to build this crate.
fn rustc_version() -> String {
    option_env!("RUSTC_VERSION")
        .or(option_env!("CARGO_PKG_RUST_VERSION"))
        .unwrap_or("unknown")
        .to_string()
}
