//! Channel management for group communication.
//!
//! Channels are named rooms that users can join and leave. Every Jarvis
//! instance is automatically a member of the `general` channel.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

/// A communication channel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub name: String,
    pub description: String,
    /// User IDs currently in this channel.
    #[serde(skip)]
    pub members: HashSet<String>,
}

/// Manages the set of available channels.
pub struct ChannelManager {
    channels: HashMap<String, Channel>,
}

impl ChannelManager {
    pub fn new() -> Self {
        let mut mgr = Self {
            channels: HashMap::new(),
        };
        // Default channels
        mgr.create("general", "General chat");
        mgr.create("games", "Game invites and coordination");
        mgr
    }

    /// Create a new channel. Returns false if it already exists.
    pub fn create(&mut self, name: &str, description: &str) -> bool {
        if self.channels.contains_key(name) {
            return false;
        }
        self.channels.insert(
            name.to_string(),
            Channel {
                name: name.to_string(),
                description: description.to_string(),
                members: HashSet::new(),
            },
        );
        true
    }

    /// Join a user to a channel. Creates the channel if it doesn't exist.
    pub fn join(&mut self, channel: &str, user_id: &str) {
        let ch = self
            .channels
            .entry(channel.to_string())
            .or_insert_with(|| Channel {
                name: channel.to_string(),
                description: String::new(),
                members: HashSet::new(),
            });
        ch.members.insert(user_id.to_string());
    }

    /// Remove a user from a channel.
    pub fn leave(&mut self, channel: &str, user_id: &str) {
        if let Some(ch) = self.channels.get_mut(channel) {
            ch.members.remove(user_id);
        }
    }

    /// Remove a user from all channels.
    pub fn leave_all(&mut self, user_id: &str) {
        for ch in self.channels.values_mut() {
            ch.members.remove(user_id);
        }
    }

    /// List all channels.
    pub fn list(&self) -> Vec<&Channel> {
        self.channels.values().collect()
    }

    /// Get a channel by name.
    pub fn get(&self, name: &str) -> Option<&Channel> {
        self.channels.get(name)
    }

    /// Get the set of member user IDs for a channel.
    pub fn members(&self, channel: &str) -> HashSet<String> {
        self.channels
            .get(channel)
            .map(|ch| ch.members.clone())
            .unwrap_or_default()
    }

    /// List channel names a user belongs to.
    pub fn user_channels(&self, user_id: &str) -> Vec<&str> {
        self.channels
            .values()
            .filter(|ch| ch.members.contains(user_id))
            .map(|ch| ch.name.as_str())
            .collect()
    }
}

impl Default for ChannelManager {
    fn default() -> Self {
        Self::new()
    }
}
