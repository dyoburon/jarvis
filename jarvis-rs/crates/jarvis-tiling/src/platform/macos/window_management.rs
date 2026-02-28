//! macOS WindowManager trait implementation using osascript/Core Graphics.

use std::process::Command;

use jarvis_common::errors::PlatformError;
use jarvis_common::types::Rect;

use crate::platform::{ExternalWindow, Result, WatchHandle, WindowEvent, WindowId, WindowManager};

use super::MacOsWindowManager;

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
