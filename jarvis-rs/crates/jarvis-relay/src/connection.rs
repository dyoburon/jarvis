//! Per-connection handler: classify, register, then forward messages.

use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use crate::protocol::{RelayHello, RelayResponse};
use crate::rate_limit::RateLimiter;
use crate::session::{Role, SessionKind, SessionStore};

/// Handle a single WebSocket connection.
pub async fn handle_connection(
    ws: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    addr: SocketAddr,
    store: SessionStore,
    limiter: &RateLimiter,
) {
    let (mut sink, mut stream) = ws.split();

    // 1. Read the hello message to identify this client.
    let (session_id, role) = match read_hello(&mut stream, addr).await {
        Some(v) => v,
        None => return,
    };

    // Validate session ID length.
    if session_id.len() > limiter.max_session_id_len() || session_id.is_empty() {
        let _ = send_response(
            &mut sink,
            &RelayResponse::Error {
                message: "invalid session ID".into(),
            },
        )
        .await;
        return;
    }

    // Bridge sessions follow the existing desktop/mobile flow.
    match role.session_kind() {
        SessionKind::Bridge => match role {
            Role::Desktop => {
                // Enforce global session cap.
                if store.count().await >= limiter.max_total_sessions() {
                    // Check if this is a reconnect (session already exists).
                    if !store.exists(&session_id).await {
                        let _ = send_response(
                            &mut sink,
                            &RelayResponse::Error {
                                message: "server at capacity".into(),
                            },
                        )
                        .await;
                        return;
                    }
                }

                match store.ensure_session(&session_id, SessionKind::Bridge).await {
                    Ok(true) => {}
                    Ok(false) => {
                        // Session already exists -- desktop is reconnecting. Allow it by
                        // unregistering the old desktop first.
                        store.unregister(&session_id, Role::Desktop).await;
                        let _ = store.ensure_session(&session_id, SessionKind::Bridge).await;
                    }
                    Err(_) => {
                        let _ = send_response(
                            &mut sink,
                            &RelayResponse::Error {
                                message: "session kind mismatch".into(),
                            },
                        )
                        .await;
                        return;
                    }
                }
            }
            Role::Mobile => {
                // Verify session exists (created by desktop)
                if !store.exists(&session_id).await {
                    let _ = send_response(
                        &mut sink,
                        &RelayResponse::Error {
                            message: "session not found".into(),
                        },
                    )
                    .await;
                    return;
                }
            }
            _ => {}
        },
        SessionKind::Broadcast => {
            if let Err(e) = store
                .ensure_session(&session_id, SessionKind::Broadcast)
                .await
            {
                let _ = send_response(&mut sink, &RelayResponse::Error { message: e.into() }).await;
                return;
            }

            if matches!(role, Role::Host) {
                store.unregister(&session_id, Role::Host).await;
            }
        }
    }

    // 2. Create our receive channel and register.
    let (tx, mut rx) = mpsc::channel::<String>(256);
    let peer_tx = if role.session_kind() == SessionKind::Bridge {
        match store.register_bridge(&session_id, role, tx).await {
            Ok(peer) => peer,
            Err(e) => {
                let _ = send_response(
                    &mut sink,
                    &RelayResponse::Error {
                        message: e.to_string(),
                    },
                )
                .await;
                return;
            }
        }
    } else {
        match store.register_broadcast(&session_id, role, tx).await {
            Ok(registration) => {
                if matches!(role, Role::Spectator) && registration.host_connected {
                    let _ = send_response(&mut sink, &RelayResponse::HostConnected).await;
                }
                None
            }
            Err(e) => {
                let _ = send_response(
                    &mut sink,
                    &RelayResponse::Error {
                        message: e.to_string(),
                    },
                )
                .await;
                return;
            }
        }
    };

    tracing::info!(
        peer = %addr,
        session = %session_id,
        role = ?role,
        "Client registered"
    );

    // 3. Send session_ready.
    if send_response(
        &mut sink,
        &RelayResponse::SessionReady {
            session_id: session_id.clone(),
        },
    )
    .await
    .is_err()
    {
        store.unregister(&session_id, role).await;
        return;
    }

    // 4. Notify peers.
    if role.session_kind() == SessionKind::Bridge {
        if let Some(ref peer) = peer_tx {
            let _ = send_response(&mut sink, &RelayResponse::PeerConnected).await;
            let json = serde_json::to_string(&RelayResponse::PeerConnected).unwrap();
            let _ = peer.send(json).await;
        }
    } else {
        if matches!(role, Role::Host) {
            let json = serde_json::to_string(&RelayResponse::HostConnected).unwrap();
            for peer in store.spectator_targets(&session_id).await {
                let _ = peer.send(json.clone()).await;
            }
        }

        notify_viewer_count(&store, &session_id).await;
    }

    // 5. Forwarding loop.
    loop {
        tokio::select! {
            // Messages from our receive channel -> send to this client's WebSocket
            Some(msg) = rx.recv() => {
                if sink.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }

            // Messages from this client's WebSocket -> forward to peer
            frame = stream.next() => {
                match frame {
                    Some(Ok(Message::Text(text))) => {
                        if role.session_kind() == SessionKind::Bridge {
                            if let Some(peer) = store.get_peer_tx(&session_id, role).await {
                                // Mobile sends `{"type":"ping"}` on the WebSocket for keepalive; desktop
                                // does not consume it as a relay envelope — drop here to avoid noise.
                                if !is_relay_keepalive_ping(&text) {
                                    if peer.send(text.to_string()).await.is_err() {
                                        tracing::debug!(session = %session_id, "Peer channel closed");
                                    }
                                }
                            }
                        } else if matches!(role, Role::Host) {
                            for peer in store.spectator_targets(&session_id).await {
                                if peer.send(text.to_string()).await.is_err() {
                                    tracing::debug!(session = %session_id, "Spectator channel closed");
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = sink.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    Some(Err(e)) => {
                        tracing::debug!(peer = %addr, error = %e, "WS error");
                        break;
                    }
                    _ => {}
                }
            }
        }
    }

    // 6. Cleanup.
    tracing::info!(
        peer = %addr,
        session = %session_id,
        role = ?role,
        "Client disconnected"
    );

    drop(rx);
    store.unregister(&session_id, role).await;

    if role.session_kind() == SessionKind::Bridge {
        if let Some(peer) = store.get_peer_tx(&session_id, role).await {
            let json = serde_json::to_string(&RelayResponse::PeerDisconnected).unwrap();
            let _ = peer.send(json).await;
        }
    } else {
        if matches!(role, Role::Host) {
            let json = serde_json::to_string(&RelayResponse::HostDisconnected).unwrap();
            for peer in store.spectator_targets(&session_id).await {
                let _ = peer.send(json.clone()).await;
            }
        }
        notify_viewer_count(&store, &session_id).await;
    }
}

fn is_relay_keepalive_ping(text: &str) -> bool {
    match serde_json::from_str::<serde_json::Value>(text) {
        Ok(v) => v.get("type").and_then(|t| t.as_str()) == Some("ping"),
        Err(_) => false,
    }
}

/// Read and parse the first message as a RelayHello.
async fn read_hello(
    stream: &mut futures_util::stream::SplitStream<
        tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    >,
    addr: SocketAddr,
) -> Option<(String, Role)> {
    // Wait up to 10 seconds for the hello message.
    let frame = tokio::time::timeout(std::time::Duration::from_secs(10), stream.next()).await;

    match frame {
        Ok(Some(Ok(Message::Text(text)))) => match serde_json::from_str::<RelayHello>(&text) {
            Ok(RelayHello::DesktopHello { session_id }) => Some((session_id, Role::Desktop)),
            Ok(RelayHello::MobileHello { session_id }) => Some((session_id, Role::Mobile)),
            Ok(RelayHello::HostHello { session_id }) => Some((session_id, Role::Host)),
            Ok(RelayHello::SpectatorHello { session_id }) => Some((session_id, Role::Spectator)),
            Err(e) => {
                tracing::warn!(peer = %addr, error = %e, "Invalid hello message");
                None
            }
        },
        Ok(Some(Ok(_))) => {
            tracing::warn!(peer = %addr, "Expected text hello, got binary");
            None
        }
        Ok(Some(Err(e))) => {
            tracing::warn!(peer = %addr, error = %e, "WS error during hello");
            None
        }
        Ok(None) => {
            tracing::debug!(peer = %addr, "Connection closed before hello");
            None
        }
        Err(_) => {
            tracing::warn!(peer = %addr, "Hello timeout (10s)");
            None
        }
    }
}

/// Send a RelayResponse as a JSON text frame.
async fn send_response(
    sink: &mut futures_util::stream::SplitSink<
        tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
        Message,
    >,
    response: &RelayResponse,
) -> Result<(), tokio_tungstenite::tungstenite::Error> {
    let json = serde_json::to_string(response).unwrap();
    sink.send(Message::Text(json.into())).await
}

async fn notify_viewer_count(store: &SessionStore, session_id: &str) {
    let json = serde_json::to_string(&RelayResponse::ViewerCount {
        count: store.viewer_count(session_id).await,
    })
    .unwrap();

    for peer in store.broadcast_targets(session_id).await {
        let _ = peer.send(json.clone()).await;
    }
}
