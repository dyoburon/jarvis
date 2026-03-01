//! Assistant panel IPC handlers.
//!
//! Handles `assistant_input` (user text from webview) and
//! `open_panel` (request to open a new panel type).

use jarvis_common::types::PaneKind;
use jarvis_tiling::tree::Direction;
use jarvis_webview::IpcPayload;

use crate::app_state::core::JarvisApp;

// =============================================================================
// CONSTANTS
// =============================================================================

/// Maximum length for assistant input text (prevents abuse).
const MAX_INPUT_LEN: usize = 4096;

/// Allowed panel names for the `open_panel` IPC command.
const ALLOWED_PANELS: &[&str] = &["terminal", "assistant", "chat", "settings", "presence"];

/// Allowed game names for the `launch_game` IPC command.
const ALLOWED_GAMES: &[&str] = &[
    "tetris",
    "asteroids",
    "minesweeper",
    "pinball",
    "doodlejump",
    "draw",
    "subway",
    "videoplayer",
];

// =============================================================================
// IPC HANDLERS
// =============================================================================

impl JarvisApp {
    /// Handle `assistant_input` — user typed text in the assistant webview.
    ///
    /// Forwards the input text to the AI assistant runtime channel.
    pub(in crate::app_state) fn handle_assistant_input(
        &mut self,
        pane_id: u32,
        payload: &IpcPayload,
    ) {
        let text = match payload {
            IpcPayload::Json(obj) => obj.get("text").and_then(|v| v.as_str()),
            _ => None,
        };

        let text = match text {
            Some(t) if !t.is_empty() && t.len() <= MAX_INPUT_LEN => t,
            Some(t) if t.len() > MAX_INPUT_LEN => {
                tracing::warn!(pane_id, len = t.len(), "assistant_input: text too long");
                return;
            }
            _ => {
                tracing::warn!(pane_id, "assistant_input: missing or empty text");
                return;
            }
        };

        tracing::debug!(pane_id, len = text.len(), "Assistant input received");

        // Forward to the assistant runtime channel
        if let Some(ref tx) = self.assistant_tx {
            if let Err(e) = tx.send(text.to_string()) {
                tracing::warn!(pane_id, error = %e, "Failed to send assistant input");
            }
        } else {
            // Lazily start the assistant runtime
            self.ensure_assistant_runtime();
            if let Some(ref tx) = self.assistant_tx {
                let _ = tx.send(text.to_string());
            }
        }
    }

    /// Handle `assistant_ready` — the assistant webview has loaded and registered IPC handlers.
    ///
    /// Starts the async Claude AI runtime so it can send back config and accept messages.
    pub(in crate::app_state) fn handle_assistant_ready(&mut self, pane_id: u32) {
        tracing::debug!(pane_id, "Assistant panel ready");
        self.ensure_assistant_runtime();
    }

    /// Handle `launch_game` — launch a fullscreen game in the requesting panel.
    ///
    /// The payload must contain `{ "game": "tetris" | "asteroids" | ... }`.
    pub(in crate::app_state) fn handle_launch_game(
        &mut self,
        pane_id: u32,
        payload: &IpcPayload,
    ) {
        let game_name = match payload {
            IpcPayload::Json(obj) => obj.get("game").and_then(|v| v.as_str()),
            _ => None,
        };

        let game_name = match game_name {
            Some(name) if ALLOWED_GAMES.contains(&name) => name,
            Some(name) => {
                tracing::warn!(pane_id, game = %name, "launch_game: unknown game name");
                return;
            }
            None => {
                tracing::warn!(pane_id, "launch_game: missing game name");
                return;
            }
        };

        let url = format!("jarvis://localhost/games/{}.html", game_name);

        if let Some(ref mut registry) = self.webviews {
            if let Some(handle) = registry.get_mut(pane_id) {
                let original_url = handle.current_url().to_string();
                if let Err(e) = handle.load_url(&url) {
                    tracing::warn!(pane_id, error = %e, "Failed to launch game");
                } else {
                    tracing::info!(pane_id, game = %game_name, "Game launched");
                    self.game_active = Some((pane_id, original_url));
                }
            }
        }
    }

    /// Handle `open_panel` — open a new tiling pane with the requested panel.
    ///
    /// The payload must contain `{ "panel": "terminal" | "chat" | ... }`.
    pub(in crate::app_state) fn handle_open_panel(&mut self, pane_id: u32, payload: &IpcPayload) {
        let panel_name = match payload {
            IpcPayload::Json(obj) => obj.get("panel").and_then(|v| v.as_str()),
            _ => None,
        };

        let panel_name = match panel_name {
            Some(name) if is_panel_allowed(name) => name,
            Some(name) => {
                tracing::warn!(
                    pane_id,
                    panel = %name,
                    "open_panel: unknown panel name"
                );
                return;
            }
            None => {
                tracing::warn!(pane_id, "open_panel: missing panel name");
                return;
            }
        };

        let kind = panel_kind_from_name(panel_name);
        let title = panel_title(kind);
        let url = panel_url_from_name(panel_name);

        // Split the focused pane to create a new pane with the requested type
        if let Some(new_id) = self.tiling.split_with(Direction::Horizontal, kind, title) {
            self.create_webview_for_pane_with_url(new_id, url);
            self.sync_webview_bounds();
            self.needs_redraw = true;
            tracing::info!(pane_id, new_id, panel = %panel_name, "Panel opened");
        }
    }
}

// =============================================================================
// HELPERS
// =============================================================================

/// Check whether a panel name is in the allowlist.
fn is_panel_allowed(name: &str) -> bool {
    ALLOWED_PANELS.contains(&name)
}

/// Map a panel name string to `PaneKind`.
fn panel_kind_from_name(name: &str) -> PaneKind {
    match name {
        "terminal" => PaneKind::Terminal,
        "assistant" => PaneKind::Assistant,
        "chat" => PaneKind::Chat,
        "settings" | "presence" => PaneKind::WebView,
        _ => PaneKind::WebView,
    }
}

/// Map a panel name to its `jarvis://` URL.
///
/// This handles special panels (settings, presence) that don't have
/// their own `PaneKind` variant but do have dedicated HTML files.
fn panel_url_from_name(name: &str) -> &'static str {
    match name {
        "terminal" => "jarvis://localhost/terminal/index.html",
        "assistant" => "jarvis://localhost/assistant/index.html",
        "chat" => "jarvis://localhost/chat/index.html",
        "settings" => "jarvis://localhost/settings/index.html",
        "presence" => "jarvis://localhost/presence/index.html",
        _ => "jarvis://localhost/terminal/index.html",
    }
}

/// Default title for a panel kind.
fn panel_title(kind: PaneKind) -> &'static str {
    match kind {
        PaneKind::Terminal => "Terminal",
        PaneKind::Assistant => "Assistant",
        PaneKind::Chat => "Chat",
        PaneKind::WebView => "WebView",
        PaneKind::ExternalApp => "External",
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panel_allowed_valid_names() {
        assert!(is_panel_allowed("terminal"));
        assert!(is_panel_allowed("assistant"));
        assert!(is_panel_allowed("chat"));
        assert!(is_panel_allowed("settings"));
        assert!(is_panel_allowed("presence"));
    }

    #[test]
    fn panel_rejected_unknown_names() {
        assert!(!is_panel_allowed(""));
        assert!(!is_panel_allowed("eval"));
        assert!(!is_panel_allowed("Terminal")); // case-sensitive
        assert!(!is_panel_allowed("game"));
    }

    #[test]
    fn panel_rejected_injection_attempts() {
        assert!(!is_panel_allowed("terminal; rm -rf /"));
        assert!(!is_panel_allowed("<script>alert(1)</script>"));
        assert!(!is_panel_allowed("../../etc/passwd"));
    }

    #[test]
    fn panel_kind_mapping() {
        assert_eq!(panel_kind_from_name("terminal"), PaneKind::Terminal);
        assert_eq!(panel_kind_from_name("assistant"), PaneKind::Assistant);
        assert_eq!(panel_kind_from_name("chat"), PaneKind::Chat);
        // Settings/presence map to WebView
        assert_eq!(panel_kind_from_name("settings"), PaneKind::WebView);
        assert_eq!(panel_kind_from_name("presence"), PaneKind::WebView);
    }

    #[test]
    fn panel_url_from_name_maps_all_panels() {
        assert_eq!(
            panel_url_from_name("terminal"),
            "jarvis://localhost/terminal/index.html"
        );
        assert_eq!(
            panel_url_from_name("assistant"),
            "jarvis://localhost/assistant/index.html"
        );
        assert_eq!(
            panel_url_from_name("chat"),
            "jarvis://localhost/chat/index.html"
        );
        assert_eq!(
            panel_url_from_name("settings"),
            "jarvis://localhost/settings/index.html"
        );
        assert_eq!(
            panel_url_from_name("presence"),
            "jarvis://localhost/presence/index.html"
        );
    }

    #[test]
    fn panel_url_from_name_unknown_falls_back() {
        assert_eq!(
            panel_url_from_name("unknown"),
            "jarvis://localhost/terminal/index.html"
        );
    }

    #[test]
    fn panel_title_for_all_kinds() {
        assert_eq!(panel_title(PaneKind::Terminal), "Terminal");
        assert_eq!(panel_title(PaneKind::Assistant), "Assistant");
        assert_eq!(panel_title(PaneKind::Chat), "Chat");
        assert_eq!(panel_title(PaneKind::WebView), "WebView");
        assert_eq!(panel_title(PaneKind::ExternalApp), "External");
    }
}
