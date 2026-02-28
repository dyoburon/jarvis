//! Thin Supabase Realtime client over Phoenix Channels v1 protocol.
//!
//! Provides a generic, reusable WebSocket client for Supabase Realtime
//! using `tokio-tungstenite`. Handles heartbeats, channel join/leave,
//! broadcast, presence tracking, and auto-reconnect with backoff.

mod client;
mod connection;
mod handler;
mod types;

pub use client::RealtimeClient;
pub use types::{
    BroadcastConfig, ChannelConfig, PhoenixMessage, PresenceConfig, RealtimeConfig, RealtimeEvent,
};
