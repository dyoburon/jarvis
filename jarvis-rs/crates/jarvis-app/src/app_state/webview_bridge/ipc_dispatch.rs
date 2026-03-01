//! IPC message validation and dispatch from webview to Rust handlers.

use jarvis_platform::input::KeyCombo;
use jarvis_webview::{IpcMessage, IpcPayload};

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
    "assistant_ready",
    "open_panel",
    "panel_close",
    "panel_toggle",
    "open_settings",
    "status_bar_init",
    "launch_game",
    "game_exit",
    "ping",
    "boot_complete",
    "crypto",
    "window_drag",
    "keybind",
    "read_file",
    "clipboard_paste",
    "open_url",
    "palette_click",
    "palette_hover",
    "debug_event",
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
            "boot_complete" => {
                self.handle_boot_complete();
            }
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
                self.notify_focus_changed();
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
            "assistant_ready" => {
                self.handle_assistant_ready(pane_id);
            }
            "open_panel" => {
                self.handle_open_panel(pane_id, &msg.payload);
            }
            "panel_close" => {
                self.handle_panel_close(pane_id);
            }
            "panel_toggle" => {
                self.handle_panel_toggle(pane_id, &msg.payload);
            }
            "open_settings" => {
                self.handle_open_settings(pane_id);
            }
            "launch_game" => {
                self.handle_launch_game(pane_id, &msg.payload);
            }
            "game_exit" => {
                tracing::debug!(pane_id, "Game exited");
            }
            "status_bar_init" => {
                self.handle_status_bar_init(pane_id);
            }
            "crypto" => {
                self.handle_crypto(pane_id, &msg.payload);
            }
            "window_drag" => {
                if let Some(ref w) = self.window {
                    let _ = w.drag_window();
                }
            }
            "keybind" => {
                self.handle_keybind_from_webview(pane_id, &msg.payload);
            }
            "read_file" => {
                self.handle_read_file(pane_id, &msg.payload);
            }
            "clipboard_paste" => {
                self.handle_clipboard_paste(pane_id, &msg.payload);
            }
            "open_url" => {
                if let IpcPayload::Json(ref v) = msg.payload {
                    if let Some(url) = v.get("url").and_then(|u| u.as_str()) {
                        let url_owned = url.to_string();
                        self.dispatch(jarvis_common::actions::Action::OpenURL(url_owned));
                    }
                }
            }
            "palette_click" => {
                if let IpcPayload::Json(ref v) = msg.payload {
                    if let Some(idx) = v.get("index").and_then(|i| i.as_u64()) {
                        if let Some(ref mut palette) = self.command_palette {
                            palette.set_selected(idx as usize);
                            if let Some(action) = palette.confirm() {
                                self.send_palette_hide();
                                self.command_palette_open = false;
                                self.command_palette = None;
                                self.input.set_mode(jarvis_platform::input_processor::InputMode::Terminal);
                                self.needs_redraw = true;
                                self.dispatch(action);
                            }
                        }
                    }
                }
            }
            "palette_hover" => {
                if let IpcPayload::Json(ref v) = msg.payload {
                    if let Some(idx) = v.get("index").and_then(|i| i.as_u64()) {
                        if let Some(ref mut palette) = self.command_palette {
                            palette.set_selected(idx as usize);
                            self.send_palette_to_webview("palette_update");
                            self.needs_redraw = true;
                        }
                    }
                }
            }
            "debug_event" => {
                if let IpcPayload::Json(ref v) = msg.payload {
                    tracing::info!(pane_id, event = %v, "[JS] webview event");
                }
            }
            _ => {
                // Shouldn't happen — allowlist checked above
                tracing::warn!(pane_id, kind = %msg.kind, "Unhandled IPC kind");
            }
        }
    }
}

impl JarvisApp {
    /// Handle a `keybind` IPC message — keyboard shortcut forwarded from webview JS.
    ///
    /// WKWebView captures Cmd+key before winit sees them, so the JS-side
    /// IPC init script intercepts these and sends them here.
    pub(in crate::app_state) fn handle_keybind_from_webview(
        &mut self,
        pane_id: u32,
        payload: &IpcPayload,
    ) {
        let obj = match payload {
            IpcPayload::Json(v) => v,
            _ => return,
        };

        let key = match obj.get("key").and_then(|v| v.as_str()) {
            Some(k) => k.to_string(),
            None => return,
        };

        let ctrl = obj.get("ctrl").and_then(|v| v.as_bool()).unwrap_or(false);
        let alt = obj.get("alt").and_then(|v| v.as_bool()).unwrap_or(false);
        let shift = obj.get("shift").and_then(|v| v.as_bool()).unwrap_or(false);
        let meta = obj.get("meta").and_then(|v| v.as_bool()).unwrap_or(false);

        // When command palette is open, route all keys to the palette handler.
        if self.command_palette_open {
            // Cmd+V pastes clipboard text into the palette search
            if meta && key.eq_ignore_ascii_case("v") {
                if let Ok(mut cb) = jarvis_platform::Clipboard::new() {
                    if let Ok(text) = cb.get_text() {
                        if let Some(ref mut palette) = self.command_palette {
                            for ch in text.chars() {
                                if ch.is_ascii_graphic() || ch == ' ' {
                                    palette.append_char(ch.to_ascii_lowercase());
                                }
                            }
                            self.send_palette_to_webview("palette_update");
                            self.needs_redraw = true;
                        }
                    }
                }
                return;
            }
            // Route regular keys (typing, Escape, Enter, arrows) to palette
            if self.handle_palette_key(&key, true) {
                self.needs_redraw = true;
                return;
            }
        }

        // When assistant overlay is open, route keys there
        if self.assistant_open {
            if self.handle_assistant_key(&key, true) {
                self.needs_redraw = true;
                return;
            }
        }

        let combo = KeyCombo::from_winit(ctrl, alt, shift, meta, key.clone());

        if let Some(action) = self.registry.lookup(&combo) {
            let action = action.clone();
            tracing::debug!(pane_id, key = %key, ?action, "Keybind from webview");
            self.dispatch(action);
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
        assert!(is_ipc_kind_allowed("panel_toggle"));
        assert!(is_ipc_kind_allowed("open_settings"));
        assert!(is_ipc_kind_allowed("status_bar_init"));
        assert!(is_ipc_kind_allowed("boot_complete"));
        assert!(is_ipc_kind_allowed("crypto"));
        assert!(is_ipc_kind_allowed("window_drag"));
        assert!(is_ipc_kind_allowed("read_file"));
        assert!(is_ipc_kind_allowed("open_url"));
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
