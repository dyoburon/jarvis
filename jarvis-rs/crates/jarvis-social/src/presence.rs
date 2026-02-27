//! Presence client backed by Supabase Realtime.
//!
//! Connects to Supabase Realtime via Phoenix Channels, tracks presence,
//! broadcasts activity updates, and receives events from other users.
//! The transport layer is handled by `realtime::RealtimeClient`.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tracing::{debug, warn};

use crate::identity::Identity;
use crate::protocol::{
    events, ActivityUpdatePayload, ChatMessagePayload, GameInvitePayload, OnlineUser,
    PokePayload, UserStatus,
};
use crate::realtime::{
    BroadcastConfig, ChannelConfig, PresenceConfig as RtPresenceConfig, RealtimeClient,
    RealtimeConfig, RealtimeEvent,
};

/// The Supabase Realtime channel name used for all social events.
const CHANNEL_NAME: &str = "jarvis-presence";

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the presence client.
#[derive(Debug, Clone)]
pub struct PresenceConfig {
    /// Supabase project reference (e.g., "ojmqzagktzkualzgpcbq").
    pub project_ref: String,
    /// Supabase anon key (publishable).
    pub api_key: String,
    /// Optional JWT for authenticated connections.
    pub access_token: Option<String>,
    /// Heartbeat interval in seconds.
    pub heartbeat_interval: u64,
    /// Reconnect delay (base) in seconds.
    pub reconnect_delay: u64,
    /// Maximum reconnect delay in seconds.
    pub max_reconnect_delay: u64,
}

impl Default for PresenceConfig {
    fn default() -> Self {
        Self {
            project_ref: String::new(),
            api_key: String::new(),
            access_token: None,
            heartbeat_interval: 25,
            reconnect_delay: 1,
            max_reconnect_delay: 30,
        }
    }
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Events emitted by the presence system for the UI to consume.
#[derive(Debug, Clone)]
pub enum PresenceEvent {
    Connected { online_count: u32 },
    Disconnected,
    UserOnline(OnlineUser),
    UserOffline { user_id: String, display_name: String },
    ActivityChanged(OnlineUser),
    GameInvite { user_id: String, display_name: String, game: String, code: Option<String> },
    Poked { user_id: String, display_name: String },
    ChatMessage { user_id: String, display_name: String, channel: String, content: String },
    Error(String),
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

/// Presence client that maintains a connection to Supabase Realtime.
pub struct PresenceClient {
    config: PresenceConfig,
    identity: Identity,
    /// Current list of online users.
    online_users: Arc<RwLock<HashMap<String, OnlineUser>>>,
    /// Handle to the realtime client.
    realtime: Option<RealtimeClient>,
    /// Whether we're currently connected.
    connected: Arc<RwLock<bool>>,
}

impl PresenceClient {
    pub fn new(identity: Identity, config: PresenceConfig) -> Self {
        Self {
            config,
            identity,
            online_users: Arc::new(RwLock::new(HashMap::new())),
            realtime: None,
            connected: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the presence connection. Returns a receiver for presence events.
    /// The connection runs in a background task with auto-reconnect.
    pub fn start(&mut self) -> mpsc::Receiver<PresenceEvent> {
        let (event_tx, event_rx) = mpsc::channel(256);

        // Build the RealtimeConfig from our PresenceConfig.
        let rt_config = RealtimeConfig {
            project_ref: self.config.project_ref.clone(),
            api_key: self.config.api_key.clone(),
            access_token: self.config.access_token.clone(),
            heartbeat_interval_secs: self.config.heartbeat_interval,
            reconnect_delay_secs: self.config.reconnect_delay,
            max_reconnect_delay_secs: self.config.max_reconnect_delay,
        };

        let (client, rt_event_rx) = RealtimeClient::connect(rt_config);

        // Join the presence channel.
        let channel_config = ChannelConfig {
            broadcast: BroadcastConfig {
                self_send: false,
                ack: true,
            },
            presence: RtPresenceConfig {
                key: self.identity.user_id.clone(),
            },
        };

        let join_client = client.clone_sender();
        let identity = self.identity.clone();
        let online_users = Arc::clone(&self.online_users);
        let connected = Arc::clone(&self.connected);

        // Spawn the event translator task.
        tokio::spawn(async move {
            // Join the channel once connected.
            join_client
                .join_channel(CHANNEL_NAME, channel_config)
                .await;

            // Track our presence.
            let presence_payload = serde_json::json!({
                "user_id": identity.user_id,
                "display_name": identity.display_name,
                "status": "online",
                "activity": null,
                "online_at": chrono_now()
            });
            join_client
                .presence_track(CHANNEL_NAME, presence_payload)
                .await;

            // Translate RealtimeEvents into PresenceEvents.
            event_translator(
                rt_event_rx,
                event_tx,
                online_users,
                connected,
                &identity.user_id,
            )
            .await;
        });

        self.realtime = Some(client);
        event_rx
    }

    /// Update activity status.
    pub async fn update_activity(&self, status: UserStatus, activity: Option<String>) {
        if let Some(rt) = &self.realtime {
            // Broadcast the activity update.
            let payload = ActivityUpdatePayload {
                user_id: self.identity.user_id.clone(),
                display_name: self.identity.display_name.clone(),
                status,
                activity: activity.clone(),
            };
            if let Ok(value) = serde_json::to_value(&payload) {
                rt.broadcast(CHANNEL_NAME, events::ACTIVITY_UPDATE, value)
                    .await;
            }

            // Also update our presence state so new joiners see correct status.
            let presence_payload = serde_json::json!({
                "user_id": self.identity.user_id,
                "display_name": self.identity.display_name,
                "status": status,
                "activity": activity,
                "online_at": chrono_now()
            });
            rt.presence_track(CHANNEL_NAME, presence_payload).await;
        }
    }

    /// Send a game invite.
    pub async fn send_invite(&self, game: &str, code: Option<String>) {
        if let Some(rt) = &self.realtime {
            let payload = GameInvitePayload {
                user_id: self.identity.user_id.clone(),
                display_name: self.identity.display_name.clone(),
                game: game.to_string(),
                code,
            };
            if let Ok(value) = serde_json::to_value(&payload) {
                rt.broadcast(CHANNEL_NAME, events::GAME_INVITE, value)
                    .await;
            }
        }
    }

    /// Poke a user.
    pub async fn send_poke(&self, target_user_id: &str) {
        if let Some(rt) = &self.realtime {
            let payload = PokePayload {
                user_id: self.identity.user_id.clone(),
                display_name: self.identity.display_name.clone(),
                target_user_id: target_user_id.to_string(),
            };
            if let Ok(value) = serde_json::to_value(&payload) {
                rt.broadcast(CHANNEL_NAME, events::POKE, value).await;
            }
        }
    }

    /// Send a chat message to a channel.
    pub async fn send_chat(&self, channel: &str, content: &str, reply_to: Option<String>) {
        if let Some(rt) = &self.realtime {
            let payload = ChatMessagePayload {
                user_id: self.identity.user_id.clone(),
                display_name: self.identity.display_name.clone(),
                channel: channel.to_string(),
                content: content.to_string(),
                timestamp: chrono_now(),
                reply_to,
            };
            if let Ok(value) = serde_json::to_value(&payload) {
                rt.broadcast(CHANNEL_NAME, events::CHAT_MESSAGE, value)
                    .await;
            }
        }
    }

    /// Disconnect from the presence server.
    pub async fn disconnect(&self) {
        if let Some(rt) = &self.realtime {
            rt.disconnect().await;
        }
    }

    /// Get the current list of online users.
    pub async fn online_users(&self) -> Vec<OnlineUser> {
        self.online_users.read().await.values().cloned().collect()
    }

    /// Check if connected.
    pub async fn is_connected(&self) -> bool {
        *self.connected.read().await
    }

    /// Get our identity.
    pub fn identity(&self) -> &Identity {
        &self.identity
    }
}

// ---------------------------------------------------------------------------
// Event Translator
// ---------------------------------------------------------------------------

/// Background task that translates `RealtimeEvent`s into `PresenceEvent`s.
async fn event_translator(
    mut rt_rx: mpsc::Receiver<RealtimeEvent>,
    event_tx: mpsc::Sender<PresenceEvent>,
    online_users: Arc<RwLock<HashMap<String, OnlineUser>>>,
    connected: Arc<RwLock<bool>>,
    our_user_id: &str,
) {
    while let Some(rt_event) = rt_rx.recv().await {
        match rt_event {
            RealtimeEvent::Connected => {
                *connected.write().await = true;
            }
            RealtimeEvent::ChannelJoined { .. } => {
                debug!("Joined presence channel");
            }
            RealtimeEvent::PresenceState { state, .. } => {
                // Full state snapshot — populate online_users.
                let mut users = online_users.write().await;
                users.clear();
                for (key, metas) in &state {
                    if let Some(user) = parse_presence_meta(metas) {
                        users.insert(key.clone(), user);
                    }
                }
                let count = users.len() as u32;
                drop(users);
                let _ = event_tx
                    .send(PresenceEvent::Connected {
                        online_count: count,
                    })
                    .await;
            }
            RealtimeEvent::PresenceDiff {
                joins, leaves, ..
            } => {
                let mut users = online_users.write().await;

                // Process joins.
                for (key, metas) in &joins {
                    if let Some(user) = parse_presence_meta(metas) {
                        users.insert(key.clone(), user.clone());
                        let _ = event_tx
                            .send(PresenceEvent::UserOnline(user))
                            .await;
                    }
                }

                // Process leaves.
                for (key, metas) in &leaves {
                    let display_name = metas
                        .first()
                        .and_then(|m| m.get("display_name"))
                        .and_then(|n| n.as_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    users.remove(key);
                    let _ = event_tx
                        .send(PresenceEvent::UserOffline {
                            user_id: key.clone(),
                            display_name,
                        })
                        .await;
                }
            }
            RealtimeEvent::Broadcast {
                event, payload, ..
            } => {
                handle_broadcast(&event, &payload, &online_users, &event_tx, our_user_id)
                    .await;
            }
            RealtimeEvent::Disconnected => {
                *connected.write().await = false;
                online_users.write().await.clear();
                let _ = event_tx.send(PresenceEvent::Disconnected).await;
            }
            RealtimeEvent::Error(msg) => {
                let _ = event_tx.send(PresenceEvent::Error(msg)).await;
            }
            RealtimeEvent::ChannelError { message, .. } => {
                warn!(message = %message, "Channel error");
                let _ = event_tx.send(PresenceEvent::Error(message)).await;
            }
        }
    }
}

/// Parse an `OnlineUser` from presence meta entries.
fn parse_presence_meta(metas: &[serde_json::Value]) -> Option<OnlineUser> {
    let meta = metas.first()?;
    Some(OnlineUser {
        user_id: meta.get("user_id")?.as_str()?.to_string(),
        display_name: meta
            .get("display_name")
            .and_then(|n| n.as_str())
            .unwrap_or("Unknown")
            .to_string(),
        status: meta
            .get("status")
            .and_then(|s| serde_json::from_value(s.clone()).ok())
            .unwrap_or_default(),
        activity: meta
            .get("activity")
            .and_then(|a| a.as_str())
            .map(|s| s.to_string()),
    })
}

/// Dispatch a broadcast event to the appropriate `PresenceEvent`.
async fn handle_broadcast(
    event: &str,
    payload: &serde_json::Value,
    online_users: &Arc<RwLock<HashMap<String, OnlineUser>>>,
    event_tx: &mpsc::Sender<PresenceEvent>,
    our_user_id: &str,
) {
    match event {
        events::ACTIVITY_UPDATE => {
            if let Ok(p) = serde_json::from_value::<ActivityUpdatePayload>(payload.clone()) {
                let user = OnlineUser {
                    user_id: p.user_id.clone(),
                    display_name: p.display_name,
                    status: p.status,
                    activity: p.activity,
                };
                online_users
                    .write()
                    .await
                    .insert(p.user_id, user.clone());
                let _ = event_tx
                    .send(PresenceEvent::ActivityChanged(user))
                    .await;
            }
        }
        events::GAME_INVITE => {
            if let Ok(p) = serde_json::from_value::<GameInvitePayload>(payload.clone()) {
                let _ = event_tx
                    .send(PresenceEvent::GameInvite {
                        user_id: p.user_id,
                        display_name: p.display_name,
                        game: p.game,
                        code: p.code,
                    })
                    .await;
            }
        }
        events::POKE => {
            if let Ok(p) = serde_json::from_value::<PokePayload>(payload.clone()) {
                // Only emit if the poke is targeted at us.
                if p.target_user_id == our_user_id {
                    let _ = event_tx
                        .send(PresenceEvent::Poked {
                            user_id: p.user_id,
                            display_name: p.display_name,
                        })
                        .await;
                }
            }
        }
        events::CHAT_MESSAGE => {
            if let Ok(p) = serde_json::from_value::<ChatMessagePayload>(payload.clone()) {
                let _ = event_tx
                    .send(PresenceEvent::ChatMessage {
                        user_id: p.user_id,
                        display_name: p.display_name,
                        channel: p.channel,
                        content: p.content,
                    })
                    .await;
            }
        }
        _ => {
            debug!(event = %event, "Unhandled broadcast event");
        }
    }
}

/// Get the current timestamp as an ISO 8601 string.
fn chrono_now() -> String {
    // Use a simple approach without adding chrono as a dependency.
    // SystemTime gives us epoch seconds which we format as a string.
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    // ISO-ish format: just use epoch millis as a string for simplicity.
    // A proper ISO format would require the chrono crate.
    format!("{}", duration.as_millis())
}
