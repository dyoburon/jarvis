//! Graceful shutdown: kill PTYs, destroy webviews, disconnect presence.

use std::time::Duration;

use super::core::JarvisApp;

// =============================================================================
// SHUTDOWN
// =============================================================================

impl JarvisApp {
    /// Perform graceful shutdown of all subsystems.
    ///
    /// Order matters:
    /// 1. Kill PTYs (stop shell processes first)
    /// 2. Destroy webviews (remove UI panels)
    /// 3. Disconnect presence (stop heartbeats, cancel background task)
    /// 4. Shut down tokio runtime (cancel async tasks)
    /// 5. Release GPU resources
    pub(super) fn shutdown(&mut self) {
        tracing::info!("Initiating graceful shutdown");

        // 1. Kill all PTY child processes
        self.ptys.kill_all();

        // 2. Destroy all webview panels
        if let Some(ref mut registry) = self.webviews {
            registry.destroy_all();
        }

        // 3. Disconnect presence (dropping senders signals the async task)
        self.presence_cmd_tx = None;
        self.presence_rx = None;
        self.online_users.clear();
        self.online_count = 0;

        // 4. Shut down tokio runtime (cancels presence background task)
        if let Some(rt) = self.tokio_runtime.take() {
            rt.shutdown_timeout(Duration::from_secs(2));
        }

        // 5. Release GPU resources
        self.render_state = None;

        tracing::info!("Graceful shutdown complete");
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use crate::app_state::core::JarvisApp;
    use jarvis_config::schema::JarvisConfig;
    use jarvis_platform::input::KeybindRegistry;

    #[test]
    fn shutdown_on_fresh_app_does_not_panic() {
        let config = JarvisConfig::default();
        let registry = KeybindRegistry::from_config(&config.keybinds);
        let mut app = JarvisApp::new(config, registry);

        app.shutdown();

        assert!(app.ptys.is_empty());
        assert!(app.online_users.is_empty());
        assert_eq!(app.online_count, 0);
        assert!(app.presence_cmd_tx.is_none());
        assert!(app.presence_rx.is_none());
        assert!(app.tokio_runtime.is_none());
        assert!(app.render_state.is_none());
    }

    #[test]
    fn shutdown_is_idempotent() {
        let config = JarvisConfig::default();
        let registry = KeybindRegistry::from_config(&config.keybinds);
        let mut app = JarvisApp::new(config, registry);

        app.shutdown();
        app.shutdown(); // second call must not panic

        assert!(app.ptys.is_empty());
        assert!(app.render_state.is_none());
    }

    #[test]
    fn shutdown_clears_all_presence_state() {
        let config = JarvisConfig::default();
        let registry = KeybindRegistry::from_config(&config.keybinds);
        let mut app = JarvisApp::new(config, registry);

        // Simulate some presence state
        app.online_count = 5;
        app.online_users.push(jarvis_social::OnlineUser {
            user_id: "test-user".to_string(),
            display_name: "Test".to_string(),
            status: jarvis_social::UserStatus::Online,
            activity: Some("coding".to_string()),
        });

        app.shutdown();

        assert_eq!(app.online_count, 0);
        assert!(app.online_users.is_empty());
    }
}
