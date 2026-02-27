//! Background WebSocket connection loop with auto-reconnect.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio_tungstenite::tungstenite::Message as WsMessage;
use tracing::{error, info, warn};

use super::handler::handle_phoenix_message;
use super::types::{
    ChannelConfig, PhoenixMessage, RealtimeCommand, RealtimeConfig, RealtimeEvent,
};

// ---------------------------------------------------------------------------
// Ref Counter
// ---------------------------------------------------------------------------

/// Monotonically increasing ref counter for Phoenix messages.
static REF_COUNTER: AtomicU64 = AtomicU64::new(1);

pub(crate) fn next_ref() -> String {
    REF_COUNTER.fetch_add(1, Ordering::Relaxed).to_string()
}

/// State for channels that should be (re)joined on reconnect.
#[derive(Clone)]
pub(crate) struct PendingChannel {
    pub(crate) config: ChannelConfig,
    pub(crate) presence_payload: Option<serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Connection Loop
// ---------------------------------------------------------------------------

/// Background task managing the WebSocket connection with auto-reconnect.
pub(crate) async fn connection_loop(
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
        info!(url = %url.split('?').next().unwrap_or(""), "Connecting to Supabase Realtime");

        match tokio::time::timeout(
            Duration::from_secs(15),
            tokio_tungstenite::connect_async(&url),
        )
        .await
        {
            Ok(Ok((ws_stream, _))) => {
                reconnect_delay = config.reconnect_delay_secs;
                *connected.write().await = true;
                let _ = event_tx.send(RealtimeEvent::Connected).await;

                let (ws_write, ws_read) = ws_stream.split();
                let ws_write = Arc::new(Mutex::new(ws_write));

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
                let heartbeat_handle = tokio::spawn(heartbeat_task(
                    heartbeat_write,
                    heartbeat_interval,
                ));

                // Spawn command forwarder.
                let cmd_write = Arc::clone(&ws_write);
                let cmd_rx = Arc::clone(&command_rx);
                let cmd_channels = Arc::clone(&joined_channels);
                let cmd_event_tx = event_tx.clone();
                let cmd_handle = tokio::spawn(command_forwarder(
                    cmd_rx,
                    cmd_write,
                    cmd_channels,
                    cmd_event_tx,
                ));

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
                                tracing::debug!(text = %text, "Unrecognized message from Supabase");
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
            Ok(Err(e)) => {
                error!(error = %e, "Failed to connect to Supabase Realtime");
                let _ = event_tx
                    .send(RealtimeEvent::Error(format!("Connection failed: {e}")))
                    .await;
            }
            Err(_elapsed) => {
                error!("WebSocket connection timed out after 15s");
                let _ = event_tx
                    .send(RealtimeEvent::Error(
                        "Connection timed out after 15s".to_string(),
                    ))
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
// Heartbeat
// ---------------------------------------------------------------------------

async fn heartbeat_task<S>(ws_write: Arc<Mutex<S>>, interval_secs: u64)
where
    S: futures_util::Sink<WsMessage> + Unpin,
{
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
    loop {
        interval.tick().await;
        let msg = PhoenixMessage {
            topic: "phoenix".to_string(),
            event: "heartbeat".to_string(),
            payload: serde_json::json!({}),
            msg_ref: Some(next_ref()),
        };
        if let Ok(json) = serde_json::to_string(&msg) {
            let mut writer = ws_write.lock().await;
            if writer.send(WsMessage::Text(json.into())).await.is_err() {
                break;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Command Forwarder
// ---------------------------------------------------------------------------

async fn command_forwarder<S>(
    cmd_rx: Arc<Mutex<mpsc::Receiver<RealtimeCommand>>>,
    cmd_write: Arc<Mutex<S>>,
    cmd_channels: Arc<RwLock<HashMap<String, PendingChannel>>>,
    cmd_event_tx: mpsc::Sender<RealtimeEvent>,
) where
    S: futures_util::Sink<WsMessage> + Unpin,
{
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
                        let _ = writer.send(WsMessage::Text(json.into())).await;
                    }
                }
                drop(channels);
                let mut writer = cmd_write.lock().await;
                let _ = writer.send(WsMessage::Close(None)).await;
                let _ = cmd_event_tx.send(RealtimeEvent::Disconnected).await;
                return; // Exit the command forwarder
            }
        }
    }
}
