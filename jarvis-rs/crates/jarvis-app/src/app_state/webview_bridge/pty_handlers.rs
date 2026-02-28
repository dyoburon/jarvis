//! PTY-specific IPC handlers: pty_input, pty_resize, pty_restart, terminal_ready.

use jarvis_webview::IpcPayload;

use crate::app_state::core::JarvisApp;
use crate::app_state::pty_bridge;

// =============================================================================
// PTY IPC HANDLERS
// =============================================================================

impl JarvisApp {
    /// Handle `pty_input` — write keystroke data to the pane's PTY.
    ///
    /// Payload: `{ data: "<string>" }`
    pub(in crate::app_state) fn handle_pty_input(&mut self, pane_id: u32, payload: &IpcPayload) {
        let data = match extract_string_field(payload, "data") {
            Some(d) => d,
            None => {
                tracing::warn!(pane_id, "pty_input: missing 'data' field");
                return;
            }
        };

        if let Err(e) = self.ptys.write_input(pane_id, data.as_bytes()) {
            tracing::warn!(pane_id, error = %e, "pty_input: write failed");
        }
    }

    /// Handle `pty_resize` — resize the pane's PTY to new dimensions.
    ///
    /// Payload: `{ cols: <number>, rows: <number> }`
    pub(in crate::app_state) fn handle_pty_resize(&mut self, pane_id: u32, payload: &IpcPayload) {
        let (cols, rows) = match extract_size_fields(payload) {
            Some(s) => s,
            None => {
                tracing::warn!(pane_id, "pty_resize: missing cols/rows");
                return;
            }
        };

        if let Err(e) = self.ptys.resize(pane_id, cols, rows) {
            tracing::warn!(pane_id, error = %e, "pty_resize: resize failed");
        } else {
            tracing::debug!(pane_id, cols, rows, "PTY resized");
        }
    }

    /// Handle `pty_restart` — kill the old PTY and spawn a new one.
    ///
    /// Payload: `{ cols: <number>, rows: <number> }`
    pub(in crate::app_state) fn handle_pty_restart(&mut self, pane_id: u32, payload: &IpcPayload) {
        // Kill existing PTY if present
        if self.ptys.contains(pane_id) {
            self.ptys.kill_and_remove(pane_id);
            tracing::info!(pane_id, "PTY killed for restart");
        }

        let (cols, rows) = extract_size_fields(payload)
            .unwrap_or((pty_bridge::DEFAULT_COLS, pty_bridge::DEFAULT_ROWS));

        match pty_bridge::spawn_pty(cols, rows) {
            Ok(handle) => {
                self.ptys.insert(pane_id, handle);
                tracing::info!(pane_id, cols, rows, "PTY restarted");
            }
            Err(e) => {
                tracing::error!(pane_id, error = %e, "PTY restart failed");
            }
        }
    }

    /// Handle `terminal_ready` — spawn a PTY for a newly loaded terminal panel.
    ///
    /// Payload: `{ cols: <number>, rows: <number> }`
    pub(in crate::app_state) fn handle_terminal_ready(
        &mut self,
        pane_id: u32,
        payload: &IpcPayload,
    ) {
        // Don't spawn if already exists (e.g. page reload)
        if self.ptys.contains(pane_id) {
            tracing::debug!(pane_id, "terminal_ready: PTY already exists");
            return;
        }

        let (cols, rows) = extract_size_fields(payload)
            .unwrap_or((pty_bridge::DEFAULT_COLS, pty_bridge::DEFAULT_ROWS));

        match pty_bridge::spawn_pty(cols, rows) {
            Ok(handle) => {
                self.ptys.insert(pane_id, handle);
                tracing::info!(pane_id, cols, rows, "PTY spawned for terminal");
            }
            Err(e) => {
                tracing::error!(pane_id, error = %e, "PTY spawn failed");
            }
        }
    }
}

// =============================================================================
// PAYLOAD HELPERS
// =============================================================================

/// Extract a string field from an IPC payload.
fn extract_string_field(payload: &IpcPayload, field: &str) -> Option<String> {
    match payload {
        IpcPayload::Json(obj) => obj.get(field)?.as_str().map(|s| s.to_string()),
        IpcPayload::Text(s) if field == "data" => Some(s.clone()),
        _ => None,
    }
}

/// Extract `cols` and `rows` from an IPC payload as `(u16, u16)`.
fn extract_size_fields(payload: &IpcPayload) -> Option<(u16, u16)> {
    match payload {
        IpcPayload::Json(obj) => {
            let cols = obj.get("cols")?.as_u64()? as u16;
            let rows = obj.get("rows")?.as_u64()? as u16;
            // Sanity bounds: reject absurd sizes
            if cols == 0 || rows == 0 || cols > 500 || rows > 500 {
                return None;
            }
            Some((cols, rows))
        }
        _ => None,
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_string_field_from_json() {
        let payload = IpcPayload::Json(serde_json::json!({ "data": "hello" }));
        assert_eq!(
            extract_string_field(&payload, "data"),
            Some("hello".to_string())
        );
    }

    #[test]
    fn extract_string_field_missing() {
        let payload = IpcPayload::Json(serde_json::json!({ "other": "value" }));
        assert_eq!(extract_string_field(&payload, "data"), None);
    }

    #[test]
    fn extract_string_field_from_text() {
        let payload = IpcPayload::Text("keystrokes".to_string());
        assert_eq!(
            extract_string_field(&payload, "data"),
            Some("keystrokes".to_string())
        );
    }

    #[test]
    fn extract_string_field_from_none() {
        let payload = IpcPayload::None;
        assert_eq!(extract_string_field(&payload, "data"), None);
    }

    #[test]
    fn extract_size_fields_valid() {
        let payload = IpcPayload::Json(serde_json::json!({ "cols": 120, "rows": 40 }));
        assert_eq!(extract_size_fields(&payload), Some((120, 40)));
    }

    #[test]
    fn extract_size_fields_missing_cols() {
        let payload = IpcPayload::Json(serde_json::json!({ "rows": 40 }));
        assert_eq!(extract_size_fields(&payload), None);
    }

    #[test]
    fn extract_size_fields_missing_rows() {
        let payload = IpcPayload::Json(serde_json::json!({ "cols": 80 }));
        assert_eq!(extract_size_fields(&payload), None);
    }

    #[test]
    fn extract_size_fields_zero_rejected() {
        let payload = IpcPayload::Json(serde_json::json!({ "cols": 0, "rows": 24 }));
        assert_eq!(extract_size_fields(&payload), None);
    }

    #[test]
    fn extract_size_fields_too_large_rejected() {
        let payload = IpcPayload::Json(serde_json::json!({ "cols": 501, "rows": 24 }));
        assert_eq!(extract_size_fields(&payload), None);
    }

    #[test]
    fn extract_size_fields_from_text_returns_none() {
        let payload = IpcPayload::Text("not json".to_string());
        assert_eq!(extract_size_fields(&payload), None);
    }

    #[test]
    fn extract_size_fields_from_none_returns_none() {
        let payload = IpcPayload::None;
        assert_eq!(extract_size_fields(&payload), None);
    }
}
