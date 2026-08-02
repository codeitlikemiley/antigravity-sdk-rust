//! Stateful conversation tracking and event chunk streaming.
//!
//! This module provides the [`Conversation`] struct, which coordinates an active session's
//! event stream, aggregates history steps, tracks token usage metadata, and filters thinking/text deltas.

use crate::connection::{AnyConnection, Connection};
use crate::types::{
    ChatResponse, Step, StepSource, StepTarget, StepType, StreamChunk, UsageMetadata,
};
use futures_util::StreamExt;
use futures_util::stream::{self, BoxStream};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

const DEFAULT_MAX_HISTORY_SIZE: usize = 10_000;

/// Internal accumulator of conversation steps and usage metrics.
#[derive(Debug)]
pub struct ConversationState {
    /// Ordered list of all executed steps (including prompts, tool calls, results, and responses).
    pub steps: Vec<Step>,
    /// Step indices marking the start of each user prompt turn.
    pub turn_start_indices: Vec<usize>,
    /// Step indices marking where state compaction was performed.
    pub compaction_indices: Vec<usize>,
    /// Total cumulative LLM token consumption across the entire session.
    pub cumulative_usage: UsageMetadata,
    /// Token usage metrics for the current active turn, if any.
    pub turn_usage: Option<UsageMetadata>,
}

/// A stateful wrapper managing an active agentic session and its history.
///
/// `Conversation` consumes step events from an underlying [`Connection`], updates the cumulative history,
/// tracks token usage, and provides high-level APIs to chat, stream structured events, and wait for run completions.
pub struct Conversation {
    conn: AnyConnection,
    max_history_size: usize,
    state: Arc<Mutex<ConversationState>>,
}

impl std::fmt::Debug for Conversation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Conversation")
            .field("conversation_id", &self.conversation_id())
            .field("max_history_size", &self.max_history_size)
            .finish_non_exhaustive()
    }
}

impl Conversation {
    /// Creates a new `Conversation` instance wrapping a [`Connection`].
    ///
    /// Optionally restricts the memory storage to `max_history_size` steps (default is 10,000 steps).
    /// If `max_history_size` is set to `0`, state trimming is disabled.
    pub fn new(conn: AnyConnection, max_history_size: Option<usize>) -> Self {
        Self {
            conn,
            max_history_size: max_history_size.unwrap_or(DEFAULT_MAX_HISTORY_SIZE),
            state: Arc::new(Mutex::new(ConversationState {
                steps: Vec::new(),
                turn_start_indices: Vec::new(),
                compaction_indices: Vec::new(),
                cumulative_usage: UsageMetadata::default(),
                turn_usage: None,
            })),
        }
    }

    /// Returns the underlying [`Connection`].
    pub fn connection(&self) -> AnyConnection {
        self.conn.clone()
    }

    /// Returns the conversation ID assigned to the session.
    pub fn conversation_id(&self) -> &str {
        self.conn.conversation_id()
    }

    /// Returns whether the connection is currently idle.
    pub fn is_idle(&self) -> bool {
        self.conn.is_idle()
    }

    /// Retrieves a copy of the current conversation history steps.
    pub async fn history(&self) -> Vec<Step> {
        self.state.lock().await.steps.clone()
    }

    /// Returns the total number of user-initiated turns executed in this session.
    pub async fn turn_count(&self) -> usize {
        self.state.lock().await.turn_start_indices.len()
    }

    /// Returns the step indices where compaction (history compression) occurred.
    pub async fn compaction_indices(&self) -> Vec<usize> {
        self.state.lock().await.compaction_indices.clone()
    }

    /// Scans the history backward and returns the text content of the last completed model response.
    pub async fn last_response(&self) -> String {
        let state = self.state.lock().await;
        let response = state
            .steps
            .iter()
            .rev()
            .find(|step| step.is_complete_response == Some(true))
            .map(|step| step.content.clone())
            .unwrap_or_default();
        drop(state);
        response
    }

    /// Returns the total token usage accumulated over all turns in the session.
    pub async fn total_usage(&self) -> UsageMetadata {
        self.state.lock().await.cumulative_usage.clone()
    }

    /// Returns the token usage metrics from the last completed turn.
    pub async fn last_turn_usage(&self) -> Option<UsageMetadata> {
        self.state.lock().await.turn_usage.clone()
    }

    /// Resets the conversation state, clearing all steps, compaction boundaries, and usage statistics.
    pub async fn clear_history(&self) {
        let mut state = self.state.lock().await;
        state.steps.clear();
        state.turn_start_indices.clear();
        state.compaction_indices.clear();
        state.cumulative_usage = UsageMetadata::default();
        state.turn_usage = None;
    }

    /// Sends a text prompt to the connection and registers the turn start boundary.
    ///
    /// # Errors
    ///
    /// Returns an error if the underlying connection fails to transmit the prompt.
    pub async fn send(&self, prompt: &str) -> Result<(), anyhow::Error> {
        // If not idle, wait for it
        if !self.conn.is_idle() {
            // Note: Unlike Python's runtime RuntimeError handling, in Rust we can just wait
            // or let the stream run-loop handle it.
        }
        let mut state = self.state.lock().await;
        let len = state.steps.len();
        state.turn_start_indices.push(len);
        state.turn_usage = None;
        drop(state);
        self.conn.send(prompt).await
    }

    /// Subscribes to step updates from the connection, inserting them into history and enforcing history limits.
    pub fn receive_steps(&self) -> BoxStream<'static, Result<Step, anyhow::Error>> {
        let conn_stream = self.conn.receive_steps();
        let state = self.state.clone();
        let max_history = self.max_history_size;

        conn_stream
            .then(move |step_res| {
                let state = state.clone();
                async move {
                    match step_res {
                        Ok(step) => {
                            let mut s = state.lock().await;
                            s.steps.push(step.clone());
                            if step.r#type == StepType::Compaction {
                                let len = s.steps.len();
                                s.compaction_indices.push(len - 1);
                            }
                            if let Some(ref usage) = step.usage_metadata {
                                s.cumulative_usage.prompt_token_count += usage.prompt_token_count;
                                s.cumulative_usage.cached_content_token_count +=
                                    usage.cached_content_token_count;
                                s.cumulative_usage.candidates_token_count +=
                                    usage.candidates_token_count;
                                s.cumulative_usage.thoughts_token_count +=
                                    usage.thoughts_token_count;
                                s.cumulative_usage.total_token_count += usage.total_token_count;

                                let mut turn_usage = s.turn_usage.take().unwrap_or_default();
                                turn_usage.prompt_token_count += usage.prompt_token_count;
                                turn_usage.cached_content_token_count +=
                                    usage.cached_content_token_count;
                                turn_usage.candidates_token_count += usage.candidates_token_count;
                                turn_usage.thoughts_token_count += usage.thoughts_token_count;
                                turn_usage.total_token_count += usage.total_token_count;
                                s.turn_usage = Some(turn_usage);
                            }

                            // Enforce max history size
                            if max_history > 0 && s.steps.len() > max_history {
                                let overflow = s.steps.len() - max_history;
                                s.steps.drain(0..overflow);
                                s.turn_start_indices = s
                                    .turn_start_indices
                                    .iter()
                                    .filter_map(|&idx| {
                                        if idx >= overflow {
                                            Some(idx - overflow)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                                s.compaction_indices = s
                                    .compaction_indices
                                    .iter()
                                    .filter_map(|&idx| {
                                        if idx >= overflow {
                                            Some(idx - overflow)
                                        } else {
                                            None
                                        }
                                    })
                                    .collect();
                            }
                            drop(s);

                            Ok(step)
                        }
                        Err(e) => Err(e),
                    }
                }
            })
            .boxed()
    }

    /// Filters and maps the step event stream to yielding high-level [`StreamChunk`] deltas.
    pub fn receive_chunks(&self) -> BoxStream<'static, Result<StreamChunk, anyhow::Error>> {
        let steps = self.receive_steps();
        let mut seen_tool_ids = HashSet::new();

        steps
            .flat_map(move |step_res| {
                let mut chunks = Vec::new();
                match step_res {
                    Ok(step) => {
                        let is_model = step.source == StepSource::Model;
                        let is_target_user = step.target == StepTarget::User;

                        if is_model && is_target_user {
                            if !step.thinking_delta.is_empty() {
                                chunks.push(Ok(StreamChunk::Thought {
                                    step_index: step.step_index,
                                    text: step.thinking_delta.clone(),
                                }));
                            }
                            if !step.content_delta.is_empty() {
                                chunks.push(Ok(StreamChunk::Text {
                                    step_index: step.step_index,
                                    text: step.content_delta.clone(),
                                }));
                            }
                        }

                        for call in step.tool_calls {
                            if call.id.is_empty() || seen_tool_ids.insert(call.id.clone()) {
                                chunks.push(Ok(StreamChunk::ToolCall(call)));
                            }
                        }
                    }
                    Err(e) => {
                        chunks.push(Err(e));
                    }
                }
                stream::iter(chunks)
            })
            .boxed()
    }

    /// Starts a prompt turn and returns a stream of [`StreamChunk`] events.
    ///
    /// # Errors
    ///
    /// Returns an error if sending the prompt fails.
    pub async fn chat(
        &self,
        prompt: &str,
    ) -> Result<BoxStream<'static, Result<StreamChunk, anyhow::Error>>, anyhow::Error> {
        self.send(prompt).await?;
        Ok(self.receive_chunks())
    }

    /// Starts a prompt turn and resolves once the model completes its response.
    ///
    /// # Errors
    ///
    /// Returns an error if sending the prompt or receiving chunk responses fails.
    pub async fn chat_to_completion(&self, prompt: &str) -> Result<ChatResponse, anyhow::Error> {
        let mut chunks = self.chat(prompt).await?;
        let mut text = String::new();
        let mut thinking = String::new();
        while let Some(chunk_res) = chunks.next().await {
            match chunk_res? {
                StreamChunk::Text { text: delta, .. } => {
                    text.push_str(&delta);
                }
                StreamChunk::Thought { text: delta, .. } => {
                    thinking.push_str(&delta);
                }
                StreamChunk::ToolCall(_) => {}
            }
        }
        // Steps for THIS turn, not the whole session. `history()` returns
        // everything, so a long conversation returned the entire transcript on
        // every reply -- growing without bound and making the field useless for
        // "what just happened". turn_start_indices is already tracked for this.
        let steps = {
            let state = self.state.lock().await;
            let start = state.turn_start_indices.last().copied().unwrap_or(0);
            state
                .steps
                .get(start..)
                .map(<[Step]>::to_vec)
                .unwrap_or_default()
        };
        let usage_metadata = self.total_usage().await;
        Ok(ChatResponse {
            text,
            thinking,
            steps,
            usage_metadata,
        })
    }

    /// Gracefully closes the underlying connection.
    ///
    /// # Errors
    ///
    /// Returns an error if disconnecting the transport layer fails.
    pub async fn disconnect(&self) -> Result<(), anyhow::Error> {
        self.conn.disconnect().await
    }
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::field_reassign_with_default,
        clippy::similar_names,
        clippy::single_match,
        clippy::match_wildcard_for_single_variants,
        clippy::manual_string_new
    )]
    use super::*;
    use crate::connection::{AnyConnection, MockConnection};
    use crate::types::{StepSource, StepTarget, StepType, ToolCall};
    use futures_util::StreamExt;

    fn test_setup(
        id: &str,
        max_history_size: Option<usize>,
    ) -> (Arc<MockConnection>, Conversation) {
        let mock = Arc::new(MockConnection::new(id));
        let conv = Conversation::new(AnyConnection::Mock(mock.clone()), max_history_size);
        (mock, conv)
    }

    #[tokio::test]
    async fn test_conversation_initialization() {
        let (_conn, conv) = test_setup("conv-123", Some(10));
        assert_eq!(conv.conversation_id(), "conv-123");
        assert!(conv.is_idle());
        assert_eq!(conv.history().await.len(), 0);
        assert_eq!(conv.turn_count().await, 0);
    }

    #[tokio::test]
    async fn test_send_records_turn_boundary() {
        let (_conn, conv) = test_setup("conv-123", Some(10));
        conv.send("hello").await.unwrap();
        assert_eq!(conv.turn_count().await, 1);
        conv.send("world").await.unwrap();
        assert_eq!(conv.turn_count().await, 2);
    }

    #[tokio::test]
    async fn test_receive_steps_accumulates_history() {
        let (conn, conv) = test_setup("conv-123", Some(10));
        let step1 = Step {
            id: "1".to_string(),
            step_index: 1,
            r#type: StepType::TextResponse,
            source: StepSource::Model,
            target: StepTarget::User,
            content: "hello".to_string(),
            is_complete_response: Some(true),
            ..Default::default()
        };
        conn.steps_to_yield.lock().unwrap().push(step1);

        let mut steps = conv.receive_steps();
        while let Some(res) = steps.next().await {
            res.unwrap();
        }

        let hist = conv.history().await;
        assert_eq!(hist.len(), 1);
        assert_eq!(hist[0].content, "hello");
        assert_eq!(conv.last_response().await, "hello");
    }

    #[tokio::test]
    async fn test_compaction_indices_tracked() {
        let (conn, conv) = test_setup("conv-123", Some(10));
        let step1 = Step {
            id: "1".to_string(),
            step_index: 1,
            r#type: StepType::Compaction,
            source: StepSource::Model,
            target: StepTarget::User,
            ..Default::default()
        };
        conn.steps_to_yield.lock().unwrap().push(step1);

        let mut steps = conv.receive_steps();
        while let Some(res) = steps.next().await {
            res.unwrap();
        }

        assert_eq!(conv.compaction_indices().await, vec![0]);
    }

    #[tokio::test]
    async fn test_max_history_size_trimming() {
        let (conn, conv) = test_setup("conv-123", Some(3));
        for i in 0..5 {
            conn.steps_to_yield.lock().unwrap().push(Step {
                id: i.to_string(),
                step_index: i,
                content: format!("step-{}", i),
                ..Default::default()
            });
        }

        let mut steps = conv.receive_steps();
        while let Some(res) = steps.next().await {
            res.unwrap();
        }

        let hist = conv.history().await;
        assert_eq!(hist.len(), 3);
        assert_eq!(hist[0].content, "step-2");
        assert_eq!(hist[2].content, "step-4");
    }

    #[tokio::test]
    async fn test_max_history_size_zero_disables_trimming() {
        let (conn, conv) = test_setup("conv-123", Some(0));
        for i in 0..5 {
            conn.steps_to_yield.lock().unwrap().push(Step {
                id: i.to_string(),
                step_index: i,
                content: format!("step-{}", i),
                ..Default::default()
            });
        }

        let mut steps = conv.receive_steps();
        while let Some(res) = steps.next().await {
            res.unwrap();
        }

        let hist = conv.history().await;
        assert_eq!(hist.len(), 5);
    }

    #[tokio::test]
    async fn test_receive_chunks_routing() {
        let (conn, conv) = test_setup("conv-123", Some(10));
        let step = Step {
            id: "1".to_string(),
            step_index: 1,
            r#type: StepType::TextResponse,
            source: StepSource::Model,
            target: StepTarget::User,
            content_delta: "hello".to_string(),
            thinking_delta: "reasoning".to_string(),
            ..Default::default()
        };
        conn.steps_to_yield.lock().unwrap().push(step);

        let mut chunks = conv.receive_chunks();
        let mut text = String::new();
        let mut thought = String::new();
        while let Some(res) = chunks.next().await {
            match res.unwrap() {
                StreamChunk::Text { text: delta, .. } => text.push_str(&delta),
                StreamChunk::Thought { text: delta, .. } => thought.push_str(&delta),
                _ => {}
            }
        }

        assert_eq!(text, "hello");
        assert_eq!(thought, "reasoning");
    }

    #[tokio::test]
    async fn test_receive_chunks_environmental_filtering() {
        let (conn, conv) = test_setup("conv-123", Some(10));
        let step = Step {
            id: "1".to_string(),
            step_index: 1,
            r#type: StepType::TextResponse,
            source: StepSource::Model,
            target: StepTarget::Environment,
            content_delta: "env content".to_string(),
            ..Default::default()
        };
        conn.steps_to_yield.lock().unwrap().push(step);

        let mut chunks = conv.receive_chunks();
        let mut text = String::new();
        while let Some(res) = chunks.next().await {
            match res.unwrap() {
                StreamChunk::Text { text: delta, .. } => text.push_str(&delta),
                _ => {}
            }
        }

        assert_eq!(text, "");
    }

    #[tokio::test]
    async fn test_receive_chunks_tool_calls_deduplication() {
        let (conn, conv) = test_setup("conv-123", Some(10));
        let tc = ToolCall {
            id: "call_a".to_string(),
            name: "tool_1".to_string(),
            args: serde_json::Value::Null,
            canonical_path: None,
        };
        let step = Step {
            id: "1".to_string(),
            step_index: 1,
            r#type: StepType::ToolCall,
            source: StepSource::Model,
            target: StepTarget::User,
            tool_calls: vec![tc.clone(), tc.clone()],
            ..Default::default()
        };
        conn.steps_to_yield.lock().unwrap().push(step);

        let mut chunks = conv.receive_chunks();
        let mut tool_calls = Vec::new();
        while let Some(res) = chunks.next().await {
            if let StreamChunk::ToolCall(c) = res.unwrap() {
                tool_calls.push(c);
            }
        }

        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].id, "call_a");
    }

    #[tokio::test]
    async fn test_receive_chunks_empty_tool_id_no_dedup() {
        let (conn, conv) = test_setup("conv-123", Some(10));
        let tc = ToolCall {
            id: "".to_string(),
            name: "tool_1".to_string(),
            args: serde_json::Value::Null,
            canonical_path: None,
        };
        let step = Step {
            id: "1".to_string(),
            step_index: 1,
            r#type: StepType::ToolCall,
            source: StepSource::Model,
            target: StepTarget::User,
            tool_calls: vec![tc.clone(), tc.clone()],
            ..Default::default()
        };
        conn.steps_to_yield.lock().unwrap().push(step);

        let mut chunks = conv.receive_chunks();
        let mut tool_calls = Vec::new();
        while let Some(res) = chunks.next().await {
            if let StreamChunk::ToolCall(c) = res.unwrap() {
                tool_calls.push(c);
            }
        }

        assert_eq!(tool_calls.len(), 2);
    }
}
