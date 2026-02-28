//! Window title management: reflects current activity.

use super::core::JarvisApp;

// =============================================================================
// WINDOW TITLE
// =============================================================================

impl JarvisApp {
    /// Update the window title to reflect the current activity.
    ///
    /// Format: "Jarvis — {activity}"
    pub(super) fn update_window_title(&self) {
        let Some(ref window) = self.window else {
            return;
        };

        let activity = if self.command_palette_open {
            "command palette"
        } else if self.assistant_open {
            "assistant"
        } else {
            "terminal"
        };

        window.set_title(&format!("Jarvis — {activity}"));
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
    fn update_title_without_window_does_not_panic() {
        let config = JarvisConfig::default();
        let registry = KeybindRegistry::from_config(&config.keybinds);
        let app = JarvisApp::new(config, registry);

        // window is None on a fresh app — should silently return
        app.update_window_title();
    }
}
