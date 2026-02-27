//! No-op WindowManager implementation.
//!
//! Used as a fallback on platforms where external window management is not
//! yet implemented, or for testing.

use jarvis_common::types::Rect;

use super::{ExternalWindow, Result, WatchHandle, WindowEvent, WindowId, WindowManager};

/// A no-op window manager that does nothing. All queries return empty
/// results and all mutations succeed silently.
pub struct NoopWindowManager;

impl WindowManager for NoopWindowManager {
    fn list_windows(&self) -> Result<Vec<ExternalWindow>> {
        Ok(Vec::new())
    }

    fn set_window_frame(&self, _window_id: WindowId, _frame: Rect) -> Result<()> {
        Ok(())
    }

    fn focus_window(&self, _window_id: WindowId) -> Result<()> {
        Ok(())
    }

    fn set_minimized(&self, _window_id: WindowId, _minimized: bool) -> Result<()> {
        Ok(())
    }

    fn watch_windows(
        &self,
        _callback: Box<dyn Fn(WindowEvent) + Send>,
    ) -> Result<WatchHandle> {
        Ok(WatchHandle::new(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_empty() {
        let wm = NoopWindowManager;
        let windows = wm.list_windows().unwrap();
        assert!(windows.is_empty());
    }

    #[test]
    fn set_frame_succeeds() {
        let wm = NoopWindowManager;
        let r = wm.set_window_frame(
            WindowId(1),
            Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
        );
        assert!(r.is_ok());
    }

    #[test]
    fn focus_succeeds() {
        let wm = NoopWindowManager;
        assert!(wm.focus_window(WindowId(1)).is_ok());
    }

    #[test]
    fn minimize_succeeds() {
        let wm = NoopWindowManager;
        assert!(wm.set_minimized(WindowId(1), true).is_ok());
    }

    #[test]
    fn watch_succeeds() {
        let wm = NoopWindowManager;
        let handle = wm.watch_windows(Box::new(|_| {}));
        assert!(handle.is_ok());
    }
}
