//! Protocol types for the Jarvis presence/social system.
//!
//! These types define the application-level payloads that ride inside
//! Supabase Realtime broadcast messages. The transport envelope (Phoenix
//! Channels protocol) is handled by `realtime.rs`.
//!
//! Voice, screen share, and pair programming types are gated behind
//! the `experimental-collab` feature flag.

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Broadcast event names
// ---------------------------------------------------------------------------

/// Event names used in Supabase Realtime broadcasts.
pub mod events {
    pub const ACTIVITY_UPDATE: &str = "activity_update";
    pub const GAME_INVITE: &str = "game_invite";
    pub const POKE: &str = "poke";
    pub const CHAT_MESSAGE: &str = "chat_message";
}

// ---------------------------------------------------------------------------
// Broadcast payloads
// ---------------------------------------------------------------------------

/// Payload for activity update broadcasts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityUpdatePayload {
    pub user_id: String,
    pub display_name: String,
    pub status: UserStatus,
    pub activity: Option<String>,
}

/// Payload for game invite broadcasts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameInvitePayload {
    pub user_id: String,
    pub display_name: String,
    pub game: String,
    pub code: Option<String>,
}

/// Payload for poke broadcasts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PokePayload {
    pub user_id: String,
    pub display_name: String,
    pub target_user_id: String,
}

/// Payload for chat message broadcasts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessagePayload {
    pub user_id: String,
    pub display_name: String,
    pub channel: String,
    pub content: String,
    pub timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<String>,
}

/// Payload tracked in Supabase Presence for each user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresencePayload {
    pub user_id: String,
    pub display_name: String,
    pub status: UserStatus,
    pub activity: Option<String>,
    pub online_at: String,
}

// ---------------------------------------------------------------------------
// Shared types
// ---------------------------------------------------------------------------

/// User presence status.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UserStatus {
    #[default]
    Online,
    Idle,
    InGame,
    InSkill,
    DoNotDisturb,
    Away,
}

/// Information about an online user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnlineUser {
    pub user_id: String,
    pub display_name: String,
    pub status: UserStatus,
    pub activity: Option<String>,
}

// ---------------------------------------------------------------------------
// WebRTC signaling types (experimental-collab only)
// ---------------------------------------------------------------------------

/// WebRTC signaling messages for voice chat.
#[cfg(feature = "experimental-collab")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum VoiceSignal {
    /// SDP offer to establish a peer connection.
    Offer { sdp: String },
    /// SDP answer in response to an offer.
    Answer { sdp: String },
    /// ICE candidate for NAT traversal.
    IceCandidate {
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u32>,
    },
}

/// WebRTC signaling messages for screen sharing.
#[cfg(feature = "experimental-collab")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ScreenShareSignal {
    /// SDP offer from the host to a viewer.
    Offer { sdp: String },
    /// SDP answer from a viewer to the host.
    Answer { sdp: String },
    /// ICE candidate.
    IceCandidate {
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u32>,
    },
}
