//! Command palette key handling and webview IPC bridge.

use jarvis_common::actions::Action;
use jarvis_platform::input_processor::InputMode;

use super::core::JarvisApp;

impl JarvisApp {
    /// Handle key events for the command palette.
    pub(super) fn handle_palette_key(&mut self, key_name: &str, is_press: bool) -> bool {
        if !is_press || !self.command_palette_open {
            return false;
        }

        let palette = match self.command_palette.as_mut() {
            Some(p) => p,
            None => return false,
        };

        match key_name {
            "Escape" => {
                self.dispatch(Action::CloseOverlay);
                true
            }
            "Enter" => {
                if let Some(action) = palette.confirm() {
                    self.send_palette_hide();
                    self.command_palette_open = false;
                    self.command_palette = None;
                    self.input.set_mode(InputMode::Terminal);
                    self.dispatch(action);
                }
                true
            }
            "Up" => {
                palette.select_prev();
                self.send_palette_update();
                true
            }
            "Down" => {
                palette.select_next();
                self.send_palette_update();
                true
            }
            "Backspace" => {
                palette.backspace();
                self.send_palette_update();
                true
            }
            "Tab" => {
                palette.select_next();
                self.send_palette_update();
                true
            }
            _ => {
                if key_name.len() == 1 {
                    let ch = key_name.chars().next().unwrap();
                    if ch.is_ascii_graphic() || ch == ' ' {
                        palette.append_char(ch.to_ascii_lowercase());
                        self.send_palette_update();
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Send palette state to the focused webview.
    pub(super) fn send_palette_to_webview(&self, kind: &str) {
        let focused = self.tiling.focused_id();
        if let Some(ref registry) = self.webviews {
            if let Some(handle) = registry.get(focused) {
                if let Some(ref palette) = self.command_palette {
                    let items: Vec<_> = palette
                        .visible_items()
                        .iter()
                        .map(|item| {
                            serde_json::json!({
                                "label": item.label,
                                "keybind": item.keybind_display
                            })
                        })
                        .collect();
                    let payload = serde_json::json!({
                        "items": items,
                        "query": palette.query(),
                        "selectedIndex": palette.selected_index()
                    });
                    let _ = handle.send_ipc(kind, &payload);
                }
            }
        }
    }

    /// Send palette_hide to the focused webview.
    pub(super) fn send_palette_hide(&self) {
        let focused = self.tiling.focused_id();
        if let Some(ref registry) = self.webviews {
            if let Some(handle) = registry.get(focused) {
                let _ = handle.send_ipc("palette_hide", &serde_json::json!({}));
            }
        }
    }

    /// Convenience: send palette_update with current state.
    fn send_palette_update(&self) {
        self.send_palette_to_webview("palette_update");
    }
}
