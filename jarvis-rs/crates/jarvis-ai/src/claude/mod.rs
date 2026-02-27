//! Anthropic Claude API client.
//!
//! Implements the `AiClient` trait for Claude models via the
//! Anthropic Messages API (https://api.anthropic.com/v1/messages).
//!
//! Uses Cloud OAuth tokens (via `CLAUDE_CODE_OAUTH_TOKEN`) for
//! authentication, matching the Python Jarvis implementation.

mod api;
mod client;
mod config;

pub use client::ClaudeClient;
pub use config::ClaudeConfig;
