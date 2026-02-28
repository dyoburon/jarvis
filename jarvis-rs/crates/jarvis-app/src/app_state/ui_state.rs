//! UI chrome state updates: status bar, tab bar, redraw requests.

use super::core::JarvisApp;

impl JarvisApp {
    /// Update UI chrome state (status bar) from current app state.
    pub(super) fn update_chrome(&mut self) {
        let focused_id = self.tiling.focused_id();
        let pane_count = self.tiling.pane_count();
        let left = format!("Jarvis v{}", env!("CARGO_PKG_VERSION"));
        let center = format!("Pane {} of {}", focused_id, pane_count);
        let right = if self.online_count > 0 {
            format!("[ {} online ]", self.online_count)
        } else {
            String::new()
        };
        self.chrome.set_status(&left, &center, &right);
    }

    /// Request a window redraw.
    pub(super) fn request_redraw(&self) {
        if let Some(ref w) = self.window {
            w.request_redraw();
        }
    }
}
