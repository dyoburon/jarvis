//! Incoming Phoenix message handler and presence parsing.

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info, warn};

use super::connection::PendingChannel;
use super::types::{PhoenixMessage, RealtimeEvent};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the short topic name from a Phoenix topic (strip "realtime:" prefix).
fn strip_topic_prefix(topic: &str) -> &str {
    topic.strip_prefix("realtime:").unwrap_or(topic)
}

/// Parse a Phoenix presence map into `HashMap<key, Vec<meta>>`.
///
/// Supabase sends presence as `{ "key": { "metas": [{ ... }] } }`.
pub(crate) fn parse_presence_map(
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

// ---------------------------------------------------------------------------
// Message Handler
// ---------------------------------------------------------------------------

/// Handle a single incoming Phoenix message.
pub(crate) async fn handle_phoenix_message(
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
