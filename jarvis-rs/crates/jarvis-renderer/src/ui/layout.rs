use jarvis_common::types::Rect;

use super::chrome::UiChrome;

impl UiChrome {
    /// Compute the rectangle available for terminal content after subtracting
    /// chrome elements (tab bar, status bar).
    pub fn content_rect(&self, window_width: f32, window_height: f32) -> Rect {
        let top = self.tab_bar.as_ref().map(|tb| tb.height).unwrap_or(0.0);
        let bottom = self.status_bar.as_ref().map(|sb| sb.height).unwrap_or(0.0);
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
