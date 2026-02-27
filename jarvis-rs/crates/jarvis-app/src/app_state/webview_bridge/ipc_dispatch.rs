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
            // Presence messages will be handled in Phase 6
            "presence_request_users" | "presence_poke" => {
                tracing::debug!(
                    pane_id,
                    kind = %msg.kind,
                    "Presence IPC: will be handled in Phase 6"
                );
            }
            // Settings messages will be handled in Phase 5
            "settings_init" | "settings_set_theme" => {
                tracing::debug!(
                    pane_id,
                    kind = %msg.kind,
                    "Settings IPC: will be handled in Phase 5"
                );
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
        assert!(is_ipc_kind_allowed("panel_focus"));
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
