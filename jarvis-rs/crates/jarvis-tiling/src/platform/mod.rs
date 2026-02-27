use jarvis_common::errors::PlatformError;
use jarvis_common::types::Rect;
use serde::{Deserialize, Serialize};

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "linux")]
pub mod wayland;
#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "linux")]
pub mod x11;

pub mod noop;

pub type Result<T> = std::result::Result<T, PlatformError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WindowId(pub u64);

#[derive(Debug, Clone)]
pub struct ExternalWindow {
    pub id: WindowId,
    pub title: String,
    pub app_name: String,
    pub frame: Rect,
    pub is_minimized: bool,
}

#[derive(Debug, Clone)]
pub enum WindowEvent {
    Created(WindowId),
    Destroyed(WindowId),
    Moved(WindowId, Rect),
    Resized(WindowId, Rect),
    FocusChanged(WindowId),
}

pub struct WatchHandle {
    _inner: Box<dyn std::any::Any + Send>,
}

impl WatchHandle {
    pub fn new(inner: impl std::any::Any + Send + 'static) -> Self {
        Self {
            _inner: Box::new(inner),
        }
    }
}

/// Platform-agnostic trait for controlling external application windows.
pub trait WindowManager: Send + Sync {
    fn list_windows(&self) -> Result<Vec<ExternalWindow>>;
    fn set_window_frame(&self, window_id: WindowId, frame: Rect) -> Result<()>;
    fn focus_window(&self, window_id: WindowId) -> Result<()>;
    fn set_minimized(&self, window_id: WindowId, minimized: bool) -> Result<()>;
    fn watch_windows(&self, callback: Box<dyn Fn(WindowEvent) + Send>) -> Result<WatchHandle>;
}

/// Create the platform-appropriate WindowManager.
///
/// On macOS: returns a CoreGraphics-based implementation.
/// On other platforms: returns a no-op implementation.
pub fn create_window_manager() -> Box<dyn WindowManager> {
    #[cfg(target_os = "macos")]
    {
        Box::new(macos::MacOsWindowManager::new())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Box::new(noop::NoopWindowManager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_id_equality() {
        assert_eq!(WindowId(1), WindowId(1));
        assert_ne!(WindowId(1), WindowId(2));
    }

    #[test]
    fn window_id_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(WindowId(1));
        set.insert(WindowId(2));
        set.insert(WindowId(1));
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn window_id_serialization() {
        let id = WindowId(42);
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: WindowId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn external_window_construction() {
        let win = ExternalWindow {
            id: WindowId(1),
            title: "Test Window".to_string(),
            app_name: "TestApp".to_string(),
            frame: Rect {
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
            },
            is_minimized: false,
        };
        assert_eq!(win.id, WindowId(1));
        assert_eq!(win.title, "Test Window");
        assert!(!win.is_minimized);
    }

    #[test]
    fn watch_handle_creation() {
        let _handle = WatchHandle::new(42u32);
    }

    #[test]
    fn create_window_manager_returns_impl() {
        let wm = create_window_manager();
        // Should be able to list windows (even if empty on noop)
        let result = wm.list_windows();
        assert!(result.is_ok());
    }

    #[test]
    fn noop_manager_returns_empty() {
        let wm = noop::NoopWindowManager;
        assert!(wm.list_windows().unwrap().is_empty());
    }
}
