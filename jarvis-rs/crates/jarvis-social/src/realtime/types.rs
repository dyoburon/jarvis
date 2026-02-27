//! Configuration, protocol types, and event/command enums for the realtime client.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for connecting to Supabase Realtime.
#[derive(Clone)]
pub struct RealtimeConfig {
    /// Supabase project reference (e.g., "ojmqzagktzkualzgpcbq").
    pub project_ref: String,
    /// Supabase anon key (publishable).
    pub api_key: String,
    /// Optional access token (JWT) for authenticated connections.
    pub access_token: Option<String>,
    /// Heartbeat interval in seconds (default: 25).
    pub heartbeat_interval_secs: u64,
    /// Reconnect base delay in seconds.
    pub reconnect_delay_secs: u64,
    /// Maximum reconnect delay in seconds.
    pub max_reconnect_delay_secs: u64,
}

impl std::fmt::Debug for RealtimeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RealtimeConfig")
            .field("project_ref", &self.project_ref)
            .field("api_key", &"[REDACTED]")
            .field(
                "access_token",
                &self.access_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field("heartbeat_interval_secs", &self.heartbeat_interval_secs)
            .field("reconnect_delay_secs", &self.reconnect_delay_secs)
            .field("max_reconnect_delay_secs", &self.max_reconnect_delay_secs)
            .finish()
    }
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            project_ref: String::new(),
            api_key: String::new(),
            access_token: None,
            heartbeat_interval_secs: 25,
            reconnect_delay_secs: 1,
            max_reconnect_delay_secs: 30,
        }
    }
}

impl RealtimeConfig {
    /// Build the WebSocket URL for Supabase Realtime.
    pub(crate) fn ws_url(&self) -> String {
        format!(
            "wss://{}.supabase.co/realtime/v1/websocket?apikey={}&vsn=1.0.0",
            self.project_ref, self.api_key
        )
    }
}

// ---------------------------------------------------------------------------
// Phoenix Protocol Types
// ---------------------------------------------------------------------------

/// A Phoenix protocol message envelope (v1 JSON format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoenixMessage {
    pub topic: String,
    pub event: String,
    pub payload: serde_json::Value,
    #[serde(rename = "ref")]
    pub msg_ref: Option<String>,
}

// ---------------------------------------------------------------------------
// Channel Configuration
// ---------------------------------------------------------------------------

/// Configuration for a Supabase Realtime channel.
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub broadcast: BroadcastConfig,
    pub presence: PresenceConfig,
}

/// Broadcast configuration for a channel.
#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    /// Whether to receive your own broadcasts (Supabase "self" key).
    pub self_send: bool,
    /// Whether broadcasts are acknowledged by the server.
    pub ack: bool,
}

/// Presence configuration for a channel.
#[derive(Debug, Clone)]
pub struct PresenceConfig {
    /// The key used to identify this client in presence state.
    pub key: String,
}

impl ChannelConfig {
    /// Serialize to the JSON payload expected by Supabase phx_join.
    pub(crate) fn to_join_payload(&self) -> serde_json::Value {
        serde_json::json!({
            "config": {
                "broadcast": {
                    "self": self.broadcast.self_send,
                    "ack": self.broadcast.ack
                },
                "presence": {
                    "key": self.presence.key
                }
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Events & Commands
// ---------------------------------------------------------------------------

/// Events emitted by the realtime client.
#[derive(Debug, Clone)]
pub enum RealtimeEvent {
    /// WebSocket connection established.
    Connected,
    /// WebSocket connection lost.
    Disconnected,
    /// Successfully joined a channel.
    ChannelJoined { topic: String },
    /// Channel closed or errored.
    ChannelError { topic: String, message: String },
    /// A broadcast event received on a channel.
    Broadcast {
        topic: String,
        event: String,
        payload: serde_json::Value,
    },
    /// Full presence state snapshot (received after joining).
    PresenceState {
        topic: String,
        state: HashMap<String, Vec<serde_json::Value>>,
    },
    /// Incremental presence changes.
    PresenceDiff {
        topic: String,
        joins: HashMap<String, Vec<serde_json::Value>>,
        leaves: HashMap<String, Vec<serde_json::Value>>,
    },
    /// Error.
    Error(String),
}

/// Commands sent to the realtime client from the application layer.
#[derive(Debug)]
pub(crate) enum RealtimeCommand {
    JoinChannel {
        topic: String,
        config: ChannelConfig,
    },
    LeaveChannel {
        topic: String,
    },
    Broadcast {
        topic: String,
        event: String,
        payload: serde_json::Value,
    },
    PresenceTrack {
        topic: String,
        payload: serde_json::Value,
    },
    PresenceUntrack {
        topic: String,
    },
    Disconnect,
}
