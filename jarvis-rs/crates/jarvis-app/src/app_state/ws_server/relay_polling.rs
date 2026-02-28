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
                    RelayEvent::Disconnected => {
                        self.relay_peer_connected = false;
                    }
                    RelayEvent::Encrypted => {
                        tracing::info!("Relay connection is now encrypted");
                    }
                    RelayEvent::Error(msg) => {
                        tracing::warn!(error = %msg, "Relay error");
                    }
                }
            }
        }
    }
}
