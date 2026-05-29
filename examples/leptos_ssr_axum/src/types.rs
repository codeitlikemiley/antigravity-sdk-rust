use serde::{Deserialize, Serialize};

/// A single renderable block in the chat timeline.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum MessageBlock {
    /// A user-sent prompt
    UserMessage { id: u64, content: String, timestamp: u64 },

    /// Streaming/completed reasoning text
    Thinking { id: u64, content: String, is_streaming: bool },

    /// A tool invocation (running state)
    ToolCall {
        id: u64,
        call_id: String,
        name: String,
        args: serde_json::Value,
        canonical_path: Option<String>,
        /// Human-readable label from the agent step (e.g. "Change Directory", "Create Rust Project").
        /// Used as the card title so users see intent, not the raw tool name.
        #[serde(default)]
        label: Option<String>,
        status: ToolCallStatus,
        #[serde(default)]
        subagent_trajectory_id: Option<String>,
        #[serde(default)]
        subagent_blocks: Vec<MessageBlock>,
    },

    /// A tool result (after invocation completes)
    ToolResult {
        id: u64,
        call_id: String,
        name: String,
        result: Option<serde_json::Value>,
        error: Option<String>,
    },

    /// Completed assistant text response
    AssistantMessage { id: u64, content: String, timestamp: u64 },

    /// Interactive question from agent (pauses stream)
    Question {
        id: u64,
        trajectory_id: String,
        step_index: u32,
        questions: Vec<AskQuestionEntry>,
        answered: bool,
    },

    /// Tool confirmation request (pauses stream)
    Confirmation {
        id: u64,
        trajectory_id: String,
        step_index: u32,
        tool_call: ClientToolCall,
        /// None = pending, Some(true) = accepted, Some(false) = rejected
        decision: Option<bool>,
    },

    /// Context compaction event
    Compaction { id: u64, step_index: u32 },

    /// Token usage summary (emitted at end of turn)
    UsageSummary {
        id: u64,
        prompt_tokens: i32,
        output_tokens: i32,
        thinking_tokens: i32,
    },

    /// Agent task completion marker
    Finish {
        id: u64,
        structured_output: Option<serde_json::Value>,
    },

    /// Error event
    Error {
        id: u64,
        message: String,
        http_code: Option<u32>,
    },
}

impl MessageBlock {
    pub fn id(&self) -> u64 {
        match self {
            Self::UserMessage { id, .. } |
            Self::Thinking { id, .. } |
            Self::ToolCall { id, .. } |
            Self::ToolResult { id, .. } |
            Self::AssistantMessage { id, .. } |
            Self::Question { id, .. } |
            Self::Confirmation { id, .. } |
            Self::Compaction { id, .. } |
            Self::UsageSummary { id, .. } |
            Self::Finish { id, .. } |
            Self::Error { id, .. } => *id,
        }
    }
}

/// Status of a tool call block.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallStatus {
    Running,
    Done,
    Error,
}

/// A tool call used in Confirmation blocks and SSE wire format.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ClientToolCall {
    /// Accept both "call_id" (canonical) and "id" (legacy/SDK) from the wire.
    #[serde(alias = "id", default)]
    pub call_id: String,
    pub name: String,
    pub args: serde_json::Value,
    pub canonical_path: Option<String>,
}

/// An option in an agent question.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct QuestionOption {
    pub id: String,
    pub text: String,
}

/// A single question from the agent.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AskQuestionEntry {
    pub question: String,
    pub options: Vec<QuestionOption>,
    pub is_multi_select: bool,
}

/// A chat session (container of a timeline of blocks).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[allow(dead_code)]
pub struct ChatSession {
    /// Session identifier (e.g. "sess_1716278400123")
    pub id: String,
    /// Display title — auto-generated from first user message, or "New Chat"
    pub title: String,
    /// Unix timestamp (seconds) when the session was created
    pub created_at: u64,
    /// Unix timestamp (seconds) when the session was last updated
    pub updated_at: u64,
    /// Ordered list of message blocks
    pub blocks: Vec<MessageBlock>,
    /// Monotonically increasing counter for assigning unique block IDs
    pub block_id_counter: u64,
}

#[allow(dead_code)]
impl ChatSession {
    /// Create a new empty session with the given ID.
    /// Timestamps are set to 0 — caller should supply the real unix time if needed.
    pub fn new(id: String) -> Self {
        #[cfg(feature = "ssr")]
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        #[cfg(not(feature = "ssr"))]
        let now = 0;

        Self {
            id,
            title: "New Chat".to_string(),
            created_at: now,
            updated_at: now,
            blocks: Vec::new(),
            block_id_counter: 0,
        }
    }

    /// Allocate the next monotonic block ID for this session.
    pub fn next_id(&mut self) -> u64 {
        self.block_id_counter += 1;
        self.block_id_counter
    }
}

/// Index of all sessions — stored separately so the sidebar can list sessions
/// without loading every block vector.
#[derive(Clone, Debug, Serialize, Deserialize, Default, PartialEq)]
#[allow(dead_code)]
pub struct SessionIndex {
    pub sessions: Vec<SessionMeta>,
}

/// Lightweight per-session metadata stored in the session index.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SessionMeta {
    pub id: String,
    pub title: String,
    pub created_at: u64,
    pub updated_at: u64,
}

/// Legacy flat chat message (retained for backward compatibility).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub id: u64,
    pub role: String,
    pub content: String,
    pub timestamp: u64,
    pub thinking: Option<String>,
    pub tool_calls: Option<Vec<ClientToolCall>>,
}

/// Individual multiple-choice or freeform answer to an interactive user question.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// Request body for `POST /answer`.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnswerPayload {
    pub session_id: String,
    pub trajectory_id: String,
    pub step_index: u32,
    pub responses: Vec<QuestionResponse>,
    pub cancelled: bool,
}

/// Request body for `POST /confirm`.
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfirmPayload {
    pub session_id: String,
    pub trajectory_id: String,
    pub step_index: u32,
    pub accepted: bool,
    /// If true, all future requests for this tool in this session are auto-approved.
    pub allow_for_session: bool,
    /// Name of the tool being confirmed (needed for auto-allow tracking).
    pub tool_name: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AgentServerUrl(pub String);


