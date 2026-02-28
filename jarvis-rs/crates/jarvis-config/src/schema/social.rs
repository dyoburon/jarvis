//! Social and presence configuration types.

use serde::{Deserialize, Serialize};

/// Presence system configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PresenceConfig {
    pub enabled: bool,
    pub server_url: String,
    pub heartbeat_interval: u32,
}

impl Default for PresenceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            server_url: String::new(),
            heartbeat_interval: 30,
        }
    }
}
