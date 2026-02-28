//! Public handle for interacting with the Supabase Realtime connection.

use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use super::connection::connection_loop;
use super::types::{ChannelConfig, RealtimeCommand, RealtimeConfig, RealtimeEvent};

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Handle for interacting with the Supabase Realtime connection.
///
/// All methods are non-blocking and send commands to the background
/// connection task.
pub struct RealtimeClient {
    command_tx: mpsc::Sender<RealtimeCommand>,
    connected: Arc<RwLock<bool>>,
}

impl RealtimeClient {
    /// Create a new client and start the background connection.
    /// Returns `(client, event_receiver)`.
    pub fn connect(config: RealtimeConfig) -> (Self, mpsc::Receiver<RealtimeEvent>) {
        let (event_tx, event_rx) = mpsc::channel(256);
        let (command_tx, command_rx) = mpsc::channel(64);
        let connected = Arc::new(RwLock::new(false));

        let client = Self {
            command_tx,
            connected: Arc::clone(&connected),
        };

        tokio::spawn(connection_loop(config, connected, event_tx, command_rx));

        (client, event_rx)
    }

    /// Clone the command sender to create a lightweight handle
    /// that can send commands to the same connection.
    pub fn clone_sender(&self) -> Self {
        Self {
            command_tx: self.command_tx.clone(),
            connected: Arc::clone(&self.connected),
        }
    }

    /// Join a Supabase Realtime channel.
    pub async fn join_channel(&self, topic: &str, config: ChannelConfig) {
        let _ = self
            .command_tx
            .send(RealtimeCommand::JoinChannel {
                topic: topic.to_string(),
                config,
            })
            .await;
    }

    /// Leave a channel.
    pub async fn leave_channel(&self, topic: &str) {
        let _ = self
            .command_tx
            .send(RealtimeCommand::LeaveChannel {
                topic: topic.to_string(),
            })
            .await;
    }

    /// Send a broadcast event on a channel.
    pub async fn broadcast(&self, topic: &str, event: &str, payload: serde_json::Value) {
        let _ = self
            .command_tx
            .send(RealtimeCommand::Broadcast {
                topic: topic.to_string(),
                event: event.to_string(),
                payload,
            })
            .await;
    }

    /// Track presence on a channel.
    pub async fn presence_track(&self, topic: &str, payload: serde_json::Value) {
        let _ = self
            .command_tx
            .send(RealtimeCommand::PresenceTrack {
                topic: topic.to_string(),
                payload,
            })
            .await;
    }

    /// Untrack presence on a channel.
    pub async fn presence_untrack(&self, topic: &str) {
        let _ = self
            .command_tx
            .send(RealtimeCommand::PresenceUntrack {
                topic: topic.to_string(),
            })
            .await;
    }

    /// Check if connected.
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Disconnect from the server.
    pub async fn disconnect(&self) {
        let _ = self.command_tx.send(RealtimeCommand::Disconnect).await;
    }
}
