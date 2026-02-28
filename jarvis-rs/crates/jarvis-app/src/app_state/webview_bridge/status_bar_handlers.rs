//! Status bar IPC handlers: panel toggle, open settings, status bar init.

use jarvis_webview::IpcPayload;

use crate::app_state::core::JarvisApp;

// =============================================================================
// ALLOWED TOGGLE PANELS
// =============================================================================

/// Panel names that can be toggled from the status bar.
const TOGGLEABLE_PANELS: &[&str] = &["terminal", "assistant", "chat", "presence", "settings"];

// =============================================================================
// HANDLERS
// =============================================================================

impl JarvisApp {
    /// Handle `panel_toggle` — show/hide/focus a panel by name.
    ///
    /// If the panel exists, focus it. If not, open a new one.
    pub(in crate::app_state) fn handle_panel_toggle(
        &mut self,
        _pane_id: u32,
        payload: &IpcPayload,
    ) {
        let panel_name = match payload {
            IpcPayload::Json(obj) => obj.get("panel").and_then(|v| v.as_str()),
            _ => None,
        };

        let panel_name = match panel_name {
            Some(name) if TOGGLEABLE_PANELS.contains(&name) => name.to_string(),
            Some(name) => {
                tracing::warn!(panel = %name, "panel_toggle: unknown panel");
                return;
            }
            None => {
                tracing::warn!("panel_toggle: missing panel name");
                return;
            }
        };

        tracing::info!(panel = %panel_name, "Status bar panel toggle");

        // Delegate to open_panel with the same payload format
        let open_payload = IpcPayload::Json(serde_json::json!({ "panel": panel_name }));
        self.handle_open_panel(0, &open_payload);
    }

    /// Handle `open_settings` — open or focus the settings panel.
    pub(in crate::app_state) fn handle_open_settings(&mut self, _pane_id: u32) {
        tracing::info!("Status bar: open settings");
        let payload = IpcPayload::Json(serde_json::json!({ "panel": "settings" }));
        self.handle_open_panel(0, &payload);
    }

    /// Handle `status_bar_init` — send current app state to the status bar.
    pub(in crate::app_state) fn handle_status_bar_init(&self, pane_id: u32) {
        if let Some(ref registry) = self.webviews {
            if let Some(handle) = registry.get(pane_id) {
                let status = serde_json::json!({
                    "online_count": self.online_count,
                    "active_panel": "terminal",
                    "connection": "connected",
                });
                if let Err(e) = handle.send_ipc("status_update", &status) {
                    tracing::warn!(pane_id, error = %e, "Failed to send status_update");
                }
            }
        }
    }

    /// Notify all webview panels about focus changes.
    ///
    /// Sends `focus_changed` with `{ "focused": true/false }` to each panel,
    /// and `status_update` with the active panel name to the status bar.
    pub(in crate::app_state) fn notify_focus_changed(&self) {
        let focused_id = self.tiling.focused_id();

        if let Some(ref registry) = self.webviews {
            for pane_id in registry.active_panes() {
                if let Some(handle) = registry.get(pane_id) {
                    let is_focused = pane_id == focused_id;
                    let payload = serde_json::json!({ "focused": is_focused });
                    if let Err(e) = handle.send_ipc("focus_changed", &payload) {
                        tracing::warn!(pane_id, error = %e, "Failed to send focus_changed");
                    }
                    // Give native focus to the focused webview so it receives
                    // keyboard and mouse events (macOS first responder).
                    if is_focused {
                        let _ = handle.focus();
                    }
                }
            }
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggleable_panels_contains_expected() {
        assert!(TOGGLEABLE_PANELS.contains(&"terminal"));
        assert!(TOGGLEABLE_PANELS.contains(&"assistant"));
        assert!(TOGGLEABLE_PANELS.contains(&"chat"));
        assert!(TOGGLEABLE_PANELS.contains(&"presence"));
        assert!(TOGGLEABLE_PANELS.contains(&"settings"));
    }

    #[test]
    fn toggleable_panels_rejects_unknown() {
        assert!(!TOGGLEABLE_PANELS.contains(&"eval"));
        assert!(!TOGGLEABLE_PANELS.contains(&""));
        assert!(!TOGGLEABLE_PANELS.contains(&"admin"));
    }
}
