//! Relay client startup.

use std::sync::Arc;

use crate::app_state::core::JarvisApp;

use super::broadcast::MobileBroadcaster;
use super::relay_client::{run_relay_client, RelayClientConfig};

/// Path to the persisted session ID file within the jarvis config directory.
fn session_id_path() -> Option<std::path::PathBuf> {
    dirs::config_dir().map(|d| d.join("jarvis").join("relay_session_id"))
}

/// Generate a random 32-character alphanumeric session ID.
fn generate_session_id() -> String {
    (0..32)
        .map(|_| {
            let idx = rand::random::<u8>() % 36;
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'a' + idx - 10) as char
            }
        })
        .collect()
}

/// Load persisted session ID from disk, or generate and save a new one.
fn load_or_create_session_id() -> String {
    if let Some(path) = session_id_path() {
        if let Ok(id) = std::fs::read_to_string(&path) {
            let id = id.trim().to_string();
            if id.len() >= 32 {
                tracing::info!(session_id = %id, "Loaded persisted relay session ID");
                return id;
            }
            if !id.is_empty() {
                tracing::info!("Session ID too short ({} chars), regenerating", id.len());
            }
        }
        let id = generate_session_id();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&path, &id);
        tracing::info!(session_id = %id, "Generated and saved new relay session ID");
        id
    } else {
        generate_session_id()
    }
}

impl JarvisApp {
    /// Revoke the current mobile pairing by generating a new session ID
    /// and restarting the relay client.
    pub(in crate::app_state) fn revoke_mobile_pairing(&mut self) {
        // Shut down existing relay connection
        if let Some(tx) = self.relay_shutdown_tx.take() {
            let _ = tx.try_send(());
        }
        self.mobile_broadcaster = None;
        self.mobile_cmd_rx = None;
        self.relay_event_rx = None;
        self.relay_session_id = None;
        self.relay_peer_connected = false;
        self.relay_key_tx = None;

        // Delete persisted session ID so a new one is generated
        if let Some(path) = session_id_path() {
            let _ = std::fs::remove_file(&path);
        }

        tracing::info!("Mobile pairing revoked");

        // Restart with a fresh session
        self.start_relay_client();
    }

    /// Start the outbound relay client on the tokio runtime.
    pub(in crate::app_state) fn start_relay_client(&mut self) {
        let relay_url = self.config.relay.url.clone();
        if relay_url.is_empty() {
            tracing::info!("Relay URL not configured, skipping mobile bridge");
            return;
        }

        let session_id = load_or_create_session_id();

        // Get DH pubkey from crypto service for key exchange.
        let dh_pubkey_base64 = self.crypto.as_ref().map(|c| c.dh_pubkey_base64.clone());

        let broadcaster = Arc::new(MobileBroadcaster::new());
        let broadcast_rx = broadcaster.subscribe();
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let (shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel(1);
        let (key_tx, key_rx) = tokio::sync::watch::channel(None);

        let rt = self.tokio_runtime.get_or_insert_with(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .expect("Failed to create tokio runtime for relay client")
        });

        let config = RelayClientConfig {
            relay_url: relay_url.clone(),
            session_id: session_id.clone(),
            dh_pubkey_base64,
            key_rx,
        };

        rt.spawn(async move {
            run_relay_client(config, broadcast_rx, cmd_tx, event_tx, shutdown_rx).await;
        });

        self.mobile_broadcaster = Some(broadcaster);
        self.mobile_cmd_rx = Some(cmd_rx);
        self.relay_event_rx = Some(event_rx);
        self.relay_session_id = Some(session_id.clone());
        self.relay_shutdown_tx = Some(shutdown_tx);
        self.relay_key_tx = Some(key_tx);

        tracing::info!(
            relay_url = %relay_url,
            session_id = %session_id,
            "Relay client started"
        );
    }
}
