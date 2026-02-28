//! Relay client startup.

use std::sync::Arc;

use crate::app_state::core::JarvisApp;

use super::broadcast::MobileBroadcaster;
use super::relay_client::{run_relay_client, RelayClientConfig};

impl JarvisApp {
    /// Start the outbound relay client on the tokio runtime.
    pub(in crate::app_state) fn start_relay_client(&mut self) {
        let relay_url = self.config.relay.url.clone();
        if relay_url.is_empty() {
            tracing::info!("Relay URL not configured, skipping mobile bridge");
            return;
        }

        // Generate a short session ID.
        let session_id: String = (0..8)
            .map(|_| {
                let idx = rand::random::<u8>() % 36;
                if idx < 10 {
                    (b'0' + idx) as char
                } else {
                    (b'a' + idx - 10) as char
                }
            })
            .collect();

        // Get DH pubkey from crypto service for key exchange.
        let dh_pubkey_base64 = self.crypto.as_ref().map(|c| c.dh_pubkey_base64.clone());

        let broadcaster = Arc::new(MobileBroadcaster::new());
        let broadcast_rx = broadcaster.subscribe();
        let (cmd_tx, cmd_rx) = std::sync::mpsc::channel();
        let (event_tx, event_rx) = std::sync::mpsc::channel();
        let (shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel(1);

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
            shared_key: None, // Set after key exchange
        };

        rt.spawn(async move {
            run_relay_client(config, broadcast_rx, cmd_tx, event_tx, shutdown_rx).await;
        });

        self.mobile_broadcaster = Some(broadcaster);
        self.mobile_cmd_rx = Some(cmd_rx);
        self.relay_event_rx = Some(event_rx);
        self.relay_session_id = Some(session_id.clone());
        self.relay_shutdown_tx = Some(shutdown_tx);

        tracing::info!(
            relay_url = %relay_url,
            session_id = %session_id,
            "Relay client started"
        );
    }
}
