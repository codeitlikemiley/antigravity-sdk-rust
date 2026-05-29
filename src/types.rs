//! Common configuration structures, enums, and SDK data models.
//!
//! This module houses all the data types shared across the SDK, including Gemini configuration
//! parameters, system instructions, capability filters, built-in tools list, and step execution progress structs.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// The default model name used when none is specified.
pub const DEFAULT_MODEL: &str = "gemini-3.5-flash";

/// The default image generation model name used.
pub const DEFAULT_IMAGE_GENERATION_MODEL: &str = "gemini-3.1-flash-image-preview";

/// Configures the intensity of the reasoning/thinking process for models that support it.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingLevel {
    /// Minimal reasoning overhead.
    Minimal,
    /// Low reasoning.
    Low,
    /// Medium reasoning.
    Medium,
    /// High reasoning.
    High,
}

/// Generation configuration parameters.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerationConfig {
    /// Desired thinking level for reasoning-based models.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<ThinkingLevel>,
}

/// Specific model entry defining the model name, key, and generation settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    /// The name/identifier of the model.
    pub name: String,
    /// Model-specific API key (if overriding the global key).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// Generation settings (e.g. thinking configurations).
    #[serde(default)]
    pub generation: GenerationConfig,
}

impl Default for ModelEntry {
    fn default() -> Self {
        Self {
            name: DEFAULT_MODEL.to_string(),
            api_key: None,
            generation: GenerationConfig::default(),
        }
    }
}

/// Mapping of models configured for different tasks in the agent's session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// The primary text/chat model.
    #[serde(default = "default_model_entry")]
    pub default: ModelEntry,
    /// The model used for image generation tasks.
    #[serde(default = "default_image_generation_entry")]
    pub image_generation: ModelEntry,
}

fn default_model_entry() -> ModelEntry {
    ModelEntry {
        name: DEFAULT_MODEL.to_string(),
        api_key: None,
        generation: GenerationConfig::default(),
    }
}

fn default_image_generation_entry() -> ModelEntry {
    ModelEntry {
        name: DEFAULT_IMAGE_GENERATION_MODEL.to_string(),
        api_key: None,
        generation: GenerationConfig::default(),
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            default: default_model_entry(),
            image_generation: default_image_generation_entry(),
        }
    }
}

/// Root configurations for the Gemini AI model endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GeminiConfig {
    /// Global API key for Gemini endpoints.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,
    /// If true, uses the Vertex AI backend instead of Gemini Developer API.
    #[serde(default)]
    pub vertex: bool,
    /// GCP Project ID for Vertex AI.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// GCP Location/Region for Vertex AI (e.g., "us-central1").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Model configurations.
    #[serde(default)]
    pub models: ModelConfig,
    /// Option to enable Google Search grounding tool.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_google_search: Option<bool>,
    /// Option to enable URL context resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_url_context: Option<bool>,
}

/// A structured section appended to system instructions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInstructionSection {
    /// Main markdown or text body of the section.
    pub content: String,
    /// Described title of the section.
    pub title: String,
}

/// Directly supplied system instruction instructions text override.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSystemInstructions {
    /// Custom raw instructions text.
    pub text: String,
}

/// Appended instructions format, maintaining identity overrides and section segments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppendedSystemInstructions {
    /// Optional override for the agent's custom identity block.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_identity: Option<String>,
    /// Sections to be appended to the standard system instructions.
    #[serde(default)]
    pub appended_sections: Vec<SystemInstructionSection>,
}

/// Represents the style or content source for system instructions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SystemInstructions {
    /// Completely custom text override.
    Custom(CustomSystemInstructions),
    /// Standard structured segments appended to the system identity.
    Appended(AppendedSystemInstructions),
}

/// Enumeration of built-in tools supported by the agent system.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum BuiltinTools {
    /// Tool to create a new file.
    #[serde(rename = "CREATE_FILE")]
    CreateFile,
    /// Tool to edit an existing file.
    #[serde(rename = "EDIT_FILE")]
    EditFile,
    /// Tool to query/find files in a directory.
    #[serde(rename = "FIND_FILE")]
    FindFile,
    /// Tool to list files inside a directory.
    #[serde(rename = "LIST_DIR")]
    ListDir,
    /// Tool to execute a shell command.
    #[serde(rename = "RUN_COMMAND")]
    RunCommand,
    /// Tool to perform ripgrep searches.
    #[serde(rename = "SEARCH_DIR")]
    SearchDir,
    /// Tool to view a file's content.
    #[serde(rename = "VIEW_FILE")]
    ViewFile,
    /// Tool to instantiate a subagent.
    #[serde(rename = "START_SUBAGENT")]
    StartSubagent,
    /// Tool to generate images from descriptions.
    #[serde(rename = "GENERATE_IMAGE")]
    GenerateImage,
    /// Terminating signal indicating the task is completed.
    #[serde(rename = "FINISH")]
    Finish,
}

impl BuiltinTools {
    /// Returns the static string slice mapping to the tool name.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::CreateFile => "CREATE_FILE",
            Self::EditFile => "EDIT_FILE",
            Self::FindFile => "FIND_FILE",
            Self::ListDir => "LIST_DIR",
            Self::RunCommand => "RUN_COMMAND",
            Self::SearchDir => "SEARCH_DIR",
            Self::ViewFile => "VIEW_FILE",
            Self::StartSubagent => "START_SUBAGENT",
            Self::GenerateImage => "GENERATE_IMAGE",
            Self::Finish => "FINISH",
        }
    }

    /// Returns a list of all safe, read-only tools.
    pub fn read_only() -> Vec<Self> {
        vec![
            Self::FindFile,
            Self::ListDir,
            Self::ViewFile,
            Self::SearchDir,
        ]
    }
}

/// Agent capabilities and tool restrictions configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CapabilitiesConfig {
    /// List of explicitly enabled tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_tools: Option<Vec<BuiltinTools>>,
    /// List of explicitly disabled tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_tools: Option<Vec<BuiltinTools>>,
    /// Threshold at which the message history is compacted/summarized.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compaction_threshold: Option<u32>,
    /// Custom schema override for the finish tool schema.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_tool_schema_json: Option<String>,
    /// Model designated for processing images.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_model: Option<String>,
}

/// Configuration settings for Model Context Protocol (MCP) servers.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum McpServerConfig {
    /// Launch the MCP server as a local stdio process.
    #[serde(rename = "stdio")]
    Stdio {
        /// Unique identifier for this MCP server.
        name: String,
        /// command binary.
        command: String,
        /// execution arguments.
        args: Vec<String>,
        /// Explicit allowlist of tools to enable. Mutually exclusive with `disabled_tools`.
        #[serde(skip_serializing_if = "Option::is_none")]
        enabled_tools: Option<Vec<String>>,
        /// Explicit denylist of tools to disable. Mutually exclusive with `enabled_tools`.
        #[serde(skip_serializing_if = "Option::is_none")]
        disabled_tools: Option<Vec<String>>,
    },
    /// Connect to the MCP server via Server-Sent Events (SSE).
    #[serde(rename = "sse")]
    Sse {
        /// Unique identifier for this MCP server.
        name: String,
        /// HTTP URL endpoint.
        url: String,
        /// Additional HTTP headers.
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
        /// Explicit allowlist of tools to enable.
        #[serde(skip_serializing_if = "Option::is_none")]
        enabled_tools: Option<Vec<String>>,
        /// Explicit denylist of tools to disable.
        #[serde(skip_serializing_if = "Option::is_none")]
        disabled_tools: Option<Vec<String>>,
    },
    /// Connect to the MCP server via standard HTTP.
    #[serde(rename = "http")]
    Http {
        /// Unique identifier for this MCP server.
        name: String,
        /// HTTP URL endpoint.
        url: String,
        /// Additional HTTP headers.
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<HashMap<String, String>>,
        /// General connection timeout in seconds.
        #[serde(default = "default_mcp_timeout")]
        timeout: f64,
        /// Reading timeout for the SSE listener.
        #[serde(default = "default_mcp_sse_timeout")]
        sse_read_timeout: f64,
        /// Flag whether to terminate the channel connection when closed.
        #[serde(default = "default_true")]
        terminate_on_close: bool,
        /// Explicit allowlist of tools to enable.
        #[serde(skip_serializing_if = "Option::is_none")]
        enabled_tools: Option<Vec<String>>,
        /// Explicit denylist of tools to disable.
        #[serde(skip_serializing_if = "Option::is_none")]
        disabled_tools: Option<Vec<String>>,
    },
}

impl McpServerConfig {
    /// Returns the unique name identifier of this MCP server.
    pub fn name(&self) -> &str {
        match self {
            Self::Stdio { name, .. } | Self::Sse { name, .. } | Self::Http { name, .. } => name,
        }
    }
}

/// Error raised when the agent execution encounters a terminal (non-recoverable) error.
///
/// This indicates that the agent loop has terminated due to a fatal error
/// (e.g. model call failure, system constraint violation) and cannot continue.
#[derive(Debug, Clone)]
pub struct AntigravityExecutionError {
    /// The error message describing the terminal failure.
    pub message: String,
}

impl std::fmt::Display for AntigravityExecutionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Terminal execution error: {}", self.message)
    }
}

impl std::error::Error for AntigravityExecutionError {}

const fn default_mcp_timeout() -> f64 {
    30.0
}
const fn default_mcp_sse_timeout() -> f64 {
    300.0
}
const fn default_true() -> bool {
    true
}

/// Describes a model's request to execute a registered tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Unique call ID generated for correlation.
    pub id: String,
    /// Name of the target tool.
    pub name: String,
    /// Arguments payload parsed as JSON.
    pub args: Value,
    /// Canonical file system path (if the tool targets a file/directory).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub canonical_path: Option<String>,
}

/// The response outcome of executing a client-side tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Name of the executed tool.
    pub name: String,
    /// Optional matching tool call ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Output result of successful tool execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Error message string if tool execution failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Consumption stats for API usage tracking.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UsageMetadata {
    /// Tokens included in the request prompt.
    pub prompt_token_count: i32,
    /// Tokens generated in candidates.
    pub candidates_token_count: i32,
    /// Total combined tokens.
    pub total_token_count: i32,
    /// Cache hit content tokens.
    #[serde(default)]
    pub cached_content_token_count: i32,
    /// Tokens consumed during inner thinking/reasoning.
    #[serde(default)]
    pub thoughts_token_count: i32,
}

/// The classification type of a step in the trajectory.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepType {
    /// Raw text content returned by the model.
    #[serde(rename = "TEXT_RESPONSE")]
    TextResponse,
    /// Execution of a tool call.
    #[serde(rename = "TOOL_CALL")]
    ToolCall,
    /// Logging or notification events from the system.
    #[serde(rename = "SYSTEM_MESSAGE")]
    SystemMessage,
    /// A history compaction step summarizing context.
    #[serde(rename = "COMPACTION")]
    Compaction,
    /// Terminating milestone indicator.
    #[serde(rename = "FINISH")]
    Finish,
    /// Catch-all variant for unrecognized steps.
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

/// The originating source component of a trajectory step.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepSource {
    /// Internal orchestration environment.
    #[serde(rename = "SYSTEM")]
    System,
    /// End-user input.
    #[serde(rename = "USER")]
    User,
    /// Generative model prediction.
    #[serde(rename = "MODEL")]
    Model,
    /// Unknown author.
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

/// The target destination of a step event.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepTarget {
    /// Event addressed to the user.
    #[serde(rename = "TARGET_USER")]
    User,
    /// Event executing in the sandbox environment.
    #[serde(rename = "TARGET_ENVIRONMENT")]
    Environment,
    /// Unspecified destination.
    #[serde(rename = "TARGET_UNSPECIFIED")]
    Unspecified,
    /// Unknown destination.
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

/// Lifecycle execution status of a trajectory step.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepStatus {
    /// Active/running state.
    #[serde(rename = "ACTIVE")]
    Active,
    /// Completed successfully.
    #[serde(rename = "DONE")]
    Done,
    /// Waiting for user response/confirmation.
    #[serde(rename = "WAITING_FOR_USER")]
    WaitingForUser,
    /// Finished with a fatal error.
    #[serde(rename = "ERROR")]
    Error,
    /// Execution was canceled.
    #[serde(rename = "CANCELED")]
    Canceled,
    /// A fatal, non-recoverable error occurred during execution.
    #[serde(rename = "TERMINAL_ERROR")]
    TerminalError,
    /// Unknown status.
    #[serde(rename = "UNKNOWN")]
    Unknown,
}

/// Individual step event recording an action in the agent's history trajectory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Unique identifier for this step.
    pub id: String,
    /// Positional index in the trajectory sequence.
    pub step_index: u32,
    /// Functional type of the step.
    pub r#type: StepType,
    /// Originating author.
    pub source: StepSource,
    /// Destination target.
    pub target: StepTarget,
    /// Execution status.
    pub status: StepStatus,
    /// Main text/markdown content associated with the step.
    pub content: String,
    /// Text difference delta relative to previous steps.
    pub content_delta: String,
    /// Reasoning thoughts generated for this step.
    pub thinking: String,
    /// Thinking reasoning difference delta relative to previous steps.
    pub thinking_delta: String,
    /// Custom tool executions registered in this step.
    pub tool_calls: Vec<ToolCall>,
    /// Captured execution errors.
    pub error: String,
    /// True if this represents the final response segment from the model.
    pub is_complete_response: Option<bool>,
    /// Parsed structured JSON output.
    pub structured_output: Option<Value>,
    /// Token usage details.
    pub usage_metadata: Option<UsageMetadata>,
    /// Unique identifier of the execution cascade grouping subagents.
    #[serde(default)]
    pub cascade_id: String,
    /// Sub-agent trajectory grouping identifier.
    #[serde(default)]
    pub trajectory_id: String,
    /// HTTP status code (if from a network action).
    #[serde(default)]
    pub http_code: u32,
}

impl Default for Step {
    fn default() -> Self {
        Self {
            id: String::new(),
            step_index: 0,
            r#type: StepType::Unknown,
            source: StepSource::Unknown,
            target: StepTarget::Unknown,
            status: StepStatus::Unknown,
            content: String::new(),
            content_delta: String::new(),
            thinking: String::new(),
            thinking_delta: String::new(),
            tool_calls: Vec::new(),
            error: String::new(),
            is_complete_response: None,
            structured_output: None,
            usage_metadata: None,
            cascade_id: String::new(),
            trajectory_id: String::new(),
            http_code: 0,
        }
    }
}

/// The result returned by a middleware hook.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookResult {
    /// True if the operation is allowed to proceed.
    pub allow: bool,
    /// Diagnostic or error message details.
    #[serde(default)]
    pub message: String,
}

/// Individual multiple-choice or freeform answer to an interactive user question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionResponse {
    /// Selected index choices (if multiple-choice).
    pub selected_option_ids: Option<Vec<String>>,
    /// Freeform response text.
    #[serde(default)]
    pub freeform_response: String,
    /// True if the question was skipped.
    #[serde(default)]
    pub skipped: bool,
}

/// Complete collection of responses answered to a set of interactive questions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionHookResult {
    /// List of user responses.
    pub responses: Vec<QuestionResponse>,
    /// True if the question panel dialogue was canceled.
    #[serde(default)]
    pub cancelled: bool,
}

/// Single choice option in a multiple-choice question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskQuestionOption {
    /// Unique identifier for this choice option.
    pub id: String,
    /// Visual label text for the choice.
    pub text: String,
}

/// Interactive user question entry structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AskQuestionEntry {
    /// Main question prompt text.
    pub question: String,
    /// List of multiple-choice options.
    pub options: Vec<AskQuestionOption>,
    /// True if multiple selections are supported.
    #[serde(default)]
    pub is_multi_select: bool,
}

/// Final summary outcome of a chat interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Combined text output returned.
    pub text: String,
    /// Combined reasoning thoughts.
    pub thinking: String,
    /// Sequence of intermediate execution steps.
    pub steps: Vec<Step>,
    /// Token usage metrics.
    pub usage_metadata: UsageMetadata,
}

/// Streaming fragment sent over chunk-based event listeners.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "chunk_type")]
pub enum StreamChunk {
    /// Streaming thinking fragment.
    Thought {
        /// Step index identifier.
        step_index: u32,
        /// Thinking segment.
        text: String,
    },
    /// Streaming text response fragment.
    Text {
        /// Step index identifier.
        step_index: u32,
        /// Text segment.
        text: String,
    },
    /// Complete parsed tool call definition.
    ToolCall(ToolCall),
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::field_reassign_with_default
    )]
    use super::*;
    use serde_json::json;

    #[test]
    fn test_tool_call_construction() {
        let tc = ToolCall {
            id: "call_1".to_string(),
            name: "read_file".to_string(),
            args: json!({"path": "/tmp/foo"}),
            canonical_path: None,
        };
        assert_eq!(tc.name, "read_file");
        assert_eq!(tc.args["path"], "/tmp/foo");
        assert_eq!(tc.id, "call_1");
        assert_eq!(tc.canonical_path, None);
    }

    #[test]
    fn test_tool_call_serialization() {
        let json_data = r#"{"id":"call_1","name":"read_file","args":{"path":"/tmp/foo"}}"#;
        let tc: ToolCall = serde_json::from_str(json_data).unwrap();
        assert_eq!(tc.name, "read_file");
        assert_eq!(tc.args["path"], "/tmp/foo");
        assert_eq!(tc.id, "call_1");
        assert_eq!(tc.canonical_path, None);
    }

    #[test]
    fn test_tool_result_success() {
        let tr = ToolResult {
            name: "sum_tool".to_string(),
            id: Some("call_1".to_string()),
            result: Some(json!(42)),
            error: None,
        };
        assert_eq!(tr.name, "sum_tool");
        assert_eq!(tr.result.unwrap(), 42);
        assert!(tr.error.is_none());
        assert_eq!(tr.id.unwrap(), "call_1");
    }

    #[test]
    fn test_tool_result_error() {
        let tr = ToolResult {
            name: "bad_tool".to_string(),
            id: None,
            result: None,
            error: Some("kaboom".to_string()),
        };
        assert_eq!(tr.name, "bad_tool");
        assert!(tr.result.is_none());
        assert_eq!(tr.error.unwrap(), "kaboom");
        assert!(tr.id.is_none());
    }

    #[test]
    fn test_tool_result_mutability() {
        let mut tr = ToolResult {
            name: "tool".to_string(),
            id: None,
            result: None,
            error: None,
        };
        tr.result = Some(json!("updated"));
        assert_eq!(tr.result.unwrap(), "updated");
    }

    #[test]
    fn test_step_defaults() {
        let step = Step::default();
        assert_eq!(step.id, "");
        assert_eq!(step.step_index, 0);
        assert!(matches!(step.r#type, StepType::Unknown));
        assert!(matches!(step.status, StepStatus::Unknown));
        assert!(matches!(step.source, StepSource::Unknown));
        assert_eq!(step.content, "");
        assert!(step.tool_calls.is_empty());
        assert_eq!(step.error, "");
    }

    #[test]
    fn test_step_mutability() {
        let mut step = Step::default();
        step.content = "goodbye".to_string();
        assert_eq!(step.content, "goodbye");
    }

    #[test]
    fn test_hook_result_defaults() {
        let hr = HookResult::default();
        assert!(!hr.allow); // derived default for bool in Rust is false
        assert_eq!(hr.message, "");
    }

    #[test]
    fn test_question_response_defaults() {
        let qr = QuestionResponse {
            selected_option_ids: None,
            freeform_response: String::new(),
            skipped: false,
        };
        assert!(qr.selected_option_ids.is_none());
        assert_eq!(qr.freeform_response, "");
        assert!(!qr.skipped);
    }

    #[test]
    fn test_question_response_skipped() {
        let qr = QuestionResponse {
            selected_option_ids: None,
            freeform_response: String::new(),
            skipped: true,
        };
        assert!(qr.skipped);
    }

    #[test]
    fn test_gemini_config_defaults() {
        let config = GeminiConfig::default();
        assert!(config.api_key.is_none());
        assert!(!config.vertex);
        assert!(config.project.is_none());
        assert!(config.location.is_none());
        assert_eq!(config.models.default.name, DEFAULT_MODEL);
        assert!(config.models.default.generation.thinking_level.is_none());
        assert!(config.enable_google_search.is_none());
        assert!(config.enable_url_context.is_none());
    }

    #[test]
    fn test_thinking_level_serialization() {
        let level = ThinkingLevel::Low;
        let json_str = serde_json::to_string(&level).unwrap();
        assert_eq!(json_str, "\"low\"");
    }

    #[test]
    fn test_capabilities_config_defaults() {
        let config = CapabilitiesConfig::default();
        assert!(config.enabled_tools.is_none());
        assert!(config.disabled_tools.is_none());
        assert!(config.compaction_threshold.is_none());
    }
}
