//! Presence client backed by Supabase Realtime.
//!
//! Connects to Supabase Realtime via Phoenix Channels, tracks presence,
//! broadcasts activity updates, and receives events from other users.
//! The transport layer is handled by `realtime::RealtimeClient`.

mod client;
mod event_translator;
mod helpers;
mod types;

pub use client::PresenceClient;
pub use types::{PresenceConfig, PresenceEvent};
