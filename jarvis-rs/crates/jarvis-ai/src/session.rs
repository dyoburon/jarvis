//! Conversation session management.
//!
//! A `Session` holds the conversation history (messages), manages
//! context windows, and orchestrates the tool-call loop.

use std::sync::atomic::{AtomicBool, Ordering};

use tracing::debug;

use crate::token_tracker::TokenTracker;
use crate::{AiClient, AiError, Message, Role, ToolCall, ToolDefinition};

/// Callback for executing tool calls. Takes a tool name + arguments,
/// returns the tool's output string.
pub type ToolExecutor = Box<dyn Fn(&str, &serde_json::Value) -> String + Send + Sync>;

/// Guard that clears the `busy` flag on drop, ensuring it is always released
/// even if the future is cancelled or an early return occurs.
struct BusyGuard<'a> {
    flag: &'a AtomicBool,
}

impl<'a> BusyGuard<'a> {
    /// Attempt to acquire the busy lock. Returns `Err` if already busy.
    fn acquire(flag: &'a AtomicBool) -> Result<Self, AiError> {
        if flag
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            return Err(AiError::ApiError(
                "Session is busy with another request".into(),
            ));
        }
        Ok(Self { flag })
    }
}

impl Drop for BusyGuard<'_> {
    fn drop(&mut self) {
        self.flag.store(false, Ordering::Release);
    }
}

/// A conversation session with message history and tool execution.
pub struct Session {
    /// Conversation message history.
    messages: Vec<Message>,
    /// System prompt (prepended to every API call).
    system_prompt: Option<String>,
    /// Available tool definitions.
    tools: Vec<ToolDefinition>,
    /// Tool executor callback.
    tool_executor: Option<ToolExecutor>,
    /// Token usage tracker.
    tracker: TokenTracker,
    /// Maximum tool-call loop iterations to prevent infinite loops.
    max_tool_rounds: u32,
    /// Provider name for token tracking.
    provider: String,
    /// Whether the session is currently processing a request.
    busy: AtomicBool,
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

    /// Add a user message and get the assistant's response.
    /// If the AI calls tools, this runs the tool-call loop automatically.
    pub async fn chat(
        &mut self,
        client: &dyn AiClient,
        user_message: impl Into<String>,
    ) -> Result<String, AiError> {
        let _guard = BusyGuard::acquire(&self.busy)?;

        self.messages.push(Message {
            role: Role::User,
            content: user_message.into(),
        });

        let mut messages = self.build_messages();
        let mut rounds = 0;

        loop {
            let response = client.send_message(&messages, &self.tools).await?;
            self.tracker.record(&self.provider, &response.usage);

            if response.tool_calls.is_empty() || self.tool_executor.is_none() {
                // No tool calls — we have the final response
                self.messages.push(Message {
                    role: Role::Assistant,
                    content: response.content.clone(),
                });
                return Ok(response.content);
            }

            // Execute tool calls
            rounds += 1;
            if rounds > self.max_tool_rounds {
                debug!("Max tool rounds reached, returning partial response");
                self.messages.push(Message {
                    role: Role::Assistant,
                    content: response.content.clone(),
                });
                return Ok(response.content);
            }

            // Add assistant message with tool calls
            messages.push(Message {
                role: Role::Assistant,
                content: response.content.clone(),
            });

            // Execute each tool and add results
            let executor = self.tool_executor.as_ref().unwrap();
            for tool_call in &response.tool_calls {
                let result = self.execute_tool(executor, tool_call);
                messages.push(Message {
                    role: Role::Tool,
                    content: format!("[Tool Result: {}]\n{}", tool_call.name, result),
                });
            }
        }
    }

    /// Send a message with streaming, returning the full response.
    pub async fn chat_streaming(
        &mut self,
        client: &dyn AiClient,
        user_message: impl Into<String>,
        on_chunk: Box<dyn Fn(String) + Send + Sync>,
    ) -> Result<String, AiError> {
        let _guard = BusyGuard::acquire(&self.busy)?;

        self.messages.push(Message {
            role: Role::User,
            content: user_message.into(),
        });

        let messages = self.build_messages();
        let response = client
            .send_message_streaming(&messages, &self.tools, on_chunk)
            .await?;

        self.tracker.record(&self.provider, &response.usage);
        self.messages.push(Message {
            role: Role::Assistant,
            content: response.content.clone(),
        });

        Ok(response.content)
    }

    fn execute_tool(&self, executor: &ToolExecutor, tool_call: &ToolCall) -> String {
        debug!(tool = %tool_call.name, "Executing tool");
        executor(&tool_call.name, &tool_call.arguments)
    }

    fn build_messages(&self) -> Vec<Message> {
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
