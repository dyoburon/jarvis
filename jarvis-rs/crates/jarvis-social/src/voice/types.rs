//! Types, configuration, and events for voice chat rooms.

use std::collections::{HashMap, HashSet};

use crate::protocol::VoiceSignal;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Per-participant state inside a voice room.
#[derive(Debug, Clone)]
pub struct VoiceParticipant {
    pub user_id: String,
    pub display_name: String,
    pub muted: bool,
    pub deafened: bool,
    pub speaking: bool,
}

/// A voice room that users can join for real-time audio.
#[derive(Debug, Clone)]
pub struct VoiceRoom {
    pub room_id: String,
    pub name: String,
    pub created_by: String,
    pub participants: HashMap<String, VoiceParticipant>,
    pub max_participants: usize,
}

impl VoiceRoom {
    pub fn new(room_id: String, name: String, created_by: String) -> Self {
        Self {
            room_id,
            name,
            created_by,
            participants: HashMap::new(),
            max_participants: 8,
        }
    }

    pub fn is_full(&self) -> bool {
        self.participants.len() >= self.max_participants
    }

    pub fn participant_ids(&self) -> HashSet<String> {
        self.participants.keys().cloned().collect()
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Events emitted by the voice system for the UI.
#[derive(Debug, Clone)]
pub enum VoiceEvent {
    RoomCreated {
        room_id: String,
        name: String,
    },
    RoomClosed {
        room_id: String,
    },
    UserJoined {
        room_id: String,
        user_id: String,
        display_name: String,
    },
    UserLeft {
        room_id: String,
        user_id: String,
        display_name: String,
    },
    MuteChanged {
        room_id: String,
        user_id: String,
        muted: bool,
    },
    DeafenChanged {
        room_id: String,
        user_id: String,
        deafened: bool,
    },
    SpeakingChanged {
        room_id: String,
        user_id: String,
        speaking: bool,
    },
    /// WebRTC signaling message received — forward to the WebRTC layer.
    Signal {
        from_user: String,
        signal: VoiceSignal,
    },
    Error(String),
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for voice chat.
#[derive(Debug, Clone)]
pub struct VoiceConfig {
    /// Maximum participants per room.
    pub max_participants: usize,
    /// Whether voice is enabled.
    pub enabled: bool,
}

impl Default for VoiceConfig {
    fn default() -> Self {
        Self {
            max_participants: 8,
            enabled: false,
        }
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Combined voice state protected by a single lock to eliminate race conditions
/// between room membership and user-room mapping.
pub struct VoiceState {
    /// All active rooms keyed by room_id.
    pub rooms: HashMap<String, VoiceRoom>,
    /// Which room each user is in (user_id → room_id).
    pub user_rooms: HashMap<String, String>,
}
