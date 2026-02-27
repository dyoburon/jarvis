//! X11 WindowManager implementation (stub).
//!
//! Will use x11rb (XCB bindings) for window management when implemented.
//! For now, delegates to NoopWindowManager.

use jarvis_common::types::Rect;

use super::{ExternalWindow, Result, WatchHandle, WindowEvent, WindowId, WindowManager};

/// X11-based window manager (stub).
pub struct X11WindowManager;

impl WindowManager for X11WindowManager {
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
