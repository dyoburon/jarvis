//! Internal types and constants for the app state module.

use std::time::Duration;

use jarvis_terminal::pty::PtyManager;
use jarvis_terminal::VteHandler;

/// Per-pane state: terminal emulator and PTY process.
pub(super) struct PaneState {
    pub vte: VteHandler,
    pub pty: PtyManager,
}

/// Events received from the async AI task.
pub(super) enum AssistantEvent {
    /// A streaming text chunk arrived.
    StreamChunk(String),
    /// The full response is complete.
    Done,
    /// An error occurred.
    Error(String),
}

/// How often to poll PTY output (approx 120 Hz).
pub(super) const POLL_INTERVAL: Duration = Duration::from_millis(8);
