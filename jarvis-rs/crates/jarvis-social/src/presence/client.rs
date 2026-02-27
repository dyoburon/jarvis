//! Presence client that maintains a connection to Supabase Realtime.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};

use crate::identity::Identity;
use crate::protocol::{
    events, ActivityUpdatePayload, ChatMessagePayload, GameInvitePayload, OnlineUser, PokePayload,
    UserStatus,
};
use crate::realtime::{
    BroadcastConfig, ChannelConfig, PresenceConfig as RtPresenceConfig, RealtimeClient,
    RealtimeConfig,
};

use super::event_translator::event_translator;
use super::helpers::chrono_now;
use super::types::{PresenceConfig, PresenceEvent};

/// The Supabase Realtime channel name used for all social events.
const CHANNEL_NAME: &str = "jarvis-presence";

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
            join_client.join_channel(CHANNEL_NAME, channel_config).await;

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
                rt.broadcast(CHANNEL_NAME, events::GAME_INVITE, value).await;
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
