//! Internal types and constants for the app state module.

use std::time::Duration;

use jarvis_social::UserStatus;

/// Events received from the async AI task.
pub(super) enum AssistantEvent {
    /// A streaming text chunk arrived.
    StreamChunk(String),
    /// The full response is complete.
    Done,
    /// An error occurred.
    Error(String),
}

/// Commands sent from the sync main thread to the async presence task.
pub(super) enum PresenceCommand {
    /// Send a poke to a specific user.
    Poke { target_user_id: String },
    /// Update our activity status.
    UpdateActivity {
        status: UserStatus,
        activity: Option<String>,
    },
}

/// How often to poll for events (approx 120 Hz).
pub(super) const POLL_INTERVAL: Duration = Duration::from_millis(8);
