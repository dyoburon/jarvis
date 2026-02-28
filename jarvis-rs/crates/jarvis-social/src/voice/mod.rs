//! Voice chat room management and signaling.
//!
//! Manages voice rooms, participant state (muted, deafened, speaking),
//! and relays WebRTC signaling messages through the presence WebSocket.
//! Actual media transport is peer-to-peer via WebRTC — this module only
//! handles the coordination layer.

mod manager;
mod types;

pub use manager::VoiceManager;
pub use types::{VoiceConfig, VoiceEvent, VoiceParticipant, VoiceRoom, VoiceState};
