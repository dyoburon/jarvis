//! Background task that translates `RealtimeEvent`s into `PresenceEvent`s.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tracing::{debug, warn};

use crate::protocol::{
    events, ActivityUpdatePayload, ChatMessagePayload, GameInvitePayload, OnlineUser, PokePayload,
};
use crate::realtime::RealtimeEvent;

use super::types::PresenceEvent;

// ---------------------------------------------------------------------------
// Event Translator
// ---------------------------------------------------------------------------

/// Background task that translates `RealtimeEvent`s into `PresenceEvent`s.
pub(crate) async fn event_translator(
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
            RealtimeEvent::PresenceDiff { joins, leaves, .. } => {
                let mut users = online_users.write().await;

                // Process joins.
                for (key, metas) in &joins {
                    if let Some(user) = parse_presence_meta(metas) {
                        users.insert(key.clone(), user.clone());
                        let _ = event_tx.send(PresenceEvent::UserOnline(user)).await;
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
            RealtimeEvent::Broadcast { event, payload, .. } => {
                handle_broadcast(&event, &payload, &online_users, &event_tx, our_user_id).await;
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

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
                online_users.write().await.insert(p.user_id, user.clone());
                let _ = event_tx.send(PresenceEvent::ActivityChanged(user)).await;
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
