//! Configuration and event types for the presence client.

use crate::protocol::OnlineUser;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the presence client.
#[derive(Debug, Clone)]
pub struct PresenceConfig {
    /// Supabase project reference (e.g., "ojmqzagktzkualzgpcbq").
    pub project_ref: String,
    /// Supabase anon key (publishable).
    pub api_key: String,
    /// Optional JWT for authenticated connections.
    pub access_token: Option<String>,
    /// Heartbeat interval in seconds.
    pub heartbeat_interval: u64,
    /// Reconnect delay (base) in seconds.
    pub reconnect_delay: u64,
    /// Maximum reconnect delay in seconds.
    pub max_reconnect_delay: u64,
}

impl Default for PresenceConfig {
    fn default() -> Self {
        Self {
            project_ref: String::new(),
            api_key: String::new(),
            access_token: None,
            heartbeat_interval: 25,
            reconnect_delay: 1,
            max_reconnect_delay: 30,
        }
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Events emitted by the presence system for the UI to consume.
#[derive(Debug, Clone)]
pub enum PresenceEvent {
    Connected {
        online_count: u32,
    },
    Disconnected,
    UserOnline(OnlineUser),
    UserOffline {
        user_id: String,
        display_name: String,
    },
    ActivityChanged(OnlineUser),
    GameInvite {
        user_id: String,
        display_name: String,
        game: String,
        code: Option<String>,
    },
    Poked {
        user_id: String,
        display_name: String,
    },
    ChatMessage {
        user_id: String,
        display_name: String,
        channel: String,
        content: String,
    },
    Error(String),
}
