//! Thin Supabase Realtime client over Phoenix Channels v1 protocol.
//!
//! Provides a generic, reusable WebSocket client for Supabase Realtime
//! using `tokio-tungstenite`. Handles heartbeats, channel join/leave,
//! broadcast, presence tracking, and auto-reconnect with backoff.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{debug, error, info, warn};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for connecting to Supabase Realtime.
#[derive(Debug, Clone)]
pub struct RealtimeConfig {
    /// Supabase project reference (e.g., "ojmqzagktzkualzgpcbq").
    pub project_ref: String,
    /// Supabase anon key (publishable).
    pub api_key: String,
    /// Optional access token (JWT) for authenticated connections.
    pub access_token: Option<String>,
    /// Heartbeat interval in seconds (default: 25).
    pub heartbeat_interval_secs: u64,
    /// Reconnect base delay in seconds.
    pub reconnect_delay_secs: u64,
    /// Maximum reconnect delay in seconds.
    pub max_reconnect_delay_secs: u64,
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            project_ref: String::new(),
            api_key: String::new(),
            access_token: None,
            heartbeat_interval_secs: 25,
            reconnect_delay_secs: 1,
            max_reconnect_delay_secs: 30,
        }
    }
}

impl RealtimeConfig {
    /// Build the WebSocket URL for Supabase Realtime.
    fn ws_url(&self) -> String {
        format!(
            "wss://{}.supabase.co/realtime/v1/websocket?apikey={}&vsn=1.0.0",
            self.project_ref, self.api_key
        )
    }
}

// ---------------------------------------------------------------------------
// Phoenix Protocol Types
// ---------------------------------------------------------------------------

/// A Phoenix protocol message envelope (v1 JSON format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhoenixMessage {
    pub topic: String,
    pub event: String,
    pub payload: serde_json::Value,
    #[serde(rename = "ref")]
    pub msg_ref: Option<String>,
}

// ---------------------------------------------------------------------------
// Channel Configuration
// ---------------------------------------------------------------------------

/// Configuration for a Supabase Realtime channel.
#[derive(Debug, Clone)]
pub struct ChannelConfig {
    pub broadcast: BroadcastConfig,
    pub presence: PresenceConfig,
}

/// Broadcast configuration for a channel.
#[derive(Debug, Clone)]
pub struct BroadcastConfig {
    /// Whether to receive your own broadcasts (Supabase "self" key).
    pub self_send: bool,
    /// Whether broadcasts are acknowledged by the server.
    pub ack: bool,
}

/// Presence configuration for a channel.
#[derive(Debug, Clone)]
pub struct PresenceConfig {
    /// The key used to identify this client in presence state.
    pub key: String,
}

impl ChannelConfig {
    /// Serialize to the JSON payload expected by Supabase phx_join.
    fn to_join_payload(&self) -> serde_json::Value {
        serde_json::json!({
            "config": {
                "broadcast": {
                    "self": self.broadcast.self_send,
                    "ack": self.broadcast.ack
                },
                "presence": {
                    "key": self.presence.key
                }
            }
        })
    }
}

// ---------------------------------------------------------------------------
// Events & Commands
// ---------------------------------------------------------------------------

/// Events emitted by the realtime client.
#[derive(Debug, Clone)]
pub enum RealtimeEvent {
    /// WebSocket connection established.
    Connected,
    /// WebSocket connection lost.
    Disconnected,
    /// Successfully joined a channel.
    ChannelJoined { topic: String },
    /// Channel closed or errored.
    ChannelError { topic: String, message: String },
    /// A broadcast event received on a channel.
    Broadcast {
        topic: String,
        event: String,
        payload: serde_json::Value,
    },
    /// Full presence state snapshot (received after joining).
    PresenceState {
        topic: String,
        state: HashMap<String, Vec<serde_json::Value>>,
    },
    /// Incremental presence changes.
    PresenceDiff {
        topic: String,
        joins: HashMap<String, Vec<serde_json::Value>>,
        leaves: HashMap<String, Vec<serde_json::Value>>,
    },
    /// Error.
    Error(String),
}

/// Commands sent to the realtime client from the application layer.
#[derive(Debug)]
enum RealtimeCommand {
    JoinChannel {
        topic: String,
        config: ChannelConfig,
    },
    LeaveChannel {
        topic: String,
    },
    Broadcast {
        topic: String,
        event: String,
        payload: serde_json::Value,
    },
    PresenceTrack {
        topic: String,
        payload: serde_json::Value,
    },
    PresenceUntrack {
        topic: String,
    },
    Disconnect,
}

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
    pub async fn broadcast(
        &self,
        topic: &str,
        event: &str,
        payload: serde_json::Value,
    ) {
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

// ---------------------------------------------------------------------------
// Connection Loop
// ---------------------------------------------------------------------------

/// Monotonically increasing ref counter for Phoenix messages.
static REF_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_ref() -> String {
    REF_COUNTER.fetch_add(1, Ordering::Relaxed).to_string()
}

/// State for channels that should be (re)joined on reconnect.
#[derive(Clone)]
struct PendingChannel {
    config: ChannelConfig,
    presence_payload: Option<serde_json::Value>,
}

/// Background task managing the WebSocket connection with auto-reconnect.
async fn connection_loop(
    config: RealtimeConfig,
    connected: Arc<RwLock<bool>>,
    event_tx: mpsc::Sender<RealtimeEvent>,
    command_rx: mpsc::Receiver<RealtimeCommand>,
) {
    let command_rx = Arc::new(Mutex::new(command_rx));
    // Channels to rejoin on reconnect.
    let joined_channels: Arc<RwLock<HashMap<String, PendingChannel>>> =
        Arc::new(RwLock::new(HashMap::new()));
    let mut reconnect_delay = config.reconnect_delay_secs;

    loop {
        let url = config.ws_url();
        info!(url = %url, "Connecting to Supabase Realtime");

        match tokio_tungstenite::connect_async(&url).await {
            Ok((ws_stream, _)) => {
                reconnect_delay = config.reconnect_delay_secs;
                *connected.write().await = true;
                let _ = event_tx.send(RealtimeEvent::Connected).await;

                let (ws_write, ws_read) = ws_stream.split();
                let ws_write = Arc::new(Mutex::new(ws_write));

                // If we have an access token, we could set it here.
                // Supabase also accepts it in the phx_join payload.

                // Rejoin previously-joined channels.
                {
                    let channels = joined_channels.read().await;
                    for (topic, pending) in channels.iter() {
                        let join_payload = pending.config.to_join_payload();
                        let msg = PhoenixMessage {
                            topic: format!("realtime:{topic}"),
                            event: "phx_join".to_string(),
                            payload: join_payload,
                            msg_ref: Some(next_ref()),
                        };
                        if let Ok(json) = serde_json::to_string(&msg) {
                            let mut writer = ws_write.lock().await;
                            let _ = writer.send(WsMessage::Text(json.into())).await;
                        }
                    }
                }

                // Spawn heartbeat task.
                let heartbeat_write = Arc::clone(&ws_write);
                let heartbeat_interval = config.heartbeat_interval_secs;
                let heartbeat_handle = tokio::spawn(async move {
                    let mut interval =
                        tokio::time::interval(Duration::from_secs(heartbeat_interval));
                    loop {
                        interval.tick().await;
                        let msg = PhoenixMessage {
                            topic: "phoenix".to_string(),
                            event: "heartbeat".to_string(),
                            payload: serde_json::json!({}),
                            msg_ref: Some(next_ref()),
                        };
                        if let Ok(json) = serde_json::to_string(&msg) {
                            let mut writer = heartbeat_write.lock().await;
                            if writer.send(WsMessage::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                });

                // Spawn command forwarder.
                let cmd_write = Arc::clone(&ws_write);
                let cmd_rx = Arc::clone(&command_rx);
                let cmd_channels = Arc::clone(&joined_channels);
                let cmd_event_tx = event_tx.clone();
                let cmd_handle = tokio::spawn(async move {
                    let mut rx = cmd_rx.lock().await;
                    while let Some(cmd) = rx.recv().await {
                        match cmd {
                            RealtimeCommand::JoinChannel { topic, config } => {
                                let join_payload = config.to_join_payload();
                                let msg = PhoenixMessage {
                                    topic: format!("realtime:{topic}"),
                                    event: "phx_join".to_string(),
                                    payload: join_payload,
                                    msg_ref: Some(next_ref()),
                                };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let mut writer = cmd_write.lock().await;
                                    let _ = writer.send(WsMessage::Text(json.into())).await;
                                }
                                cmd_channels.write().await.insert(
                                    topic,
                                    PendingChannel {
                                        config,
                                        presence_payload: None,
                                    },
                                );
                            }
                            RealtimeCommand::LeaveChannel { topic } => {
                                let msg = PhoenixMessage {
                                    topic: format!("realtime:{topic}"),
                                    event: "phx_leave".to_string(),
                                    payload: serde_json::json!({}),
                                    msg_ref: Some(next_ref()),
                                };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let mut writer = cmd_write.lock().await;
                                    let _ = writer.send(WsMessage::Text(json.into())).await;
                                }
                                cmd_channels.write().await.remove(&topic);
                            }
                            RealtimeCommand::Broadcast {
                                topic,
                                event,
                                payload,
                            } => {
                                let msg = PhoenixMessage {
                                    topic: format!("realtime:{topic}"),
                                    event: "broadcast".to_string(),
                                    payload: serde_json::json!({
                                        "type": "broadcast",
                                        "event": event,
                                        "payload": payload
                                    }),
                                    msg_ref: Some(next_ref()),
                                };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let mut writer = cmd_write.lock().await;
                                    let _ = writer.send(WsMessage::Text(json.into())).await;
                                }
                            }
                            RealtimeCommand::PresenceTrack { topic, payload } => {
                                let msg = PhoenixMessage {
                                    topic: format!("realtime:{topic}"),
                                    event: "presence".to_string(),
                                    payload: serde_json::json!({
                                        "type": "presence",
                                        "event": "track",
                                        "payload": payload
                                    }),
                                    msg_ref: Some(next_ref()),
                                };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let mut writer = cmd_write.lock().await;
                                    let _ = writer.send(WsMessage::Text(json.into())).await;
                                }
                                // Store for re-tracking on reconnect.
                                if let Some(ch) = cmd_channels.write().await.get_mut(&topic) {
                                    ch.presence_payload = Some(payload);
                                }
                            }
                            RealtimeCommand::PresenceUntrack { topic } => {
                                let msg = PhoenixMessage {
                                    topic: format!("realtime:{topic}"),
                                    event: "presence".to_string(),
                                    payload: serde_json::json!({
                                        "type": "presence",
                                        "event": "untrack"
                                    }),
                                    msg_ref: Some(next_ref()),
                                };
                                if let Ok(json) = serde_json::to_string(&msg) {
                                    let mut writer = cmd_write.lock().await;
                                    let _ = writer.send(WsMessage::Text(json.into())).await;
                                }
                                if let Some(ch) = cmd_channels.write().await.get_mut(&topic) {
                                    ch.presence_payload = None;
                                }
                            }
                            RealtimeCommand::Disconnect => {
                                // Send phx_leave for all channels, then close.
                                let channels = cmd_channels.read().await;
                                for topic in channels.keys() {
                                    let msg = PhoenixMessage {
                                        topic: format!("realtime:{topic}"),
                                        event: "phx_leave".to_string(),
                                        payload: serde_json::json!({}),
                                        msg_ref: Some(next_ref()),
                                    };
                                    if let Ok(json) = serde_json::to_string(&msg) {
                                        let mut writer = cmd_write.lock().await;
                                        let _ =
                                            writer.send(WsMessage::Text(json.into())).await;
                                    }
                                }
                                drop(channels);
                                let mut writer = cmd_write.lock().await;
                                let _ = writer.send(WsMessage::Close(None)).await;
                                let _ =
                                    cmd_event_tx.send(RealtimeEvent::Disconnected).await;
                                return; // Exit the command forwarder
                            }
                        }
                    }
                });

                // Process incoming messages.
                let mut read_stream = ws_read;
                while let Some(msg_result) = read_stream.next().await {
                    match msg_result {
                        Ok(WsMessage::Text(text)) => {
                            if let Ok(phoenix_msg) =
                                serde_json::from_str::<PhoenixMessage>(&text)
                            {
                                handle_phoenix_message(
                                    &phoenix_msg,
                                    &joined_channels,
                                    &event_tx,
                                )
                                .await;
                            } else {
                                debug!(text = %text, "Unrecognized message from Supabase");
                            }
                        }
                        Ok(WsMessage::Close(_)) => {
                            info!("Supabase Realtime closed connection");
                            break;
                        }
                        Err(e) => {
                            warn!(error = %e, "WebSocket error");
                            break;
                        }
                        _ => {}
                    }
                }

                // Cleanup.
                heartbeat_handle.abort();
                cmd_handle.abort();
                *connected.write().await = false;
                let _ = event_tx.send(RealtimeEvent::Disconnected).await;
            }
            Err(e) => {
                error!(error = %e, "Failed to connect to Supabase Realtime");
                let _ = event_tx
                    .send(RealtimeEvent::Error(format!("Connection failed: {e}")))
                    .await;
            }
        }

        // Exponential backoff reconnect.
        info!(
            delay = reconnect_delay,
            "Reconnecting in {} seconds", reconnect_delay
        );
        tokio::time::sleep(Duration::from_secs(reconnect_delay)).await;
        reconnect_delay = (reconnect_delay * 2).min(config.max_reconnect_delay_secs);
    }
}

// ---------------------------------------------------------------------------
// Message Handler
// ---------------------------------------------------------------------------

/// Extract the short topic name from a Phoenix topic (strip "realtime:" prefix).
fn strip_topic_prefix(topic: &str) -> &str {
    topic.strip_prefix("realtime:").unwrap_or(topic)
}

/// Parse a Phoenix presence map into `HashMap<key, Vec<meta>>`.
///
/// Supabase sends presence as `{ "key": { "metas": [{ ... }] } }`.
fn parse_presence_map(
    value: &serde_json::Value,
) -> HashMap<String, Vec<serde_json::Value>> {
    let mut result = HashMap::new();
    if let Some(obj) = value.as_object() {
        for (key, val) in obj {
            if let Some(metas) = val.get("metas").and_then(|m| m.as_array()) {
                result.insert(key.clone(), metas.clone());
            }
        }
    }
    result
}

/// Handle a single incoming Phoenix message.
async fn handle_phoenix_message(
    msg: &PhoenixMessage,
    _joined_channels: &Arc<RwLock<HashMap<String, PendingChannel>>>,
    event_tx: &mpsc::Sender<RealtimeEvent>,
) {
    let topic = strip_topic_prefix(&msg.topic);

    match msg.event.as_str() {
        "phx_reply" => {
            // Join acknowledgment or broadcast ack.
            if let Some(status) = msg.payload.get("status").and_then(|s| s.as_str()) {
                if status == "ok" {
                    debug!(topic = %topic, "Channel reply: ok");
                    let _ = event_tx
                        .send(RealtimeEvent::ChannelJoined {
                            topic: topic.to_string(),
                        })
                        .await;
                } else {
                    let message = msg
                        .payload
                        .get("response")
                        .and_then(|r| r.get("reason"))
                        .and_then(|r| r.as_str())
                        .unwrap_or("unknown error")
                        .to_string();
                    warn!(topic = %topic, status = %status, "Channel reply error");
                    let _ = event_tx
                        .send(RealtimeEvent::ChannelError {
                            topic: topic.to_string(),
                            message,
                        })
                        .await;
                }
            }
        }
        "phx_error" => {
            warn!(topic = %topic, "Channel error");
            let _ = event_tx
                .send(RealtimeEvent::ChannelError {
                    topic: topic.to_string(),
                    message: "Channel error".to_string(),
                })
                .await;
        }
        "phx_close" => {
            info!(topic = %topic, "Channel closed");
            let _ = event_tx
                .send(RealtimeEvent::ChannelError {
                    topic: topic.to_string(),
                    message: "Channel closed".to_string(),
                })
                .await;
        }
        "broadcast" => {
            // Extract the inner event name and payload.
            let inner_event = msg
                .payload
                .get("event")
                .and_then(|e| e.as_str())
                .unwrap_or("unknown")
                .to_string();
            let inner_payload = msg
                .payload
                .get("payload")
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            debug!(topic = %topic, event = %inner_event, "Broadcast received");
            let _ = event_tx
                .send(RealtimeEvent::Broadcast {
                    topic: topic.to_string(),
                    event: inner_event,
                    payload: inner_payload,
                })
                .await;
        }
        "presence_state" => {
            let state = parse_presence_map(&msg.payload);
            debug!(topic = %topic, users = state.len(), "Presence state received");
            let _ = event_tx
                .send(RealtimeEvent::PresenceState {
                    topic: topic.to_string(),
                    state,
                })
                .await;
        }
        "presence_diff" => {
            let joins = msg
                .payload
                .get("joins")
                .map(parse_presence_map)
                .unwrap_or_default();
            let leaves = msg
                .payload
                .get("leaves")
                .map(parse_presence_map)
                .unwrap_or_default();
            debug!(
                topic = %topic,
                joins = joins.len(),
                leaves = leaves.len(),
                "Presence diff received"
            );
            let _ = event_tx
                .send(RealtimeEvent::PresenceDiff {
                    topic: topic.to_string(),
                    joins,
                    leaves,
                })
                .await;
        }
        _ => {
            debug!(
                topic = %topic,
                event = %msg.event,
                "Unhandled Phoenix event"
            );
        }
    }
}
