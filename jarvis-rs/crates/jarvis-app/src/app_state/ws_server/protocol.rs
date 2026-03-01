//! Wire protocol for the mobile ↔ desktop PTY bridge.
//!
//! These messages travel inside `RelayEnvelope::Plaintext` (or `Encrypted`)
//! through the relay. The relay never inspects them.

use serde::{Deserialize, Serialize};

/// Pane metadata sent to mobile clients.
#[derive(Debug, Clone, Serialize)]
pub struct PaneInfo {
    pub id: u32,
    pub kind: String,
    pub title: String,
}

/// Messages sent from desktop to mobile.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "pty_output")]
    PtyOutput { pane_id: u32, data: String },

    #[serde(rename = "pty_exit")]
    PtyExit { pane_id: u32, code: u32 },

    #[serde(rename = "pane_list")]
    PaneList { panes: Vec<PaneInfo>, focused_id: u32 },
}

/// Messages received from mobile.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "pty_input")]
    PtyInput { pane_id: u32, data: String },

    #[serde(rename = "pty_resize")]
    PtyResize { pane_id: u32, cols: u16, rows: u16 },

    #[serde(rename = "ping")]
    Ping,
}
