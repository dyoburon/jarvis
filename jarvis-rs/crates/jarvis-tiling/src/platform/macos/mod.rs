//! macOS WindowManager implementation.
//!
//! Uses Core Graphics `CGWindowListCopyWindowInfo` for listing windows
//! and AppleScript (via `osascript`) for window manipulation (position,
//! focus, minimize). A future optimization would use the Accessibility
//! API (AXUIElement) directly for lower-latency control.

mod types;
mod window_management;

pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::{WindowId, WindowManager};
    use jarvis_common::types::Rect;

    #[test]
    fn new_manager() {
        let wm = MacOsWindowManager::new();
        assert_eq!(wm.min_layer, 0);
    }

    #[test]
    fn default_impl() {
        let wm = MacOsWindowManager::default();
        assert_eq!(wm.min_layer, 0);
    }

    #[test]
    fn set_frame_is_ok() {
        let wm = MacOsWindowManager::new();
        let result = wm.set_window_frame(
            WindowId(1),
            Rect {
                x: 100.0,
                y: 100.0,
                width: 800.0,
                height: 600.0,
            },
        );
        assert!(result.is_ok());
    }

    #[test]
    fn focus_is_ok() {
        let wm = MacOsWindowManager::new();
        assert!(wm.focus_window(WindowId(1)).is_ok());
    }

    #[test]
    fn minimize_is_ok() {
        let wm = MacOsWindowManager::new();
        assert!(wm.set_minimized(WindowId(1), true).is_ok());
    }

    #[test]
    fn watch_returns_handle() {
        let wm = MacOsWindowManager::new();
        let handle = wm.watch_windows(Box::new(|_| {}));
        assert!(handle.is_ok());
    }
}
