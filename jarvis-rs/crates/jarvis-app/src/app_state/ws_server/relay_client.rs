//! Outbound relay client: connects to the relay server and bridges PTY data.

use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use super::crypto_bridge::RelayCipher;
use super::protocol::{ClientMessage, ServerMessage};
use super::relay_protocol::{DesktopHello, RelayEnvelope, RelayResponse};

/// Commands forwarded from mobile → main thread.
pub enum ClientCommand {
    PtyInput { pane_id: u32, data: String },
    PtyResize { pane_id: u32, cols: u16, rows: u16 },
}

/// Events the relay client sends to the main thread.
#[allow(dead_code)]
pub enum RelayEvent {
    Connected { session_id: String },
    PeerConnected,
    PeerDisconnected,
    KeyExchange { dh_pubkey: String },
    Encrypted,
    Disconnected,
    Error(String),
}

/// Configuration for the relay client.
pub struct RelayClientConfig {
    pub relay_url: String,
    pub session_id: String,
    /// Desktop's DH public key (SPKI DER, base64). Sent to mobile for ECDH.
    pub dh_pubkey_base64: Option<String>,
    /// Watch channel receiver for the derived AES key bytes (set by main thread after key exchange).
    pub key_rx: tokio::sync::watch::Receiver<Option<[u8; 32]>>,
}

/// Run the relay client with auto-reconnect.
pub async fn run_relay_client(
    config: RelayClientConfig,
    mut broadcast_rx: broadcast::Receiver<ServerMessage>,
    cmd_tx: std::sync::mpsc::Sender<ClientCommand>,
    event_tx: std::sync::mpsc::Sender<RelayEvent>,
    mut shutdown_rx: mpsc::Receiver<()>,
) {
    let mut backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(30);

    loop {
        tracing::info!(
            url = %config.relay_url,
            session = %config.session_id,
            "Connecting to relay..."
        );

        match connect_async(&config.relay_url).await {
            Ok((ws, _)) => {
                backoff = Duration::from_secs(1);
                let result = relay_session(
                    ws,
                    &config,
                    &mut broadcast_rx,
                    &cmd_tx,
                    &event_tx,
                    &mut shutdown_rx,
                )
                .await;

                match result {
                    SessionResult::Shutdown => {
                        tracing::info!("Relay client shutting down");
                        return;
                    }
                    SessionResult::Disconnected(reason) => {
                        tracing::warn!(reason = %reason, "Relay connection lost");
                        let _ = event_tx.send(RelayEvent::Disconnected);
                    }
                }
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to connect to relay");
                let _ = event_tx.send(RelayEvent::Error(format!("connect failed: {e}")));
            }
        }

        tokio::select! {
            _ = tokio::time::sleep(backoff) => {}
            _ = shutdown_rx.recv() => return,
        }

        backoff = (backoff * 2).min(max_backoff);
    }
}

enum SessionResult {
    Shutdown,
    Disconnected(String),
}

/// Handle a single relay session: handshake → key exchange → forward loop.
async fn relay_session(
    ws: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    config: &RelayClientConfig,
    broadcast_rx: &mut broadcast::Receiver<ServerMessage>,
    cmd_tx: &std::sync::mpsc::Sender<ClientCommand>,
    event_tx: &std::sync::mpsc::Sender<RelayEvent>,
    shutdown_rx: &mut mpsc::Receiver<()>,
) -> SessionResult {
    let (mut sink, mut stream) = ws.split();

    // 1. Send desktop_hello
    let hello = serde_json::to_string(&DesktopHello::new(config.session_id.clone())).unwrap();
    if sink.send(Message::Text(hello.into())).await.is_err() {
        return SessionResult::Disconnected("failed to send hello".into());
    }

    // 2. Wait for session_ready
    match read_relay_response(&mut stream).await {
        Some(RelayResponse::SessionReady { session_id }) => {
            tracing::info!(session = %session_id, "Relay session ready");
            let _ = event_tx.send(RelayEvent::Connected {
                session_id: session_id.clone(),
            });
        }
        Some(RelayResponse::Error { message }) => {
            return SessionResult::Disconnected(format!("relay error: {message}"));
        }
        _ => {
            return SessionResult::Disconnected("unexpected relay response".into());
        }
    }

    // The cipher gets set after key exchange completes via the watch channel.
    let mut cipher: Option<RelayCipher> = None;
    let mut key_rx = config.key_rx.clone();

    // 3. Forwarding loop
    loop {
        tokio::select! {
            // Watch for derived key from main thread
            result = key_rx.changed() => {
                if result.is_ok() {
                    if let Some(key_bytes) = *key_rx.borrow() {
                        tracing::info!("Received derived key, encryption enabled");
                        cipher = Some(RelayCipher::new(key_bytes));
                        let _ = event_tx.send(RelayEvent::Encrypted);
                    }
                }
            }

            // PTY output → encrypt → send to relay (drop if no cipher)
            msg = broadcast_rx.recv() => {
                match msg {
                    Ok(server_msg) => {
                        let envelope = if let Some(ref c) = cipher {
                            match c.encrypt_server_message(&server_msg) {
                                Ok(env) => env,
                                Err(e) => {
                                    tracing::warn!(error = %e, "Encryption failed, dropping message");
                                    continue;
                                }
                            }
                        } else {
                            // No cipher yet — drop the message.
                            // Key exchange should complete within milliseconds.
                            tracing::trace!("No cipher yet, dropping outbound message");
                            continue;
                        };
                        let json = serde_json::to_string(&envelope).unwrap();
                        if sink.send(Message::Text(json.into())).await.is_err() {
                            return SessionResult::Disconnected("send failed".into());
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(skipped = n, "Relay broadcast lagged");
                    }
                    Err(broadcast::error::RecvError::Closed) => {
                        return SessionResult::Shutdown;
                    }
                }
            }

            // Messages from relay → parse and route
            frame = stream.next() => {
                match frame {
                    Some(Ok(Message::Text(text))) => {
                        // Try as relay control message first
                        if let Ok(relay_msg) = serde_json::from_str::<RelayResponse>(&text) {
                            match relay_msg {
                                RelayResponse::PeerConnected => {
                                    tracing::info!("Mobile peer connected");
                                    let _ = event_tx.send(RelayEvent::PeerConnected);

                                    // Send our DH pubkey for key exchange
                                    if let Some(ref dh_pub) = config.dh_pubkey_base64 {
                                        let ke = RelayEnvelope::KeyExchange {
                                            dh_pubkey: dh_pub.clone(),
                                        };
                                        let json = serde_json::to_string(&ke).unwrap();
                                        let _ = sink.send(Message::Text(json.into())).await;
                                        tracing::info!("Sent DH pubkey to mobile");
                                    }
                                }
                                RelayResponse::PeerDisconnected => {
                                    tracing::info!("Mobile peer disconnected");
                                    cipher = None; // Reset encryption on peer disconnect
                                    let _ = event_tx.send(RelayEvent::PeerDisconnected);
                                }
                                RelayResponse::Error { message } => {
                                    tracing::warn!(error = %message, "Relay error");
                                    let _ = event_tx.send(RelayEvent::Error(message));
                                }
                                _ => {}
                            }
                            continue;
                        }

                        // Try as relay envelope (from mobile peer)
                        if let Ok(envelope) = serde_json::from_str::<RelayEnvelope>(&text) {
                            match envelope {
                                RelayEnvelope::Plaintext { payload } => {
                                    handle_client_message(&payload, cmd_tx);
                                }
                                RelayEnvelope::KeyExchange { dh_pubkey } => {
                                    tracing::info!("Received mobile DH pubkey");
                                    let _ = event_tx.send(RelayEvent::KeyExchange {
                                        dh_pubkey,
                                    });
                                }
                                RelayEnvelope::Encrypted { iv, ct } => {
                                    if let Some(ref c) = cipher {
                                        match c.decrypt_client_message(&iv, &ct) {
                                            Ok(client_msg) => {
                                                dispatch_client_message(client_msg, cmd_tx);
                                            }
                                            Err(e) => {
                                                tracing::warn!(error = %e, "Decryption failed");
                                            }
                                        }
                                    } else {
                                        tracing::warn!("Received encrypted message but no cipher");
                                    }
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = sink.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        return SessionResult::Disconnected("relay closed connection".into());
                    }
                    Some(Err(e)) => {
                        return SessionResult::Disconnected(format!("ws error: {e}"));
                    }
                    _ => {}
                }
            }

            // Shutdown signal
            _ = shutdown_rx.recv() => {
                let _ = sink.close().await;
                return SessionResult::Shutdown;
            }
        }
    }
}

/// Parse a client message from JSON and forward to main thread.
fn handle_client_message(json: &str, cmd_tx: &std::sync::mpsc::Sender<ClientCommand>) {
    match serde_json::from_str::<ClientMessage>(json) {
        Ok(msg) => dispatch_client_message(msg, cmd_tx),
        Err(e) => {
            tracing::debug!(error = %e, "Bad client message from mobile");
        }
    }
}

/// Dispatch a parsed ClientMessage to the main thread.
fn dispatch_client_message(msg: ClientMessage, cmd_tx: &std::sync::mpsc::Sender<ClientCommand>) {
    match msg {
        ClientMessage::PtyInput { pane_id, data } => {
            let _ = cmd_tx.send(ClientCommand::PtyInput { pane_id, data });
        }
        ClientMessage::PtyResize { pane_id, cols, rows } => {
            let _ = cmd_tx.send(ClientCommand::PtyResize { pane_id, cols, rows });
        }
        ClientMessage::Ping => {}
    }
}

/// Read a single text frame and parse as RelayResponse.
async fn read_relay_response(
    stream: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
) -> Option<RelayResponse> {
    let timeout = tokio::time::timeout(Duration::from_secs(10), stream.next()).await;
    match timeout {
        Ok(Some(Ok(Message::Text(text)))) => serde_json::from_str(&text).ok(),
        _ => None,
    }
}
