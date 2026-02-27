//! UI chrome elements surrounding the terminal content.
//!
//! Defines borders, tab bar, status bar, and pane dividers that frame the
//! terminal grid. Layout calculations determine the content area available
//! after subtracting chrome elements.

mod chrome;
mod layout;
mod types;

pub use chrome::*;
pub use types::{PaneBorder, StatusBar, Tab, TabBar};

#[cfg(test)]
mod tests {
    use super::*;
    use jarvis_common::types::Rect;
    use jarvis_config::schema::LayoutConfig;

    #[test]
    fn content_rect_subtracts_tab_and_status_bar() {
        let mut chrome = UiChrome::new();
        chrome.set_tabs(
            vec![Tab {
                title: "Tab 1".into(),
                is_active: true,
            }],
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
            vec![Tab {
                title: "Only".into(),
                is_active: false,
            }],
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
