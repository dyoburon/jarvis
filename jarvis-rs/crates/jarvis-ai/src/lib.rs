//! AI engine for Jarvis.
//!
//! Provides Claude, Gemini, and Whisper API clients with:
//! - Streaming (SSE) support
//! - Tool calling (function use)
//! - Session management with automatic tool-call loops
//! - Token usage tracking
//! - Skill-based routing between providers

pub mod claude;
pub mod gemini;
pub mod router;
pub mod session;
pub mod streaming;
pub mod token_tracker;
pub mod tools;
pub mod whisper;

use async_trait::async_trait;

pub use claude::{ClaudeClient, ClaudeConfig};
pub use gemini::{GeminiClient, GeminiConfig};
pub use router::{Provider, Skill, SkillRouter};
pub use session::Session;
pub use token_tracker::TokenTracker;
pub use whisper::{WhisperClient, WhisperConfig};

#[async_trait]
pub trait AiClient: Send + Sync {
    async fn send_message(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> Result<AiResponse, AiError>;

    async fn send_message_streaming(
        &self,
        messages: &[Message],
        tools: &[ToolDefinition],
        on_chunk: Box<dyn Fn(String) + Send + Sync>,
    ) -> Result<AiResponse, AiError>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone)]
pub struct AiResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Default)]
pub struct TokenUsage {
    pub input_tokens: u64,
    pub output_tokens: u64,
}

impl TokenUsage {
    pub fn total_tokens(&self) -> u64 {
        self.input_tokens.saturating_add(self.output_tokens)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AiError {
    #[error("API error: {0}")]
    ApiError(String),
    #[error("Rate limited")]
    RateLimited,
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Timeout")]
    Timeout,
}
