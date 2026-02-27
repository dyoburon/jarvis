//! Chat history management.
//!
//! Stores messages per channel with a bounded ring buffer so memory
//! usage stays predictable.

use std::collections::{HashMap, VecDeque};

use serde::{Deserialize, Serialize};

/// A single chat message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub user_id: String,
    pub display_name: String,
    pub channel: String,
    pub content: String,
    pub timestamp: String,
    pub reply_to: Option<String>,
}

/// Configuration for chat history storage.
#[derive(Debug, Clone)]
pub struct ChatHistoryConfig {
    /// Maximum messages to retain per channel.
    pub max_messages_per_channel: usize,
}

impl Default for ChatHistoryConfig {
    fn default() -> Self {
        Self {
            max_messages_per_channel: 500,
        }
    }
}

/// In-memory chat history, keyed by channel name.
pub struct ChatHistory {
    config: ChatHistoryConfig,
    channels: HashMap<String, VecDeque<ChatMessage>>,
}

impl ChatHistory {
    pub fn new(config: ChatHistoryConfig) -> Self {
        Self {
            config,
            channels: HashMap::new(),
        }
    }

    /// Push a message into a channel. Oldest messages are evicted when the
    /// buffer is full.
    pub fn push(&mut self, msg: ChatMessage) {
        let buf = self.channels.entry(msg.channel.clone()).or_default();
        if buf.len() >= self.config.max_messages_per_channel {
            buf.pop_front();
        }
        buf.push_back(msg);
    }

    /// Get the most recent `limit` messages from a channel (oldest first).
    pub fn recent(&self, channel: &str, limit: usize) -> Vec<&ChatMessage> {
        match self.channels.get(channel) {
            Some(buf) => {
                let skip = buf.len().saturating_sub(limit);
                buf.iter().skip(skip).collect()
            }
            None => Vec::new(),
        }
    }

    /// Get all messages in a channel.
    pub fn all(&self, channel: &str) -> Vec<&ChatMessage> {
        match self.channels.get(channel) {
            Some(buf) => buf.iter().collect(),
            None => Vec::new(),
        }
    }

    /// Clear a specific channel's history.
    pub fn clear_channel(&mut self, channel: &str) {
        self.channels.remove(channel);
    }

    /// Clear all history.
    pub fn clear_all(&mut self) {
        self.channels.clear();
    }

    /// List channels that have messages.
    pub fn active_channels(&self) -> Vec<&str> {
        self.channels.keys().map(|s| s.as_str()).collect()
    }

    /// Total number of stored messages across all channels.
    pub fn total_messages(&self) -> usize {
        self.channels.values().map(|b| b.len()).sum()
    }
}

impl Default for ChatHistory {
    fn default() -> Self {
        Self::new(ChatHistoryConfig::default())
    }
}
