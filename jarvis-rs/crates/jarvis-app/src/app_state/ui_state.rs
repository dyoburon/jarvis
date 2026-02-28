//! UI chrome state updates: status bar, tab bar, redraw requests.

use jarvis_renderer::Tab;

use super::core::JarvisApp;

impl JarvisApp {
    /// Update UI chrome state (status bar, tab bar) from current app state.
    pub(super) fn update_chrome(&mut self) {
        // Status bar
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

        // Tab bar -- build from pane IDs sorted
        let focused = self.tiling.focused_id();
        let mut pane_ids: Vec<u32> = self.panes.keys().copied().collect();
        pane_ids.sort();
        let tabs: Vec<Tab> = pane_ids
            .iter()
            .map(|&id| Tab {
                title: format!("Terminal {id}"),
                is_active: id == focused,
            })
            .collect();
        let active_idx = tabs.iter().position(|t| t.is_active).unwrap_or(0);
        self.chrome.set_tabs(tabs, active_idx);
    }

    /// Request a window redraw.
    pub(super) fn request_redraw(&self) {
        if let Some(ref w) = self.window {
            w.request_redraw();
        }
    }
}
