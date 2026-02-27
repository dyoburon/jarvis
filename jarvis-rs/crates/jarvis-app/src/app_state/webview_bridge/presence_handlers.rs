//! Presence IPC handlers and webview forwarding.
//!
//! Handles `presence_request_users` and `presence_poke` from webviews,
//! and provides methods to forward presence state updates to all panels.

use jarvis_webview::IpcPayload;

use crate::app_state::core::JarvisApp;
use crate::app_state::types::PresenceCommand;

// =============================================================================
// IPC HANDLERS
// =============================================================================

impl JarvisApp {
    /// Handle `presence_request_users` — send the current user list.
    pub(in crate::app_state) fn handle_presence_request_users(
        &self,
        pane_id: u32,
        _payload: &IpcPayload,
    ) {
        let users: Vec<serde_json::Value> = self
            .online_users
            .iter()
            .map(|u| {
                serde_json::json!({
                    "user_id": u.user_id,
                    "display_name": sanitize_display_name(&u.display_name),
                    "status": u.status,
                    "activity": u.activity,
                })
            })
            .collect();

        let payload = serde_json::json!({ "users": users });

        if let Some(ref registry) = self.webviews {
            if let Some(handle) = registry.get(pane_id) {
                if let Err(e) = handle.send_ipc("presence_users", &payload) {
                    tracing::warn!(pane_id, error = %e, "Failed to send presence_users");
                }
            }
        }
    }

    /// Handle `presence_poke` — forward poke to the async presence client.
    pub(in crate::app_state) fn handle_presence_poke(&self, pane_id: u32, payload: &IpcPayload) {
        let target = match payload {
            IpcPayload::Json(obj) => obj.get("target_user_id").and_then(|v| v.as_str()),
            _ => None,
        };

        let target = match target {
            Some(id) if !id.is_empty() && id.len() <= 64 => id.to_string(),
            _ => {
                tracing::warn!(pane_id, "presence_poke: missing or invalid target_user_id");
                return;
            }
        };

        if let Some(ref tx) = self.presence_cmd_tx {
            if let Err(e) = tx.try_send(PresenceCommand::Poke {
                target_user_id: target,
            }) {
                tracing::warn!(pane_id, error = %e, "Failed to send poke command");
            }
        }
    }
}

// =============================================================================
// WEBVIEW FORWARDING
// =============================================================================

impl JarvisApp {
    /// Send the current online status line to all webview panels.
    pub(in crate::app_state) fn send_presence_status_to_webviews(&self) {
        let status = if self.online_count > 0 {
            format!("[ {} online ]", self.online_count)
        } else {
            "[ offline ]".to_string()
        };

        let payload = serde_json::json!({ "status": status });
        self.broadcast_ipc_to_all("presence_update", &payload);
    }

    /// Send the full user list to all webview panels.
    pub(in crate::app_state) fn send_presence_users_to_webviews(&self) {
        let users: Vec<serde_json::Value> = self
            .online_users
            .iter()
            .map(|u| {
                serde_json::json!({
                    "user_id": u.user_id,
                    "display_name": sanitize_display_name(&u.display_name),
                    "status": u.status,
                    "activity": u.activity,
                })
            })
            .collect();

        let payload = serde_json::json!({ "users": users });
        self.broadcast_ipc_to_all("presence_users", &payload);
    }

    /// Send a notification line to all webview panels.
    pub(in crate::app_state) fn send_presence_notification_to_webviews(&self, line: &str) {
        let payload = serde_json::json!({ "line": line });
        self.broadcast_ipc_to_all("presence_notification", &payload);
    }

    /// Broadcast an IPC message to all active webview panels.
    fn broadcast_ipc_to_all(&self, kind: &str, payload: &serde_json::Value) {
        if let Some(ref registry) = self.webviews {
            for pane_id in registry.active_panes() {
                if let Some(handle) = registry.get(pane_id) {
                    if let Err(e) = handle.send_ipc(kind, payload) {
                        tracing::warn!(pane_id, kind, error = %e, "Failed to broadcast IPC");
                    }
                }
            }
        }
    }
}

// =============================================================================
// SANITIZATION
// =============================================================================

/// Sanitize a display name for safe rendering in the UI.
///
/// - Truncates to 20 characters
/// - Keeps only alphanumeric, spaces, and dashes
/// - Falls back to "Unknown" if empty after sanitization
pub fn sanitize_display_name(name: &str) -> String {
    let sanitized: String = name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-')
        .take(20)
        .collect();

    let trimmed = sanitized.trim();
    if trimmed.is_empty() {
        "Unknown".to_string()
    } else {
        trimmed.to_string()
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_display_name_normal() {
        assert_eq!(sanitize_display_name("Agent-42"), "Agent-42");
        assert_eq!(sanitize_display_name("jarvis user"), "jarvis user");
    }

    #[test]
    fn sanitize_display_name_truncates_to_20() {
        let long = "a".repeat(30);
        let result = sanitize_display_name(&long);
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn sanitize_display_name_strips_special_chars() {
        assert_eq!(sanitize_display_name("user<script>"), "userscript");
        assert_eq!(sanitize_display_name("hello@world!"), "helloworld");
        assert_eq!(sanitize_display_name("test;drop table"), "testdrop table");
    }

    #[test]
    fn sanitize_display_name_empty_fallback() {
        assert_eq!(sanitize_display_name(""), "Unknown");
        assert_eq!(sanitize_display_name("!!!"), "Unknown");
        assert_eq!(sanitize_display_name("   "), "Unknown");
    }

    #[test]
    fn sanitize_display_name_unicode() {
        // Unicode alphanumeric chars are kept
        assert_eq!(sanitize_display_name("日本語テスト"), "日本語テスト");
    }

    #[test]
    fn sanitize_display_name_trims_whitespace() {
        assert_eq!(sanitize_display_name("  hello  "), "hello");
    }

    #[test]
    fn sanitize_display_name_mixed_valid_invalid() {
        assert_eq!(sanitize_display_name("Agent <42> (cool)"), "Agent 42 cool");
    }
}
