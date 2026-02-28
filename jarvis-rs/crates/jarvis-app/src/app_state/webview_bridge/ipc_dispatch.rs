//! IPC message validation and dispatch from webview to Rust handlers.

use jarvis_webview::IpcMessage;

use crate::app_state::core::JarvisApp;

// =============================================================================
// IPC ALLOWLIST
// =============================================================================

/// Allowed IPC message kinds from JavaScript.
///
/// Any message with a `kind` not in this list is rejected and logged.
const ALLOWED_IPC_KINDS: &[&str] = &[
    "pty_input",
    "pty_resize",
    "pty_restart",
    "terminal_ready",
    "panel_focus",
    "presence_request_users",
    "presence_poke",
    "settings_init",
    "settings_set_theme",
    "settings_update",
    "settings_reset_section",
    "settings_get_config",
    "assistant_input",
    "open_panel",
    "panel_close",
    "ping",
];

/// Check whether an IPC message kind is in the allowlist.
pub fn is_ipc_kind_allowed(kind: &str) -> bool {
    ALLOWED_IPC_KINDS.contains(&kind)
}

// =============================================================================
// DISPATCH
// =============================================================================

impl JarvisApp {
    /// Handle a single IPC message from a webview.
    pub(in crate::app_state) fn handle_ipc_message(&mut self, pane_id: u32, body: &str) {
        let msg = match IpcMessage::from_json(body) {
            Some(m) => m,
            None => {
                tracing::warn!(
                    pane_id,
                    body_len = body.len(),
                    "IPC message rejected: failed to parse"
                );
                return;
            }
        };

        if !is_ipc_kind_allowed(&msg.kind) {
            tracing::warn!(
                pane_id,
                kind = %msg.kind,
                "IPC message rejected: unknown kind"
            );
            return;
        }

        tracing::debug!(pane_id, kind = %msg.kind, "IPC message dispatched");

        match msg.kind.as_str() {
            "ping" => {
                // Respond with pong — used for IPC round-trip testing
                if let Some(ref registry) = self.webviews {
                    if let Some(handle) = registry.get(pane_id) {
                        let payload = serde_json::json!("pong");
                        if let Err(e) = handle.send_ipc("pong", &payload) {
                            tracing::warn!(pane_id, error = %e, "Failed to send pong");
                        }
                    }
                }
            }
            "panel_focus" => {
                self.tiling.focus_pane(pane_id);
                self.needs_redraw = true;
            }
            "pty_input" => {
                self.handle_pty_input(pane_id, &msg.payload);
            }
            "pty_resize" => {
                self.handle_pty_resize(pane_id, &msg.payload);
            }
            "pty_restart" => {
                self.handle_pty_restart(pane_id, &msg.payload);
            }
            "terminal_ready" => {
                self.handle_terminal_ready(pane_id, &msg.payload);
            }
            "presence_request_users" => {
                self.handle_presence_request_users(pane_id, &msg.payload);
            }
            "presence_poke" => {
                self.handle_presence_poke(pane_id, &msg.payload);
            }
            "settings_init" => {
                self.handle_settings_init(pane_id, &msg.payload);
            }
            "settings_set_theme" => {
                self.handle_settings_set_theme(pane_id, &msg.payload);
            }
            "settings_update" => {
                self.handle_settings_update(pane_id, &msg.payload);
            }
            "settings_reset_section" => {
                self.handle_settings_reset_section(pane_id, &msg.payload);
            }
            "settings_get_config" => {
                self.handle_settings_get_config(pane_id, &msg.payload);
            }
            "assistant_input" => {
                self.handle_assistant_input(pane_id, &msg.payload);
            }
            "open_panel" => {
                self.handle_open_panel(pane_id, &msg.payload);
            }
            "panel_close" => {
                self.handle_panel_close(pane_id);
            }
            _ => {
                // Shouldn't happen — allowlist checked above
                tracing::warn!(pane_id, kind = %msg.kind, "Unhandled IPC kind");
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
    fn ipc_kind_allowed_valid() {
        assert!(is_ipc_kind_allowed("pty_input"));
        assert!(is_ipc_kind_allowed("ping"));
        assert!(is_ipc_kind_allowed("settings_set_theme"));
        assert!(is_ipc_kind_allowed("settings_update"));
        assert!(is_ipc_kind_allowed("settings_reset_section"));
        assert!(is_ipc_kind_allowed("settings_get_config"));
        assert!(is_ipc_kind_allowed("panel_focus"));
        assert!(is_ipc_kind_allowed("assistant_input"));
        assert!(is_ipc_kind_allowed("open_panel"));
        assert!(is_ipc_kind_allowed("panel_close"));
    }

    #[test]
    fn ipc_kind_rejected_unknown() {
        assert!(!is_ipc_kind_allowed("eval"));
        assert!(!is_ipc_kind_allowed("exec"));
        assert!(!is_ipc_kind_allowed(""));
        assert!(!is_ipc_kind_allowed("pty_input_extra"));
        assert!(!is_ipc_kind_allowed("PTY_INPUT")); // case-sensitive
    }

    #[test]
    fn ipc_kind_rejected_injection_attempts() {
        assert!(!is_ipc_kind_allowed("pty_input\0"));
        assert!(!is_ipc_kind_allowed("ping; rm -rf /"));
        assert!(!is_ipc_kind_allowed("<script>alert(1)</script>"));
    }
}
