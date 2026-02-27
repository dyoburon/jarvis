//! Social presence: connecting to the presence server and polling events.

use jarvis_common::notifications::Notification;
use jarvis_social::presence::{PresenceConfig, PresenceEvent};
use jarvis_social::Identity;

use super::core::JarvisApp;

impl JarvisApp {
    /// Poll social presence events (non-blocking).
    pub(super) fn poll_presence(&mut self) {
        if let Some(ref rx) = self.presence_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    PresenceEvent::Connected { online_count } => {
                        self.online_count = online_count;
                        tracing::info!("Presence connected: {online_count} online");
                    }
                    PresenceEvent::UserOnline(_) => {
                        self.online_count += 1;
                    }
                    PresenceEvent::UserOffline { .. } => {
                        self.online_count = self.online_count.saturating_sub(1);
                    }
                    PresenceEvent::Poked {
                        display_name, ..
                    } => {
                        tracing::info!("poke received");
                        self.notifications.push(Notification::info(
                            "Poke!",
                            format!("{display_name} poked you"),
                        ));
                    }
                    PresenceEvent::ChatMessage { content, .. } => {
                        tracing::info!("[chat] message received, {} chars", content.len());
                    }
                    PresenceEvent::Disconnected => {
                        self.online_count = 0;
                        tracing::info!("Presence disconnected");
                    }
                    PresenceEvent::Error(msg) => {
                        tracing::warn!("Presence error: {msg}");
                    }
                    _ => {
                        tracing::debug!("unhandled presence event");
                    }
                }
                self.needs_redraw = true;
            }
        }
    }

    /// Start the social presence client in a background tokio runtime.
    pub(super) fn start_presence(&mut self) {
        if !self.config.presence.enabled {
            return;
        }

        // Need a non-empty server_url to connect
        if self.config.presence.server_url.is_empty() {
            tracing::debug!("Presence skipped: no server_url configured");
            return;
        }

        let hostname = std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .unwrap_or_else(|_| "jarvis-user".to_string());
        let identity = Identity::generate(&hostname);

        let presence_config = PresenceConfig {
            project_ref: self.config.presence.server_url.clone(),
            api_key: String::new(), // Would come from config/env in production
            heartbeat_interval: self.config.presence.heartbeat_interval as u64,
            ..Default::default()
        };

        let (sync_tx, sync_rx) = std::sync::mpsc::channel();

        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build();

        match rt {
            Ok(rt) => {
                rt.spawn(async move {
                    let mut client =
                        jarvis_social::PresenceClient::new(identity, presence_config);
                    let mut event_rx = client.start();
                    while let Some(event) = event_rx.recv().await {
                        if sync_tx.send(event).is_err() {
                            break;
                        }
                    }
                });

                self.presence_rx = Some(sync_rx);
                self.tokio_runtime = Some(rt);
                tracing::info!("Presence client started");
            }
            Err(e) => {
                tracing::warn!("Failed to start tokio runtime for presence: {e}");
            }
        }
    }
}
