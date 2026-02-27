//! Wayland WindowManager implementation (stub).
//!
//! Will use wlr-foreign-toplevel-management protocol when implemented.
//! For now, delegates to NoopWindowManager.

use jarvis_common::types::Rect;

use super::{ExternalWindow, Result, WatchHandle, WindowEvent, WindowId, WindowManager};

/// Wayland-based window manager (stub).
pub struct WaylandWindowManager;

impl WindowManager for WaylandWindowManager {
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
