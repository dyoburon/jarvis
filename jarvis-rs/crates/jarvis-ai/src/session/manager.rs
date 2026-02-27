//! Session struct and conversation management.

use std::sync::atomic::AtomicBool;

use tracing::debug;

use crate::token_tracker::TokenTracker;
use crate::{Message, Role, ToolCall, ToolDefinition};

use super::types::ToolExecutor;

/// A conversation session with message history and tool execution.
pub struct Session {
    /// Conversation message history.
    pub(super) messages: Vec<Message>,
    /// System prompt (prepended to every API call).
    pub(super) system_prompt: Option<String>,
    /// Available tool definitions.
    pub(super) tools: Vec<ToolDefinition>,
    /// Tool executor callback.
    pub(super) tool_executor: Option<ToolExecutor>,
    /// Token usage tracker.
    pub(super) tracker: TokenTracker,
    /// Maximum tool-call loop iterations to prevent infinite loops.
    pub(super) max_tool_rounds: u32,
    /// Provider name for token tracking.
    pub(super) provider: String,
    /// Whether the session is currently processing a request.
    pub(super) busy: AtomicBool,
}

impl Session {
    pub fn new(provider: impl Into<String>) -> Self {
        Self {
            messages: Vec::new(),
            system_prompt: None,
            tools: Vec::new(),
            tool_executor: None,
            tracker: TokenTracker::new(),
            max_tool_rounds: 10,
            provider: provider.into(),
            busy: AtomicBool::new(false),
        }
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_tool_executor(mut self, executor: ToolExecutor) -> Self {
        self.tool_executor = Some(executor);
        self
    }

    pub fn with_max_tool_rounds(mut self, max: u32) -> Self {
        self.max_tool_rounds = max;
        self
    }

    pub(crate) fn execute_tool(&self, executor: &ToolExecutor, tool_call: &ToolCall) -> String {
        debug!(tool = %tool_call.name, "Executing tool");
        executor(&tool_call.name, &tool_call.arguments)
    }

    pub(crate) fn build_messages(&self) -> Vec<Message> {
        let mut msgs = Vec::new();
        if let Some(ref system) = self.system_prompt {
            msgs.push(Message {
                role: Role::System,
                content: system.clone(),
            });
        }
        msgs.extend(self.messages.clone());
        msgs
    }

    /// Get the full conversation history.
    pub fn messages(&self) -> &[Message] {
        &self.messages
    }

    /// Get the token tracker.
    pub fn tracker(&self) -> &TokenTracker {
        &self.tracker
    }

    /// Clear conversation history.
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Number of messages in history.
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
}

impl Default for Session {
    fn default() -> Self {
        Self::new("default")
    }
}
