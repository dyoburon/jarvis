//! Types, configuration, and events for pair programming sessions.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Role a participant has in a pair session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairRole {
    /// Can type into the shared terminal.
    Driver,
    /// View-only; can request control.
    Navigator,
}

/// A participant in a pair programming session.
#[derive(Debug, Clone)]
pub struct PairParticipant {
    pub user_id: String,
    pub display_name: String,
    pub role: PairRole,
    /// Cursor row in the shared terminal (for showing remote cursors).
    pub cursor_row: u16,
    /// Cursor col.
    pub cursor_col: u16,
}

/// An active pair programming session.
#[derive(Debug, Clone)]
pub struct PairSession {
    pub session_id: String,
    /// The user hosting the terminal.
    pub host_user_id: String,
    pub host_display_name: String,
    /// Terminal dimensions.
    pub cols: u16,
    pub rows: u16,
    /// All participants (including the host).
    pub participants: HashMap<String, PairParticipant>,
    /// Who currently has the driver role.
    pub driver_user_id: String,
    /// Whether guests can request driver role.
    pub allow_takeover: bool,
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Events emitted by the pair programming system.
#[derive(Debug, Clone)]
pub enum PairEvent {
    /// A new session was created.
    SessionCreated {
        session_id: String,
        host_user_id: String,
        host_display_name: String,
        cols: u16,
        rows: u16,
    },
    /// A session ended.
    SessionEnded {
        session_id: String,
    },
    /// A user joined the session.
    UserJoined {
        session_id: String,
        user_id: String,
        display_name: String,
        role: PairRole,
    },
    /// A user left the session.
    UserLeft {
        session_id: String,
        user_id: String,
    },
    /// Driver role changed hands.
    DriverChanged {
        session_id: String,
        new_driver: String,
        old_driver: String,
    },
    /// Terminal output from the host's PTY — relay to guests.
    TerminalOutput {
        session_id: String,
        data: Vec<u8>,
    },
    /// Keystroke input from the current driver — relay to host PTY.
    TerminalInput {
        session_id: String,
        from_user: String,
        data: Vec<u8>,
    },
    /// Remote cursor position update.
    CursorMoved {
        session_id: String,
        user_id: String,
        row: u16,
        col: u16,
    },
    /// Terminal was resized by the host.
    Resized {
        session_id: String,
        cols: u16,
        rows: u16,
    },
    Error(String),
}

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for pair programming.
#[derive(Debug, Clone)]
pub struct PairConfig {
    pub enabled: bool,
    /// Maximum participants per session (including host).
    pub max_participants: usize,
    /// Whether guests can request driver role by default.
    pub allow_takeover: bool,
}

impl Default for PairConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            max_participants: 4,
            allow_takeover: true,
        }
    }
}

// ---------------------------------------------------------------------------
// State
// ---------------------------------------------------------------------------

/// Combined state for pair sessions and user-to-session mappings.
///
/// Both maps live under a single `RwLock` so they are always mutated
/// atomically — eliminating the race condition where the two maps
/// could get out of sync under concurrent access.
pub struct PairState {
    /// Active sessions keyed by session_id.
    pub sessions: HashMap<String, PairSession>,
    /// user_id → session_id (each user can only be in one session).
    pub user_sessions: HashMap<String, String>,
}
