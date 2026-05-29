//! WebAssembly (wasm32-wasip1) Connection Implementation.
//!
//! This module provides a WebSocket-based harness connection for WebAssembly environments,
//! connecting to the host `localharness` process over the network.

use anyhow::{Result, anyhow};
use futures_util::stream::{self, BoxStream, StreamExt};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use tokio::sync::{Mutex, mpsc};
use tracing;

#[cfg(not(target_arch = "wasm32"))]
use tokio_tungstenite::tungstenite;
#[cfg(target_arch = "wasm32")]
use tungstenite;

use tungstenite::{Message as WsMessage, client::client, handshake::client::Request};

use crate::connection::Connection;
use crate::hooks::HookRunner;
use crate::proto::localharness::{
    FileEditToolConfig, FilesystemWorkspace, FindToolConfig, GeminiConfig as ProtoGeminiConfig,
    GenerateImageToolConfig, GrepSearchToolConfig, HarnessConfig, HarnessSideTools,
    InitializeConversationEvent, InputEvent, ListDirToolConfig, MultipleChoiceAnswer, OutputEvent,
    RunCommandToolConfig, StepUpdate, SubagentsConfig,
    SystemInstructions as ProtoSystemInstructions, Tool as ProtoTool, ToolConfirmation,
    ToolResponse, UserQuestionAnswer, UserQuestionsConfig, UserQuestionsResponse,
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

/// Internal state tracker for matching `StepUpdate` payloads with active handshakes.
#[derive(Debug, Default)]
pub struct StepTracker {
    state: i32,
    handled_requests: HashSet<String>,
}

impl StepTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub const fn update_state(&mut self, state: i32) {
        self.state = state;
    }

    pub fn mark_handled(&mut self, request_name: &str) -> bool {
        if self.state == 3 && !self.handled_requests.contains(request_name) {
            self.handled_requests.insert(request_name.to_string());
            return true;
        }
        false
    }
}

#[cfg(test)]
pub static TEST_PORT: std::sync::atomic::AtomicU16 = std::sync::atomic::AtomicU16::new(0);

/// A WASM connection strategy that establishes a connection to a remote localharness
/// instance running on the host machine.
#[derive(Debug)]
pub struct WasmConnectionStrategy {
    pub gemini_config: GeminiConfig,
    pub capabilities_config: CapabilitiesConfig,
    pub system_instructions: Option<SystemInstructions>,
    pub save_dir: Option<String>,
    pub workspaces: Vec<String>,
    pub skills_paths: Vec<String>,
    pub tool_runner: Option<ToolRunner>,
    pub hook_runner: Option<HookRunner>,
    pub conversation_id: String,
}

impl WasmConnectionStrategy {
    #[allow(
        clippy::too_many_lines,
        clippy::significant_drop_tightening,
        clippy::option_if_let_else
    )]
    pub async fn connect(&self) -> Result<WasmConnection, anyhow::Error> {
        let api_key = self
            .gemini_config
            .models
            .default
            .api_key
            .clone()
            .or_else(|| self.gemini_config.api_key.clone())
            .or_else(|| std::env::var("ANTIGRAVITY_API_KEY").ok())
            .or_else(|| std::env::var("GEMINI_API_KEY").ok())
            .ok_or_else(|| {
                anyhow!(
                    "A Gemini API key is required. Set it via GeminiConfig or ANTIGRAVITY_API_KEY env var."
                )
            })?;

        let host =
            std::env::var("ANTIGRAVITY_HARNESS_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        #[allow(unused_mut)]
        let mut port =
            std::env::var("ANTIGRAVITY_HARNESS_PORT").unwrap_or_else(|_| "8000".to_string());
        #[cfg(test)]
        {
            let test_port = TEST_PORT.load(std::sync::atomic::Ordering::SeqCst);
            if test_port != 0 {
                port = test_port.to_string();
            }
        }

        let ws_url = format!("ws://{host}:{port}/");
        tracing::info!("Connecting to localharness WebSocket at {}", ws_url);

        let req = Request::builder()
            .uri(&ws_url)
            .header("Host", format!("{host}:{port}"))
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                tungstenite::handshake::client::generate_key(),
            )
            .header("x-goog-api-key", &api_key)
            .body(())?;

        let tcp = TcpStream::connect(format!("{host}:{port}")).map_err(|e| {
            anyhow!("Failed to connect to localharness TCP socket at {host}:{port}: {e:?}")
        })?;

        let (mut ws, _) = client(req, tcp)
            .map_err(|e| anyhow!("WebSocket handshake with localharness failed: {e:?}"))?;

        // Configure TCP stream to be non-blocking for concurrent read/write via Arc/Mutex yielding.
        ws.get_ref().set_nonblocking(true)?;

        // Build HarnessConfig proto
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

        let all_tools = [
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

        // Send InitializeConversationEvent
        let init_event = InitializeConversationEvent {
            config: Some(harness_config),
        };
        let init_json = serde_json::to_string(&init_event)?;
        ws.write(WsMessage::Text(init_json))?;
        ws.flush()?;

        let socket = Arc::new(Mutex::new(ws));

        // Spawn Background WS Sender Loop
        let (ws_tx, mut ws_rx) = mpsc::unbounded_channel::<String>();
        let socket_sender = socket.clone();
        crate::spawn_task(async move {
            while let Some(msg) = ws_rx.recv().await {
                let mut delay = std::time::Duration::from_millis(5);
                loop {
                    if let Ok(mut ws_lock) = socket_sender.try_lock() {
                        if let Err(e) = ws_lock.write(WsMessage::Text(msg)) {
                            tracing::error!("WS Write Loop Error: {:?}", e);
                        } else {
                            let _ = ws_lock.flush();
                        }
                        break;
                    }
                    tokio::time::sleep(delay).await;
                    if delay < std::time::Duration::from_millis(50) {
                        delay *= 2;
                    }
                }
            }
        });

        // Setup channels for step stream
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

        // Spawn WS Reader Loop
        let pending_builtin_tool_calls =
            Arc::new(Mutex::new(HashMap::<(String, u32), ToolCall>::new()));
        let socket_reader = socket.clone();

        crate::spawn_task(async move {
            loop {
                let read_res = {
                    let mut ws_lock = socket_reader.lock().await;
                    ws_lock.read()
                };
                match read_res {
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
                                                let tracker = trackers.entry(key.clone()).or_insert_with(StepTracker::new);
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
                                                (is_q, is_tc)
                                            };

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

                                            let usage = output_event.usage_metadata.as_ref().map(|u| UsageMetadata {
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
                                            if state_val == 2 || state_val == 4 {
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
                                                        crate::spawn_task(async move {
                                                            let _ = runner_clone.dispatch_post_tool_call(&tr).await;
                                                        });
                                                    } else {
                                                        let err_msg = step_update.error_message.clone().unwrap_or_else(|| "Built-in tool failed".to_string());
                                                        let err = anyhow!(err_msg);
                                                        let runner_clone = runner.clone();
                                                        crate::spawn_task(async move {
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
                                                crate::spawn_task(async move {
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
                                                crate::spawn_task(async move {
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
                                            crate::spawn_task(async move {
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

                                                // Wrap non-object values under "result"
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
                    Err(tungstenite::Error::Io(ref e))
                        if e.kind() == std::io::ErrorKind::WouldBlock =>
                    {
                        // Release lock & yield to let other tasks run (specifically the WS sender)
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    }
                    Err(e) => {
                        let _ = step_tx.send(Err(anyhow!("WS read error: {e:?}")));
                        break;
                    }
                }
            }
        });

        // Hook runners dispatch session start
        if let Some(ref runner) = self.hook_runner {
            runner.dispatch_session_start().await?;
        }

        Ok(WasmConnection {
            conversation_id,
            learned_id,
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

/// Stateful WebSocket harness connection for WebAssembly.
#[allow(dead_code)]
#[derive(Debug)]
pub struct WasmConnection {
    conversation_id: String,
    learned_id: Arc<std::sync::OnceLock<String>>,
    is_idle: Arc<AtomicBool>,
    step_rx: Arc<Mutex<Option<mpsc::UnboundedReceiver<Result<Step, anyhow::Error>>>>>,
    ws_tx: mpsc::UnboundedSender<String>,
    tool_runner: Option<ToolRunner>,
    hook_runner: Option<HookRunner>,
    parent_idle: Arc<Mutex<bool>>,
    active_subagent_ids: Arc<Mutex<HashSet<String>>>,
    step_trackers: Arc<Mutex<HashMap<(String, u32), StepTracker>>>,
}

impl Connection for WasmConnection {
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
        // No explicit subprocess to kill in WASM connection.
        Ok(())
    }
}

#[allow(clippy::too_many_lines)]
fn extract_builtin_tool_call(step_update: &StepUpdate) -> Option<ToolCall> {
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
                "command_line": run.command_line,
                "working_dir": run.working_dir,
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

fn extract_tool_result(step_update: &StepUpdate) -> Option<ToolResult> {
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

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::field_reassign_with_default)]
mod tests {
    use super::*;
    use crate::proto::localharness::{
        ActionCreateFile, ActionEditFile, ActionFindFile, ActionGenerateImage, ActionListDirectory,
        ActionRunCommand, ActionViewFile, StepUpdate,
    };
    use crate::types::{CapabilitiesConfig, GeminiConfig};
    use futures_util::{SinkExt, StreamExt};
    use tokio::net::TcpListener;
    use tokio_tungstenite::accept_async;
    use tokio_tungstenite::tungstenite::Message as WsMessage;

    #[test]
    fn test_step_tracker_new() {
        let mut tracker = StepTracker::new();
        assert!(!tracker.mark_handled("questions_request"));
    }

    #[test]
    fn test_step_tracker_update_state() {
        let mut tracker = StepTracker::new();
        tracker.update_state(3);
        assert!(tracker.mark_handled("questions_request"));
        assert!(!tracker.mark_handled("questions_request"));
        assert!(tracker.mark_handled("tool_confirmation_request"));

        tracker.update_state(1);
        assert!(!tracker.mark_handled("another_request"));
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_extract_builtin_tool_call_all_types() {
        // 1. FindFile
        let step_update_find = StepUpdate {
            trajectory_id: Some("traj_1".to_string()),
            step_index: Some(1),
            find_file: Some(ActionFindFile {
                directory_path: Some("dir_path".to_string()),
                query: Some("query_str".to_string()),
                output: None,
            }),
            ..Default::default()
        };
        let tc = extract_builtin_tool_call(&step_update_find).unwrap();
        assert_eq!(tc.id, "traj_1_1");
        assert_eq!(tc.name, "FIND_FILE");
        assert_eq!(tc.canonical_path, Some("dir_path".to_string()));
        assert_eq!(
            tc.args,
            serde_json::json!({
                "directory_path": "dir_path",
                "query": "query_str"
            })
        );

        // Test extract_tool_result for FindFile
        let step_update_find_res = StepUpdate {
            trajectory_id: Some("traj_1".to_string()),
            step_index: Some(1),
            find_file: Some(ActionFindFile {
                directory_path: Some("dir_path".to_string()),
                query: Some("query_str".to_string()),
                output: None,
            }),
            text: Some("result_text".to_string()),
            error_message: Some("error_text".to_string()),
            ..Default::default()
        };
        let tr = extract_tool_result(&step_update_find_res).unwrap();
        assert_eq!(tr.id, Some("traj_1_1".to_string()));
        assert_eq!(tr.name, "FIND_FILE");
        assert_eq!(
            tr.result,
            Some(serde_json::Value::String("result_text".to_string()))
        );
        assert_eq!(tr.error, Some("error_text".to_string()));

        // 2. RunCommand
        let step_update_run = StepUpdate {
            trajectory_id: Some("traj_1".to_string()),
            step_index: Some(2),
            run_command: Some(ActionRunCommand {
                command_line: Some("echo hello".to_string()),
                working_dir: Some("work_dir".to_string()),
                exit_code: None,
                combined_output: None,
            }),
            ..Default::default()
        };
        let tc = extract_builtin_tool_call(&step_update_run).unwrap();
        assert_eq!(tc.id, "traj_1_2");
        assert_eq!(tc.name, "RUN_COMMAND");
        assert_eq!(tc.canonical_path, None);
        assert_eq!(
            tc.args,
            serde_json::json!({
                "command_line": "echo hello",
                "working_dir": "work_dir"
            })
        );

        // 3. ViewFile
        let step_update_view = StepUpdate {
            trajectory_id: Some("traj_1".to_string()),
            step_index: Some(3),
            view_file: Some(ActionViewFile {
                file_path: Some("view_path".to_string()),
                start_line: Some(10),
                end_line: Some(20),
            }),
            ..Default::default()
        };
        let tc = extract_builtin_tool_call(&step_update_view).unwrap();
        assert_eq!(tc.id, "traj_1_3");
        assert_eq!(tc.name, "VIEW_FILE");
        assert_eq!(tc.canonical_path, Some("view_path".to_string()));
        assert_eq!(
            tc.args,
            serde_json::json!({
                "file_path": "view_path",
                "start_line": 10,
                "end_line": 20
            })
        );

        // 4. CreateFile
        let step_update_create = StepUpdate {
            trajectory_id: Some("traj_1".to_string()),
            step_index: Some(4),
            create_file: Some(ActionCreateFile {
                file_path: Some("create_path".to_string()),
                contents: Some("create_contents".to_string()),
            }),
            ..Default::default()
        };
        let tc = extract_builtin_tool_call(&step_update_create).unwrap();
        assert_eq!(tc.id, "traj_1_4");
        assert_eq!(tc.name, "CREATE_FILE");
        assert_eq!(tc.canonical_path, Some("create_path".to_string()));
        assert_eq!(
            tc.args,
            serde_json::json!({
                "file_path": "create_path",
                "contents": "create_contents"
            })
        );

        // 5. EditFile
        let step_update_edit = StepUpdate {
            trajectory_id: Some("traj_1".to_string()),
            step_index: Some(5),
            edit_file: Some(ActionEditFile {
                file_path: Some("edit_path".to_string()),
                diff_block: vec![],
            }),
            ..Default::default()
        };
        let tc = extract_builtin_tool_call(&step_update_edit).unwrap();
        assert_eq!(tc.id, "traj_1_5");
        assert_eq!(tc.name, "EDIT_FILE");
        assert_eq!(tc.canonical_path, Some("edit_path".to_string()));
        assert_eq!(
            tc.args,
            serde_json::json!({
                "file_path": "edit_path"
            })
        );

        // 5.5 InvokeSubagent
        let step_update_sub = StepUpdate {
            trajectory_id: Some("traj_1".to_string()),
            step_index: Some(6),
            invoke_subagent: Some(crate::proto::localharness::ActionInvokeSubagent {}),
            request_text: Some("Do a subtask".to_string()),
            ..Default::default()
        };
        let tc = extract_builtin_tool_call(&step_update_sub).unwrap();
        assert_eq!(tc.id, "traj_1_6");
        assert_eq!(tc.name, "START_SUBAGENT");
        assert_eq!(tc.canonical_path, None);
        assert_eq!(
            tc.args,
            serde_json::json!({
                "prompt": "Do a subtask"
            })
        );

        // 5.6 ListDirectory
        let step_update_list = StepUpdate {
            trajectory_id: Some("traj_1".to_string()),
            step_index: Some(8),
            list_directory: Some(ActionListDirectory {
                directory_path: Some("list_path".to_string()),
                results: vec![],
            }),
            ..Default::default()
        };
        let tc = extract_builtin_tool_call(&step_update_list).unwrap();
        assert_eq!(tc.id, "traj_1_8");
        assert_eq!(tc.name, "LIST_DIR");
        assert_eq!(tc.canonical_path, Some("list_path".to_string()));
        assert_eq!(
            tc.args,
            serde_json::json!({
                "directory_path": "list_path"
            })
        );

        // 5.7 GenerateImage
        let step_update_gen = StepUpdate {
            trajectory_id: Some("traj_1".to_string()),
            step_index: Some(9),
            generate_image: Some(ActionGenerateImage {
                prompt: Some("a gold dragon logo".to_string()),
                image_paths: vec!["/tmp/dragon.png".to_string()],
                image_name: Some("dragon_logo".to_string()),
            }),
            ..Default::default()
        };
        let tc = extract_builtin_tool_call(&step_update_gen).unwrap();
        assert_eq!(tc.id, "traj_1_9");
        assert_eq!(tc.name, "GENERATE_IMAGE");
        assert_eq!(tc.canonical_path, None);
        assert_eq!(tc.args["prompt"], "a gold dragon logo");
        assert_eq!(tc.args["image_name"], "dragon_logo");
        assert_eq!(tc.args["image_paths"][0], "/tmp/dragon.png");

        // 6. None
        let step_update_none = StepUpdate {
            trajectory_id: Some("traj_1".to_string()),
            step_index: Some(7),
            ..Default::default()
        };
        assert!(extract_builtin_tool_call(&step_update_none).is_none());
        assert!(extract_tool_result(&step_update_none).is_none());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_wasm_connection_integration_mock() {
        // Bind to a free local port
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        // Spawn the mock WebSocket server in the background
        let server_handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut ws_stream = accept_async(stream).await.unwrap();

            // 1. Receive the InitializeConversationEvent configuration message
            let msg = ws_stream.next().await.unwrap().unwrap();
            let text = msg.to_text().unwrap();
            assert!(text.contains("InitializeConversationEvent") || text.contains("cascadeId"));

            // 2. Send trajectoryStateUpdate (RUNNING)
            let traj_running = serde_json::json!({
                "trajectoryStateUpdate": {
                    "trajectoryId": "test_traj",
                    "state": "STATE_RUNNING"
                }
            });
            ws_stream
                .send(WsMessage::Text(traj_running.to_string()))
                .await
                .unwrap();

            // 3. Send StepUpdate
            let step_update = serde_json::json!({
                "stepUpdate": {
                    "stepIndex": 1,
                    "cascadeId": "test_traj",
                    "trajectoryId": "test_traj",
                    "text": "Hello from mock harness!",
                    "textDelta": "Hello from mock harness!",
                    "state": "STATE_ACTIVE",
                    "source": "SOURCE_MODEL",
                    "target": "TARGET_USER"
                }
            });
            ws_stream
                .send(WsMessage::Text(step_update.to_string()))
                .await
                .unwrap();

            // 4. Send trajectoryStateUpdate (IDLE)
            let traj_idle = serde_json::json!({
                "trajectoryStateUpdate": {
                    "trajectoryId": "test_traj",
                    "state": "STATE_IDLE"
                }
            });
            ws_stream
                .send(WsMessage::Text(traj_idle.to_string()))
                .await
                .unwrap();

            // 5. Wait for the client to send "hello"
            let msg2 = ws_stream.next().await.unwrap().unwrap();
            let text2 = msg2.to_text().unwrap();
            assert!(text2.contains("hello"));

            // Keep connection open long enough
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        });

        // Configure host/port via static atomic variable (safe, no unsafe_code)
        TEST_PORT.store(port, std::sync::atomic::Ordering::SeqCst);

        // Instantiate strategy
        let mut gemini_config = GeminiConfig::default();
        gemini_config.api_key = Some("mock_key".to_string());
        let strategy = WasmConnectionStrategy {
            gemini_config,
            capabilities_config: CapabilitiesConfig::default(),
            system_instructions: None,
            save_dir: None,
            workspaces: vec![],
            skills_paths: vec![],
            tool_runner: None,
            hook_runner: None,
            conversation_id: "test_traj".to_string(),
        };

        // Connect
        let conn = strategy.connect().await.unwrap();
        assert_eq!(conn.conversation_id(), "test_traj");

        // Consume the step stream
        let mut steps = conn.receive_steps();
        let step = steps.next().await.unwrap().unwrap();
        assert_eq!(step.content, "Hello from mock harness!");
        assert_eq!(step.step_index, 1);

        // Stream should end (returns None) once transitioned to IDLE
        let next_step = steps.next().await;
        assert!(next_step.is_none());

        // Send a message
        conn.send("hello").await.unwrap();

        // Join the server task
        server_handle.await.unwrap();
    }
}
