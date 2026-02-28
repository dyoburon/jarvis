//! Per-connection handler: classify, register, then forward messages.

use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

use crate::protocol::{RelayHello, RelayResponse};
use crate::session::{Role, SessionStore};

/// Handle a single WebSocket connection.
pub async fn handle_connection(
    ws: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    addr: SocketAddr,
    store: SessionStore,
) {
    let (mut sink, mut stream) = ws.split();

    // 1. Read the hello message to identify this client.
    let (session_id, role) = match read_hello(&mut stream, addr).await {
        Some(v) => v,
        None => return,
    };

    // For desktop_hello, create the session. For mobile_hello, it must already exist.
    match role {
        Role::Desktop => {
            if !store.create_session(&session_id).await {
                // Session already exists — desktop is reconnecting. Allow it by
                // unregistering the old desktop first.
                store.unregister(&session_id, Role::Desktop).await;
                store.create_session(&session_id).await;
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
    }

    // 2. Create our receive channel and register.
    let (tx, mut rx) = mpsc::channel::<String>(256);
    let peer_tx = match store.register(&session_id, role, tx).await {
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

    // 4. If peer is already connected, notify both sides.
    if let Some(ref peer) = peer_tx {
        let _ = send_response(&mut sink, &RelayResponse::PeerConnected).await;
        let json = serde_json::to_string(&RelayResponse::PeerConnected).unwrap();
        let _ = peer.send(json).await;
    }

    // 5. Forwarding loop.
    loop {
        tokio::select! {
            // Messages from our receive channel → send to this client's WebSocket
            Some(msg) = rx.recv() => {
                if sink.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }

            // Messages from this client's WebSocket → forward to peer
            frame = stream.next() => {
                match frame {
                    Some(Ok(Message::Text(text))) => {
                        // Forward to peer if connected
                        if let Some(peer) = store.get_peer_tx(&session_id, role).await {
                            if peer.send(text.to_string()).await.is_err() {
                                // Peer's channel closed — they disconnected
                                tracing::debug!(session = %session_id, "Peer channel closed");
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

    // Notify peer of disconnection.
    if let Some(peer) = store.get_peer_tx(&session_id, role).await {
        let json = serde_json::to_string(&RelayResponse::PeerDisconnected).unwrap();
        let _ = peer.send(json).await;
    }

    store.unregister(&session_id, role).await;
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
