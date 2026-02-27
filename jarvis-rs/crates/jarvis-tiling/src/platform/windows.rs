//! Windows WindowManager implementation (stub).
//!
//! Will use Win32 API (SetWindowPos, EnumWindows) when implemented.
//! For now, delegates to NoopWindowManager.

use jarvis_common::types::Rect;

use super::{ExternalWindow, Result, WatchHandle, WindowEvent, WindowId, WindowManager};

/// Win32-based window manager (stub).
pub struct Win32WindowManager;

impl WindowManager for Win32WindowManager {
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

    fn watch_windows(&self, _callback: Box<dyn Fn(WindowEvent) + Send>) -> Result<WatchHandle> {
        Ok(WatchHandle::new(()))
    }
}
