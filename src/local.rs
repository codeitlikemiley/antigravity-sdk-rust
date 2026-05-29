//! Local subprocess connection and negotiation.
//!
//! This module provides [`LocalConnectionStrategy`] to initialize and spawn the local
//! agent subprocess, perform the initial handshake, and transition to a WebSocket session
//! wrapped by [`LocalConnection`].

use crate::connection::Connection;
use crate::hooks::HookRunner;
use crate::proto::localharness::{
    ClientInfo as ProtoClientInfo, FileEditToolConfig, FilesystemWorkspace, FindToolConfig,
    GeminiConfig as ProtoGeminiConfig, GenerateImageToolConfig, GrepSearchToolConfig,
    HarnessConfig, HarnessSideTools, InitializeConversationEvent, InputConfig, InputEvent,
    ListDirToolConfig, MultipleChoiceAnswer, OutputConfig, OutputEvent, RunCommandToolConfig,
    SubagentsConfig, SystemInstructions as ProtoSystemInstructions, Tool as ProtoTool,
    ToolConfirmation, ToolResponse, UserQuestionAnswer, UserQuestionsConfig, UserQuestionsResponse,
    ViewFileToolConfig, Workspace as ProtoWorkspace, WriteToFileToolConfig,
    appended_system_instructions::Section, custom_system_instructions::Part,
    user_questions_response::QuestionsResponse, workspace::WorkspaceType,
};
use crate::tools::ToolRunner;
use crate::types::{
    AntigravityExecutionError, AskQuestionEntry, AskQuestionOption, BuiltinTools,
    CapabilitiesConfig, GeminiConfig, QuestionHookResult, Step, StepSource, StepStatus, StepTarget,
    StepType, SystemInstructions, ToolCall, ToolResult, UsageMetadata,
};

use anyhow::anyhow;
use futures_util::stream::{self, BoxStream};
use futures_util::{SinkExt, StreamExt};
use prost::Message;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::process::Stdio;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt};
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::sync::mpsc::{self, UnboundedSender};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;

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
    step_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<Result<Step, anyhow::Error>>>>>,
    ws_tx: UnboundedSender<String>,
    tool_runner: Option<ToolRunner>,
    hook_runner: Option<HookRunner>,
    parent_idle: Arc<Mutex<bool>>,
    active_subagent_ids: Arc<Mutex<HashSet<String>>>,
    step_trackers: Arc<Mutex<HashMap<(String, u32), StepTracker>>>,
}

impl std::fmt::Debug for LocalConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalConnection")
            .field("conversation_id", &self.conversation_id())
            .field("is_idle", &self.is_idle)
            .field("tool_runner", &self.tool_runner)
            .field("hook_runner", &self.hook_runner)
            .field("parent_idle", &self.parent_idle)
            .field("active_subagent_ids", &self.active_subagent_ids)
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

    fn receive_steps(&self) -> BoxStream<'static, Result<Step, anyhow::Error>> {
        let step_rx = self.step_rx.clone();
        let is_idle = self.is_idle.clone();
        stream::unfold(false, move |mut checked_initial_idle| {
            let step_rx = step_rx.clone();
            let is_idle = is_idle.clone();
            async move {
                // If the connection is already idle on the first poll and the queue is empty, terminate.
                if !checked_initial_idle {
                    checked_initial_idle = true;
                    let mut guard = step_rx.lock().await;
                    if guard
                        .as_mut()
                        .is_some_and(|rx| rx.is_empty() && is_idle.load(Ordering::SeqCst))
                    {
                        return None;
                    }
                }

                loop {
                    let mut guard = step_rx.lock().await;
                    let Some(rx) = &mut *guard else {
                        return None;
                    };
                    match rx.try_recv() {
                        Ok(step_res) => match &step_res {
                            Ok(step) if step.id == "IDLE_SENTINEL" => {
                                if is_idle.load(Ordering::SeqCst) {
                                    return None;
                                }
                            }
                            _ => {
                                return Some((step_res, checked_initial_idle));
                            }
                        },
                        Err(mpsc::error::TryRecvError::Empty) => {
                            drop(guard);
                            let mut guard2 = step_rx.lock().await;
                            let Some(rx2) = &mut *guard2 else {
                                return None;
                            };
                            let step_res = rx2.recv().await;
                            drop(guard2);
                            match step_res {
                                Some(res) => match &res {
                                    Ok(step) if step.id == "IDLE_SENTINEL" => {
                                        if is_idle.load(Ordering::SeqCst) {
                                            return None;
                                        }
                                    }
                                    _ => {
                                        return Some((res, checked_initial_idle));
                                    }
                                },
                                None => return None,
                            }
                        }
                        Err(mpsc::error::TryRecvError::Disconnected) => {
                            return None;
                        }
                    }
                }
            }
        })
        .boxed()
    }

    async fn send(&self, content: &str) -> Result<(), anyhow::Error> {
        self.is_idle.store(false, Ordering::SeqCst);
        {
            let mut p_idle = self.parent_idle.lock().await;
            *p_idle = false;
        }
        {
            let mut active = self.active_subagent_ids.lock().await;
            active.clear();
        }
        {
            let mut guard = self.step_rx.lock().await;
            if let Some(rx) = &mut *guard {
                while rx.try_recv().is_ok() {}
            }
        }

        let input_event = InputEvent {
            event: Some(crate::proto::localharness::input_event::Event::UserInput(
                content.to_string(),
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
        let resp_json = if let Some(ref val) = result.result {
            // The Go harness expects responseJson to always be a JSON object.
            // If the tool returned a non-object value (string, number, array, etc.),
            // wrap it under a "result" key to match the Python SDK's behavior.
            if val.is_object() {
                serde_json::to_string(val)?
            } else {
                serde_json::to_string(&serde_json::json!({ "result": val }))?
            }
        } else if let Some(ref err) = result.error {
            serde_json::to_string(&serde_json::json!({ "error": err }))?
        } else {
            "{}".to_string()
        };

        let resp = ToolResponse {
            id: Some(id.to_string()),
            response_json: Some(resp_json),
            supplemental_media: Vec::new(),
            response: None,
        };
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
        let mut proc = self.process.lock().await;
        let _ = proc.kill().await;
        drop(proc);
        Ok(())
    }
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
}

impl LocalConnectionStrategy {
    /// Creates a new `LocalConnectionStrategy`.
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
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

        let api_key = api_key.unwrap_or_default();

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
            language: Some("rust".to_string()),
            version: Some(env!("CARGO_PKG_VERSION").to_string()),
            language_version: Some(rustc_version()),
        };

        let input_config = InputConfig {
            storage_directory: self.save_dir.clone(),
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
        let ws_url = format!("ws://localhost:{port}/");
        let mut req = ws_url.clone().into_client_request()?;
        req.headers_mut()
            .insert("x-goog-api-key", harness_api_key.parse()?);

        // Connect with retry/backoff
        let mut ws_stream = None;
        let mut delay = std::time::Duration::from_millis(100);
        for attempt in 0..5 {
            match connect_async(req.clone()).await {
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
        if let Some(ref runner) = self.tool_runner {
            let tools = runner.tools.read().await;
            for t in tools.values() {
                proto_tools.push(ProtoTool {
                    name: Some(t.name().to_string()),
                    description: Some(t.description().to_string()),
                    parameters_json_schema: Some(t.parameters_json_schema().to_string()),
                    response_json_schema: None,
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

        let proto_gemini = ProtoGeminiConfig {
            api_key: Some(api_key),
            base_url: None,
            model_name: Some(self.gemini_config.models.default.name.clone()),
            thinking_level: self
                .gemini_config
                .models
                .default
                .generation
                .thinking_level
                .map(|l| match l {
                    crate::types::ThinkingLevel::Minimal => "minimal".to_string(),
                    crate::types::ThinkingLevel::Low => "low".to_string(),
                    crate::types::ThinkingLevel::Medium => "medium".to_string(),
                    crate::types::ThinkingLevel::High => "high".to_string(),
                }),
            enable_url_context: self.gemini_config.enable_url_context,
            enable_google_search: self.gemini_config.enable_google_search,
            use_vertex: Some(self.gemini_config.vertex),
            project: self.gemini_config.project.clone(),
            location: self.gemini_config.location.clone(),
        };

        let mut proto_workspaces = Vec::new();
        for w in &self.workspaces {
            proto_workspaces.push(ProtoWorkspace {
                workspace_type: Some(WorkspaceType::FilesystemWorkspace(FilesystemWorkspace {
                    directory: Some(w.clone()),
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
                enabled: Some(true),
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
                model_name: self.capabilities_config.image_model.clone(),
            }),
        };

        let harness_config = HarnessConfig {
            cascade_id: Some(self.conversation_id.clone()),
            model_config: Some(
                crate::proto::localharness::harness_config::ModelConfig::GeminiConfig(proto_gemini),
            ),
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
        let (step_tx, step_rx) = mpsc::unbounded_channel::<Result<Step, anyhow::Error>>();
        let client_tool_step_counter = Arc::new(AtomicU32::new(50_000));

        let is_idle = Arc::new(AtomicBool::new(false));
        let parent_idle = Arc::new(Mutex::new(false));
        let active_subagent_ids = Arc::new(Mutex::new(HashSet::new()));
        let step_trackers = Arc::new(Mutex::new(HashMap::new()));

        let conn_ws_tx = ws_tx.clone();
        let conn_is_idle = is_idle.clone();
        let conn_parent_idle = parent_idle.clone();
        let conn_active_subagents = active_subagent_ids.clone();
        let conn_step_trackers = step_trackers.clone();

        let tool_runner = self.tool_runner.clone();
        let hook_runner = self.hook_runner.clone();
        let conversation_id = self.conversation_id.clone();
        let learned_id = Arc::new(std::sync::OnceLock::new());
        let conn_learned_id = learned_id.clone();
        let conn_cascade_id: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let conn_cascade_id_for_ws = conn_cascade_id.clone();

        // 8. Spawn Stderr Reader
        let mut reader = tokio::io::BufReader::new(child_stderr);
        tokio::spawn(async move {
            let mut line = String::new();
            while let Ok(n) = reader.read_line(&mut line).await {
                if n == 0 {
                    break;
                }
                tracing::info!("Harness stderr: {}", line.trim_end());
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

                                            // Learn the cascade_id from the first StepUpdate
                                            // where cascade_id == trajectory_id (Python parity)
                                            {
                                                let cascade_id_val = step_update.cascade_id.clone().unwrap_or_default();
                                                if !cascade_id_val.is_empty() && cascade_id_val == traj_id {
                                                    let _ = conn_learned_id.set(cascade_id_val.clone());
                                                    let mut cid = conn_cascade_id_for_ws.lock().await;
                                                    if cid.is_none() {
                                                        tracing::debug!("Learned cascade_id from StepUpdate: {}", cascade_id_val);
                                                        *cid = Some(cascade_id_val);
                                                    }
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
                                            if let Some(tc) = extract_builtin_tool_call(&step_update) {
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
                                                Some(4) => StepStatus::Error,
                                                Some(5) => StepStatus::TerminalError,
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

                                            let error_msg = step_update.error_message.clone().unwrap_or_default();
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

                                            let _ = step_tx.send(Ok(step));

                                            // Detect platform-level errors (source=SYSTEM) and propagate them.
                                            if source == StepSource::System
                                                && status == StepStatus::Error
                                                && (http_code == 400 || http_code == 401 || http_code == 403)
                                            {
                                                let err_str = step_update.error.as_ref().and_then(|e| e.error_message.clone()).unwrap_or_else(|| "System error occurred.".to_string());
                                                let _ = step_tx.send(Err(anyhow!("System step error (HTTP {}): {}", http_code, err_str)));
                                                break;
                                             }

                                            // Handle terminal errors — non-recoverable agent execution failure.
                                            if status == StepStatus::TerminalError {
                                                let err_msg = step_update.error_message.clone()
                                                    .unwrap_or_else(|| "Terminal error occurred during execution".to_string());
                                                let _ = step_tx.send(Err(
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
                                                            let _ = runner_clone.dispatch_on_tool_error(&err).await;
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
                                                    for uq in &q_req_clone.questions {
                                                        if let Some(crate::proto::localharness::user_question::QuestionType::MultipleChoice(ref mc)) = uq.question_type {
                                                            let mut opts = Vec::new();
                                                            for (j, choice) in mc.choices.iter().enumerate() {
                                                                opts.push(AskQuestionOption {
                                                                    id: (j + 1).to_string(),
                                                                    text: choice.clone(),
                                                                });
                                                            }
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
                                                            for (orig_idx, r) in q_res.responses.iter().enumerate() {
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
                                                    let tool_call = extract_builtin_tool_call(&step_update_clone);
                                                    if let Some(ref tc) = tool_call {
                                                        if let Some(ref runner) = hook_runner {
                                                            let pre_call = runner.dispatch_pre_tool_call(tc).await;
                                                            if let Ok(res) = pre_call {
                                                                allow = res.allow;
                                                            }
                                                        }
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
                                            let sub_id = tsu.trajectory_id.clone().unwrap_or_default();
                                            let learned_cascade = conn_cascade_id_for_ws.lock().await;
                                            let is_subagent = learned_cascade.as_ref().is_some_and(|cid| !sub_id.is_empty() && sub_id != *cid);
                                            tracing::debug!("TrajectoryStateUpdate: trajectory_id={:?}, state={:?}, is_subagent={}, learned_cascade_id={:?}", sub_id, tsu.state, is_subagent, *learned_cascade);
                                            drop(learned_cascade);

                                            let mut active_subs = conn_active_subagents.lock().await;
                                            let mut p_idle = conn_parent_idle.lock().await;

                                            if tsu.state == Some(1) { // STATE_RUNNING
                                                if is_subagent {
                                                    active_subs.insert(sub_id);
                                                }
                                            } else if tsu.state == Some(2) { // STATE_IDLE
                                                if is_subagent {
                                                    active_subs.remove(&sub_id);
                                                } else {
                                                    *p_idle = true;
                                                }
                                            }

                                            tracing::debug!("TrajectoryStateUpdate: p_idle={}, active_subs_empty={}", *p_idle, active_subs.is_empty());
                                            if *p_idle && active_subs.is_empty() && !conn_is_idle.swap(true, Ordering::SeqCst) {
                                                tracing::debug!("Connection transitioned to IDLE, sending sentinel");
                                                let sentinel = Step {
                                                    id: "IDLE_SENTINEL".to_string(),
                                                    ..Default::default()
                                                };
                                                let _ = step_tx.send(Ok(sentinel));
                                            }
                                        }
                                        crate::proto::localharness::output_event::Event::ToolCall(tool_call) => {
                                            let conn_ws_tx = conn_ws_tx.clone();
                                            let tool_runner = tool_runner.clone();
                                            let hook_runner = hook_runner.clone();
                                            let step_tx_clone = step_tx.clone();
                                            let learned_id_clone = conn_learned_id.clone();
                                            let counter = client_tool_step_counter.clone();
                                            tokio::spawn(async move {
                                                let args: Value = serde_json::from_str(&tool_call.arguments_json.clone().unwrap_or_default()).unwrap_or(Value::Null);
                                                let tc = ToolCall {
                                                    id: tool_call.id.clone().unwrap_or_default(),
                                                    name: tool_call.name.clone().unwrap_or_default(),
                                                    args: args.clone(),
                                                    canonical_path: None,
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
                                                let _ = step_tx_clone.send(Ok(active_step));

                                                let allow = if let Some(runner) = hook_runner.as_ref() {
                                                    let res = runner.dispatch_pre_tool_call(&tc).await.map_or(true, |res| res.allow);
                                                    tracing::debug!("Policy decision for tool {}: allow={}", tc.name, res);
                                                    res
                                                } else {
                                                    true
                                                };

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
                                                        error: "Execution denied by hook policy".to_string(),
                                                        tool_calls: vec![tc.clone()],
                                                        trajectory_id: traj_id,
                                                        ..Default::default()
                                                    };
                                                    let _ = step_tx_clone.send(Ok(denied_step));

                                                    let resp = ToolResponse {
                                                        id: tool_call.id.clone(),
                                                        response_json: Some("{\"error\": \"Execution denied by hook policy\"}".to_string()),
                                                        supplemental_media: Vec::new(),
                                                        response: None,
                                                    };
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
                                                    if let Ok((res, val)) = runner.dispatch_on_tool_error(&anyhow!(err_str.clone())).await {
                                                        let allow_error = res.allow;
                                                        if allow_error {
                                                            result.result = val;
                                                            result.error = None;
                                                        }
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
                                                    }],
                                                    trajectory_id: traj_id,
                                                    ..Default::default()
                                                };
                                                let _ = step_tx_clone.send(Ok(done_step));

                                                // The Go harness expects responseJson to always be a JSON object.
                                                // Wrap non-object values (string, number, array, etc.) under "result".
                                                let resp_json = if let Some(ref val) = result.result {
                                                    if val.is_object() {
                                                        serde_json::to_string(val).unwrap_or_default()
                                                    } else {
                                                        serde_json::to_string(&serde_json::json!({ "result": val })).unwrap_or_default()
                                                    }
                                                } else if let Some(ref err) = result.error {
                                                    serde_json::to_string(&serde_json::json!({ "error": err })).unwrap_or_default()
                                                } else {
                                                    "{}".to_string()
                                                };

                                                let resp = ToolResponse {
                                                    id: tool_call.id.clone(),
                                                    response_json: Some(resp_json),
                                                    supplemental_media: Vec::new(),
                                                    response: None,
                                                };
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
                        let _ = step_tx.send(Err(anyhow!("WS read error: {e:?}")));
                        break;
                    }
                }
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
            parent_idle,
            active_subagent_ids,
            step_trackers,
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
    pub const fn update_state(&mut self, state: i32) {
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

#[allow(clippy::too_many_lines)]
fn extract_builtin_tool_call(
    step_update: &crate::proto::localharness::StepUpdate,
) -> Option<ToolCall> {
    let traj_id = step_update.trajectory_id.clone().unwrap_or_default();
    let step_idx = step_update.step_index.unwrap_or(0);
    let id = format!("{traj_id}_{step_idx}");

    if step_update.invoke_subagent.is_some() {
        return Some(ToolCall {
            id,
            name: "START_SUBAGENT".to_string(),
            args: serde_json::json!({
                "prompt": step_update.request_text.clone().unwrap_or_default()
            }),
            canonical_path: None,
        });
    }

    if let Some(ref fd) = step_update.find_file {
        return Some(ToolCall {
            id,
            name: "FIND_FILE".to_string(),
            args: serde_json::json!({
                "directory_path": fd.directory_path,
                "query": fd.query,
            }),
            canonical_path: fd.directory_path.clone(),
        });
    }
    if let Some(ref run) = step_update.run_command {
        return Some(ToolCall {
            id,
            name: "RUN_COMMAND".to_string(),
            args: serde_json::json!({
                "command_line":    run.command_line,
                "working_dir":     run.working_dir,
                // Include the execution result fields so the frontend
                // can display stdout/stderr instead of "(no output)".
                "combined_output": run.combined_output,
                "exit_code":       run.exit_code,
            }),
            canonical_path: None,
        });
    }
    if let Some(ref view) = step_update.view_file {
        return Some(ToolCall {
            id,
            name: "VIEW_FILE".to_string(),
            args: serde_json::json!({
                "file_path": view.file_path,
                "start_line": view.start_line,
                "end_line": view.end_line,
            }),
            canonical_path: view.file_path.clone(),
        });
    }
    if let Some(ref write) = step_update.create_file {
        return Some(ToolCall {
            id,
            name: "CREATE_FILE".to_string(),
            args: serde_json::json!({
                "file_path": write.file_path,
                "contents": write.contents,
            }),
            canonical_path: write.file_path.clone(),
        });
    }
    if let Some(ref edit) = step_update.edit_file {
        return Some(ToolCall {
            id,
            name: "EDIT_FILE".to_string(),
            args: serde_json::json!({
                "file_path": edit.file_path,
            }),
            canonical_path: edit.file_path.clone(),
        });
    }
    if let Some(ref search) = step_update.search_directory {
        // The harness puts grep/search results into `step_update.text`.
        // Pack them into `args.output` so the frontend can display them,
        // mirroring how RUN_COMMAND packs `combined_output`.
        return Some(ToolCall {
            id,
            name: "SEARCH_DIR".to_string(),
            args: serde_json::json!({
                "directory_path": search.directory_path,
                "query": search.query,
                "num_results": search.num_results,
                // Actual grep results from the harness
                "output": step_update.text,
            }),
            canonical_path: search.directory_path.clone(),
        });
    }
    if let Some(ref list) = step_update.list_directory {
        return Some(ToolCall {
            id,
            name: "LIST_DIR".to_string(),
            args: serde_json::json!({
                "directory_path": list.directory_path,
            }),
            canonical_path: list.directory_path.clone(),
        });
    }
    if let Some(ref img_gen) = step_update.generate_image {
        return Some(ToolCall {
            id,
            name: "GENERATE_IMAGE".to_string(),
            args: serde_json::json!({
                "prompt": img_gen.prompt,
                "image_paths": img_gen.image_paths,
                "image_name": img_gen.image_name,
            }),
            canonical_path: None,
        });
    }
    None
}

fn extract_tool_result(step_update: &crate::proto::localharness::StepUpdate) -> Option<ToolResult> {
    let traj_id = step_update.trajectory_id.clone().unwrap_or_default();
    let step_idx = step_update.step_index.unwrap_or(0);
    let id = format!("{traj_id}_{step_idx}");

    let tool_call = extract_builtin_tool_call(step_update)?;
    let result = step_update.text.clone().map(Value::String);
    let error = step_update.error_message.clone();

    Some(ToolResult {
        id: Some(id),
        name: tool_call.name,
        result,
        error,
    })
}

/// Returns the Rust compiler version used to build this crate.
fn rustc_version() -> String {
    option_env!("RUSTC_VERSION")
        .or(option_env!("CARGO_PKG_RUST_VERSION"))
        .unwrap_or("unknown")
        .to_string()
}
