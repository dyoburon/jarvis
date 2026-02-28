//! Relay-level wire protocol. Only the first message is parsed; everything after
//! is forwarded as opaque text frames.

use serde::{Deserialize, Serialize};

/// First message a client sends to identify itself.
#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum RelayHello {
    #[serde(rename = "desktop_hello")]
    DesktopHello { session_id: String },

    #[serde(rename = "mobile_hello")]
    MobileHello { session_id: String },
}

/// Messages the relay sends back to clients.
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum RelayResponse {
    #[serde(rename = "session_ready")]
    SessionReady { session_id: String },

    #[serde(rename = "peer_connected")]
    PeerConnected,

    #[serde(rename = "peer_disconnected")]
    PeerDisconnected,

    #[serde(rename = "error")]
    Error { message: String },
}
