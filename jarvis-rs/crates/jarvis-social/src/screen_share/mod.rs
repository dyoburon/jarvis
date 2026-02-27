//! Screen sharing session management.
//!
//! Tracks active screen share sessions, viewer lists, and quality
//! settings. Like voice, the actual media transport is WebRTC P2P —
//! this module handles coordination and signaling relay.

mod manager;
mod types;

pub use manager::ScreenShareManager;
pub use types::{ScreenShareConfig, ScreenShareEvent, ScreenShareSession, ShareQuality};
