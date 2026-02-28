//! Social presence: connecting to the presence server and polling events.

use jarvis_social::presence::{PresenceConfig, PresenceEvent};
use jarvis_social::{Identity, UserStatus};

use super::core::JarvisApp;
use super::types::PresenceCommand;

impl JarvisApp {
    /// Poll social presence events (non-blocking).
    ///
    /// Drains events from the bridge channel, updates the shadow user
    /// list, and forwards updates to presence panel webviews.
    pub(super) fn poll_presence(&mut self) {
        if let Some(ref rx) = self.presence_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    PresenceEvent::Connected { online_count } => {
                        self.online_count = online_count;
                        tracing::info!("Presence connected: {online_count} online");
                        self.send_presence_status_to_webviews();
                        self.send_presence_users_to_webviews();
                        // Set initial activity
                        self.send_presence_activity(
                            UserStatus::Online,
                            Some("in terminal".to_string()),
                        );
                    }
                    PresenceEvent::UserOnline(user) => {
                        self.online_count += 1;
                        self.online_users.push(user);
                        self.send_presence_status_to_webviews();
                        self.send_presence_users_to_webviews();
                    }
                    PresenceEvent::UserOffline { user_id, .. } => {
                        self.online_count = self.online_count.saturating_sub(1);
                        self.online_users.retain(|u| u.user_id != user_id);
                        self.send_presence_status_to_webviews();
                        self.send_presence_users_to_webviews();
                    }
                    PresenceEvent::ActivityChanged(user) => {
                        if let Some(u) = self
                            .online_users
                            .iter_mut()
                            .find(|u| u.user_id == user.user_id)
                        {
                            u.status = user.status;
                            u.activity = user.activity.clone();
                        }
                        self.send_presence_users_to_webviews();
                    }
                    PresenceEvent::Poked { display_name, .. } => {
                        tracing::info!("poke received");
                        self.notifications.push(
                            jarvis_common::notifications::Notification::info(
                                "Poke!",
                                format!("{display_name} poked you"),
                            ),
                        );
                        self.send_presence_notification_to_webviews(&format!(
                            "{display_name} poked you!"
                        ));
                    }
                    PresenceEvent::ChatMessage { content, .. } => {
                        tracing::info!("[chat] message received, {} chars", content.len());
                    }
                    PresenceEvent::Disconnected => {
                        self.online_count = 0;
                        self.online_users.clear();
                        tracing::info!("Presence disconnected");
                        self.send_presence_status_to_webviews();
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
            api_key: String::new(),
            heartbeat_interval: self.config.presence.heartbeat_interval as u64,
            ..Default::default()
        };

        let (sync_tx, sync_rx) = std::sync::mpsc::channel();
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::channel::<PresenceCommand>(64);

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

                    loop {
                        tokio::select! {
                            Some(event) = event_rx.recv() => {
                                if sync_tx.send(event).is_err() {
                                    break;
                                }
                            }
                            Some(cmd) = cmd_rx.recv() => {
                                match cmd {
                                    PresenceCommand::Poke { target_user_id } => {
                                        client.send_poke(&target_user_id).await;
                                    }
                                    PresenceCommand::UpdateActivity { status, activity } => {
                                        client.update_activity(status, activity).await;
                                    }
                                }
                            }
                            else => break,
                        }
                    }
                });

                self.presence_rx = Some(sync_rx);
                self.presence_cmd_tx = Some(cmd_tx);
                self.tokio_runtime = Some(rt);
                tracing::info!("Presence client started");
            }
            Err(e) => {
                tracing::warn!("Failed to start tokio runtime for presence: {e}");
            }
        }
    }

    /// Send an activity update to the presence server (non-blocking).
    pub(super) fn send_presence_activity(
        &self,
        status: UserStatus,
        activity: Option<String>,
    ) {
        if let Some(ref tx) = self.presence_cmd_tx {
            let cmd = PresenceCommand::UpdateActivity { status, activity };
            if let Err(e) = tx.try_send(cmd) {
                tracing::warn!(error = %e, "Failed to send activity update");
            }
        }
    }
}
