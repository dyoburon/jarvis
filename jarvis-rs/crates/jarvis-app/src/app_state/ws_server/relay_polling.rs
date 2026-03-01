//! Polling relay events on the main thread.

use crate::app_state::core::JarvisApp;

use super::relay_client::RelayEvent;

impl JarvisApp {
    /// Process relay status events (non-blocking).
    ///
    /// Called from the main poll loop to track relay connection state.
    pub(in crate::app_state) fn poll_relay_events(&mut self) {
        if let Some(ref rx) = self.relay_event_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    RelayEvent::Connected { session_id } => {
                        tracing::info!(session = %session_id, "Connected to relay");
                        self.relay_session_id = Some(session_id);
                    }
                    RelayEvent::PeerConnected => {
                        tracing::info!("Mobile peer connected via relay");
                        self.relay_peer_connected = true;
                        self.needs_redraw = true;
                    }
                    RelayEvent::PeerDisconnected => {
                        tracing::info!("Mobile peer disconnected from relay");
                        self.relay_peer_connected = false;
                        self.needs_redraw = true;
                    }
                    RelayEvent::KeyExchange { dh_pubkey } => {
                        tracing::info!("Deriving shared key from mobile DH pubkey");
                        if let Some(ref mut crypto) = self.crypto {
                            match crypto.derive_shared_key(&dh_pubkey) {
                                Ok(handle) => match crypto.export_key(handle) {
                                    Ok(key_bytes) => {
                                        if let Some(ref tx) = self.relay_key_tx {
                                            let _ = tx.send(Some(key_bytes));
                                            tracing::info!(
                                                "Shared key derived and sent to relay task"
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!(error = %e, "Failed to export derived key");
                                    }
                                },
                                Err(e) => {
                                    tracing::error!(error = %e, "ECDH key derivation failed");
                                }
                            }
                        } else {
                            tracing::error!("CryptoService not available for key exchange");
                        }
                    }
                    RelayEvent::Disconnected => {
                        self.relay_peer_connected = false;
                        if let Some(ref tx) = self.relay_key_tx {
                            let _ = tx.send(None);
                        }
                    }
                    RelayEvent::Encrypted => {
                        tracing::info!("Relay connection is now encrypted");
                        // Clear the QR code from the terminal and show success
                        if let Some(pane_id) = self.pairing_pane_id.take() {
                            if let Some(ref registry) = self.webviews {
                                if let Some(handle) = registry.get(pane_id) {
                                    // Clear screen and show paired message
                                    let clear = "\x1b[2J\x1b[H\x1b[32m  Mobile paired (encrypted)\x1b[0m\r\n\r\n";
                                    let payload = serde_json::json!({ "data": clear });
                                    let _ = handle.send_ipc("pty_output", &payload);
                                }
                            }
                        }
                    }
                    RelayEvent::Error(msg) => {
                        tracing::warn!(error = %msg, "Relay error");
                    }
                }
            }
        }
    }
}
