//! Conversation session management.
//!
//! A `Session` holds the conversation history (messages), manages
//! context windows, and orchestrates the tool-call loop.

mod chat;
mod manager;
mod types;

pub use manager::Session;
pub use types::ToolExecutor;
