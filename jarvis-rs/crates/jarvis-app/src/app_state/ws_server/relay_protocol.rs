//! Relay-level envelope messages — separate from the inner PTY protocol.

use serde::{Deserialize, Serialize};

/// First message desktop sends to identify itself.
#[derive(Debug, Serialize)]
pub struct DesktopHello {
    #[serde(rename = "type")]
    pub msg_type: &'static str,
    pub session_id: String,
}

impl DesktopHello {
    pub fn new(session_id: String) -> Self {
        Self {
            msg_type: "desktop_hello",
            session_id,
        }
    }
}

/// Messages the relay sends back to us.
#[derive(Debug, Deserialize)]
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

/// Envelope for messages forwarded through the relay.
/// For now, messages are sent as plain JSON (no E2E encryption yet).
/// Phase 3 will add key_exchange and encrypted variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RelayEnvelope {
    /// Key exchange: send our DH public key to the peer.
    #[serde(rename = "key_exchange")]
    KeyExchange { dh_pubkey: String },

    /// Encrypted payload — inner PTY protocol message.
    #[serde(rename = "encrypted")]
    Encrypted { iv: String, ct: String },

    /// Plaintext PTY protocol message (used before encryption is set up).
    #[serde(rename = "plaintext")]
    Plaintext { payload: String },
}
