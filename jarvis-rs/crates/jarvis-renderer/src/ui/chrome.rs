use jarvis_config::schema::LayoutConfig;

use super::types::{
    PaneBorder, StatusBar, Tab, TabBar, DEFAULT_STATUS_BAR_HEIGHT, DEFAULT_TAB_BAR_HEIGHT,
};

/// All UI chrome elements that surround the terminal content area.
pub struct UiChrome {
    /// Optional tab bar at the top of the window.
    pub tab_bar: Option<TabBar>,
    /// Optional status bar at the bottom of the window.
    pub status_bar: Option<StatusBar>,
    /// Borders around individual panes.
    pub borders: Vec<PaneBorder>,
    /// Gap between adjacent panes in pixels.
    pub pane_gap: f32,
}

impl UiChrome {
    /// Create a new `UiChrome` with no tab bar, no status bar, and default gap.
    pub fn new() -> Self {
        Self {
            tab_bar: None,
            status_bar: None,
            borders: Vec::new(),
            pane_gap: 2.0,
        }
    }

    /// Create `UiChrome` from the layout configuration.
    pub fn from_config(config: &LayoutConfig) -> Self {
        Self {
            tab_bar: None,
            status_bar: None,
            borders: Vec::new(),
            pane_gap: config.panel_gap as f32,
        }
    }

    /// Set the tabs displayed in the tab bar.
    ///
    /// Creates the tab bar if it does not yet exist. Each tab's `is_active`
    /// field is set based on the `active` index.
    pub fn set_tabs(&mut self, mut tabs: Vec<Tab>, active: usize) {
        let active = active.min(tabs.len().saturating_sub(1));
        for (i, tab) in tabs.iter_mut().enumerate() {
            tab.is_active = i == active;
        }
        self.tab_bar = Some(TabBar {
            tabs,
            active_tab: active,
            height: DEFAULT_TAB_BAR_HEIGHT,
        });
    }

    /// Update the status bar text fields.
    ///
    /// Creates the status bar with default styling if it does not yet exist.
    pub fn set_status(&mut self, left: &str, center: &str, right: &str) {
        if let Some(ref mut bar) = self.status_bar {
            bar.left_text = left.to_owned();
            bar.center_text = center.to_owned();
            bar.right_text = right.to_owned();
        } else {
            self.status_bar = Some(StatusBar {
                left_text: left.to_owned(),
                center_text: center.to_owned(),
                right_text: right.to_owned(),
                height: DEFAULT_STATUS_BAR_HEIGHT,
                bg_color: [0.1, 0.1, 0.1, 0.9],
                fg_color: [0.9, 0.9, 0.9, 1.0],
            });
        }
    }

    /// Replace all pane borders.
    pub fn set_borders(&mut self, borders: Vec<PaneBorder>) {
        self.borders = borders;
    }
}

impl Default for UiChrome {
    fn default() -> Self {
        Self::new()
    }
}
