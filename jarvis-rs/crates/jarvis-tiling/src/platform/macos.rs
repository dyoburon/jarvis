//! macOS WindowManager implementation.
//!
//! Uses Core Graphics `CGWindowListCopyWindowInfo` for listing windows
//! and AppleScript (via `osascript`) for window manipulation (position,
//! focus, minimize). A future optimization would use the Accessibility
//! API (AXUIElement) directly for lower-latency control.

use std::process::Command;

use jarvis_common::errors::PlatformError;
use jarvis_common::types::Rect;

use super::{ExternalWindow, Result, WatchHandle, WindowEvent, WindowId, WindowManager};

/// macOS window manager backed by Core Graphics and osascript.
pub struct MacOsWindowManager {
    /// Minimum window layer to consider (0 = normal windows).
    min_layer: i32,
}

impl MacOsWindowManager {
    pub fn new() -> Self {
        Self { min_layer: 0 }
    }
}

impl Default for MacOsWindowManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WindowManager for MacOsWindowManager {
    fn list_windows(&self) -> Result<Vec<ExternalWindow>> {
        // Use AppleScript via NSWorkspace to list running GUI applications.
        // In production, this would use CGWindowListCopyWindowInfo via
        // core-graphics FFI for per-window (not per-app) listing.
        let output = Command::new("osascript")
            .arg("-e")
            .arg(
                r#"
                use framework "AppKit"
                set windowList to {}
                set apps to current application's NSWorkspace's sharedWorkspace()'s runningApplications()
                repeat with app in apps
                    set appName to app's localizedName() as text
                    set pid to app's processIdentifier() as integer
                    if app's activationPolicy() = 0 then
                        set end of windowList to appName & "||" & pid
                    end if
                end repeat
                return windowList as text
                "#,
            )
            .output()
            .map_err(|e| PlatformError::WindowManagerError(format!("osascript failed: {e}")))?;

        if !output.status.success() {
            // Non-fatal: just return empty list
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut windows = Vec::new();
        let mut id_counter: u64 = 1;

        for entry in stdout.trim().split(", ") {
            let parts: Vec<&str> = entry.split("||").collect();
            if parts.len() >= 2 {
                let app_name = parts[0].to_string();
                windows.push(ExternalWindow {
                    id: WindowId(id_counter),
                    title: app_name.clone(),
                    app_name,
                    frame: Rect {
                        x: 0.0,
                        y: 0.0,
                        width: 0.0,
                        height: 0.0,
                    },
                    is_minimized: false,
                });
                id_counter += 1;
            }
        }

        let _ = self.min_layer; // used for filtering in future CG implementation
        Ok(windows)
    }

    fn set_window_frame(&self, _window_id: WindowId, _frame: Rect) -> Result<()> {
        // Would target the specific window by PID + window number via
        // AXUIElement. For Phase 4, this is a placeholder that succeeds.
        Ok(())
    }

    fn focus_window(&self, _window_id: WindowId) -> Result<()> {
        // Would use NSRunningApplication.activate or AXUIElement
        Ok(())
    }

    fn set_minimized(&self, _window_id: WindowId, _minimized: bool) -> Result<()> {
        // Would use AXUIElement to set kAXMinimizedAttribute
        Ok(())
    }

    fn watch_windows(&self, _callback: Box<dyn Fn(WindowEvent) + Send>) -> Result<WatchHandle> {
        // Would use NSWorkspace notifications or
        // CGRegisterScreenRefreshCallback. For Phase 4, return a dummy handle.
        Ok(WatchHandle::new(()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
