//! UI chrome elements surrounding the terminal content.
//!
//! Defines borders, tab bar, status bar, and pane dividers that frame the
//! terminal grid. Layout calculations determine the content area available
//! after subtracting chrome elements.

use jarvis_common::types::Rect;
use jarvis_config::schema::LayoutConfig;

/// Default tab bar height in pixels.
const DEFAULT_TAB_BAR_HEIGHT: f32 = 32.0;

/// Default status bar height in pixels.
const DEFAULT_STATUS_BAR_HEIGHT: f32 = 24.0;

/// A border drawn around a terminal pane.
#[derive(Debug, Clone)]
pub struct PaneBorder {
    /// Bounding rectangle of the border.
    pub rect: Rect,
    /// Border color as RGBA (each component 0.0..=1.0).
    pub color: [f32; 4],
    /// Border line width in pixels.
    pub width: f32,
    /// Whether this pane currently has keyboard focus.
    pub is_focused: bool,
}

/// A single tab in the tab bar.
#[derive(Debug, Clone)]
pub struct Tab {
    /// Display title for the tab.
    pub title: String,
    /// Whether this tab is the currently active one.
    pub is_active: bool,
}

/// The tab bar shown at the top of the window.
#[derive(Debug, Clone)]
pub struct TabBar {
    /// All tabs in order.
    pub tabs: Vec<Tab>,
    /// Index of the active tab.
    pub active_tab: usize,
    /// Height of the tab bar in pixels.
    pub height: f32,
}

/// The status bar shown at the bottom of the window.
#[derive(Debug, Clone)]
pub struct StatusBar {
    /// Text aligned to the left.
    pub left_text: String,
    /// Text aligned to the center.
    pub center_text: String,
    /// Text aligned to the right.
    pub right_text: String,
    /// Height of the status bar in pixels.
    pub height: f32,
    /// Background color as RGBA.
    pub bg_color: [f32; 4],
    /// Foreground (text) color as RGBA.
    pub fg_color: [f32; 4],
}

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

    /// Compute the rectangle available for terminal content after subtracting
    /// chrome elements (tab bar, status bar).
    pub fn content_rect(&self, window_width: f32, window_height: f32) -> Rect {
        let top = self
            .tab_bar
            .as_ref()
            .map(|tb| tb.height)
            .unwrap_or(0.0);
        let bottom = self
            .status_bar
            .as_ref()
            .map(|sb| sb.height)
            .unwrap_or(0.0);
        Rect {
            x: 0.0,
            y: top as f64,
            width: window_width as f64,
            height: (window_height - top - bottom).max(0.0) as f64,
        }
    }

    /// Compute the rectangle for the tab bar, if present.
    pub fn tab_bar_rect(&self, window_width: f32) -> Option<Rect> {
        self.tab_bar.as_ref().map(|tb| Rect {
            x: 0.0,
            y: 0.0,
            width: window_width as f64,
            height: tb.height as f64,
        })
    }

    /// Compute the rectangle for the status bar, if present.
    pub fn status_bar_rect(&self, window_width: f32, window_height: f32) -> Option<Rect> {
        self.status_bar.as_ref().map(|sb| Rect {
            x: 0.0,
            y: (window_height - sb.height) as f64,
            width: window_width as f64,
            height: sb.height as f64,
        })
    }
}

impl Default for UiChrome {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn content_rect_subtracts_tab_and_status_bar() {
        let mut chrome = UiChrome::new();
        chrome.set_tabs(
            vec![
                Tab {
                    title: "Tab 1".into(),
                    is_active: true,
                },
            ],
            0,
        );
        chrome.set_status("left", "center", "right");

        let rect = chrome.content_rect(1920.0, 1080.0);
        // Tab bar = 32px, status bar = 24px, content = 1080 - 32 - 24 = 1024
        assert!((rect.y - 32.0).abs() < 1e-3);
        assert!((rect.height - 1024.0).abs() < 1e-3);
        assert!((rect.width - 1920.0).abs() < 1e-3);
        assert!((rect.x - 0.0).abs() < 1e-3);
    }

    #[test]
    fn content_rect_no_tab_bar_uses_full_top() {
        let mut chrome = UiChrome::new();
        chrome.set_status("left", "", "");

        let rect = chrome.content_rect(800.0, 600.0);
        // No tab bar, status bar = 24px, content = 600 - 0 - 24 = 576
        assert!((rect.y - 0.0).abs() < 1e-3);
        assert!((rect.height - 576.0).abs() < 1e-3);
    }

    #[test]
    fn content_rect_no_chrome_uses_full_window() {
        let chrome = UiChrome::new();
        let rect = chrome.content_rect(1920.0, 1080.0);
        assert!((rect.y - 0.0).abs() < 1e-3);
        assert!((rect.height - 1080.0).abs() < 1e-3);
        assert!((rect.width - 1920.0).abs() < 1e-3);
    }

    #[test]
    fn content_rect_clamps_to_zero_when_too_small() {
        let mut chrome = UiChrome::new();
        chrome.tab_bar = Some(TabBar {
            tabs: vec![],
            active_tab: 0,
            height: 500.0,
        });
        chrome.status_bar = Some(StatusBar {
            left_text: String::new(),
            center_text: String::new(),
            right_text: String::new(),
            height: 500.0,
            bg_color: [0.0; 4],
            fg_color: [1.0; 4],
        });
        let rect = chrome.content_rect(100.0, 100.0);
        assert!(rect.height >= 0.0);
    }

    #[test]
    fn from_config_sets_gap() {
        let config = LayoutConfig {
            panel_gap: 8,
            ..Default::default()
        };
        let chrome = UiChrome::from_config(&config);
        assert!((chrome.pane_gap - 8.0).abs() < 1e-3);
    }

    #[test]
    fn from_config_default_gap() {
        let config = LayoutConfig::default();
        let chrome = UiChrome::from_config(&config);
        assert!((chrome.pane_gap - 2.0).abs() < 1e-3);
    }

    #[test]
    fn set_tabs_marks_active_tab() {
        let mut chrome = UiChrome::new();
        chrome.set_tabs(
            vec![
                Tab {
                    title: "A".into(),
                    is_active: false,
                },
                Tab {
                    title: "B".into(),
                    is_active: false,
                },
                Tab {
                    title: "C".into(),
                    is_active: false,
                },
            ],
            1,
        );
        let tb = chrome.tab_bar.as_ref().unwrap();
        assert_eq!(tb.active_tab, 1);
        assert!(!tb.tabs[0].is_active);
        assert!(tb.tabs[1].is_active);
        assert!(!tb.tabs[2].is_active);
    }

    #[test]
    fn set_tabs_clamps_active_index() {
        let mut chrome = UiChrome::new();
        chrome.set_tabs(
            vec![
                Tab {
                    title: "Only".into(),
                    is_active: false,
                },
            ],
            99,
        );
        let tb = chrome.tab_bar.as_ref().unwrap();
        assert_eq!(tb.active_tab, 0);
        assert!(tb.tabs[0].is_active);
    }

    #[test]
    fn set_status_creates_bar_if_absent() {
        let mut chrome = UiChrome::new();
        assert!(chrome.status_bar.is_none());
        chrome.set_status("L", "C", "R");
        let sb = chrome.status_bar.as_ref().unwrap();
        assert_eq!(sb.left_text, "L");
        assert_eq!(sb.center_text, "C");
        assert_eq!(sb.right_text, "R");
    }

    #[test]
    fn set_status_updates_existing_bar() {
        let mut chrome = UiChrome::new();
        chrome.set_status("first", "", "");
        chrome.set_status("second", "mid", "end");
        let sb = chrome.status_bar.as_ref().unwrap();
        assert_eq!(sb.left_text, "second");
        assert_eq!(sb.center_text, "mid");
        assert_eq!(sb.right_text, "end");
    }

    #[test]
    fn tab_bar_rect_none_when_no_tab_bar() {
        let chrome = UiChrome::new();
        assert!(chrome.tab_bar_rect(1920.0).is_none());
    }

    #[test]
    fn tab_bar_rect_correct_dimensions() {
        let mut chrome = UiChrome::new();
        chrome.set_tabs(
            vec![Tab {
                title: "T".into(),
                is_active: true,
            }],
            0,
        );
        let rect = chrome.tab_bar_rect(1920.0).unwrap();
        assert!((rect.x - 0.0).abs() < 1e-3);
        assert!((rect.y - 0.0).abs() < 1e-3);
        assert!((rect.width - 1920.0).abs() < 1e-3);
        assert!((rect.height - 32.0).abs() < 1e-3);
    }

    #[test]
    fn status_bar_rect_none_when_no_status_bar() {
        let chrome = UiChrome::new();
        assert!(chrome.status_bar_rect(1920.0, 1080.0).is_none());
    }

    #[test]
    fn status_bar_rect_correct_dimensions() {
        let mut chrome = UiChrome::new();
        chrome.set_status("", "", "");
        let rect = chrome.status_bar_rect(1920.0, 1080.0).unwrap();
        assert!((rect.x - 0.0).abs() < 1e-3);
        assert!((rect.y - 1056.0).abs() < 1e-3); // 1080 - 24
        assert!((rect.width - 1920.0).abs() < 1e-3);
        assert!((rect.height - 24.0).abs() < 1e-3);
    }

    #[test]
    fn set_borders_replaces_all() {
        let mut chrome = UiChrome::new();
        assert!(chrome.borders.is_empty());
        chrome.set_borders(vec![
            PaneBorder {
                rect: Rect {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                },
                color: [1.0, 0.0, 0.0, 1.0],
                width: 1.0,
                is_focused: true,
            },
            PaneBorder {
                rect: Rect {
                    x: 100.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                },
                color: [0.0, 1.0, 0.0, 1.0],
                width: 1.0,
                is_focused: false,
            },
        ]);
        assert_eq!(chrome.borders.len(), 2);
        assert!(chrome.borders[0].is_focused);
        assert!(!chrome.borders[1].is_focused);
    }
}
