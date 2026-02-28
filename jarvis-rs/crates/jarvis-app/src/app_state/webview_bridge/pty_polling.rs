//! PTY output polling: reads from all PTYs and sends output to webviews.

use crate::app_state::core::JarvisApp;

// =============================================================================
// PTY OUTPUT POLLING
// =============================================================================

impl JarvisApp {
    /// Drain output from all PTYs and send to their corresponding webviews.
    ///
    /// Called from the main poll loop. For each PTY with pending output,
    /// encodes the bytes as a base64 string and dispatches via IPC to the
    /// terminal's `pty_output` handler in xterm.js.
    ///
    /// Also checks for finished PTYs and sends `pty_exit` notifications.
    pub(in crate::app_state) fn poll_pty_output(&mut self) {
        // Drain output from all PTYs
        let outputs = self.ptys.drain_all_output();

        if let Some(ref registry) = self.webviews {
            for (pane_id, data) in outputs {
                if let Some(handle) = registry.get(pane_id) {
                    // Send output as a string — xterm.js expects text data.
                    // PTY output is terminal escape sequences + text, which
                    // is valid UTF-8 in most cases. For binary data, we use
                    // lossy conversion (replacement char for invalid bytes).
                    let text = String::from_utf8_lossy(&data);
                    let payload = serde_json::json!({ "data": text });

                    if let Err(e) = handle.send_ipc("pty_output", &payload) {
                        tracing::warn!(
                            pane_id,
                            error = %e,
                            "Failed to send PTY output to webview"
                        );
                    }
                }
            }
        }

        // Check for finished PTYs and notify webviews
        let finished = self.ptys.check_finished();
        for pane_id in finished {
            tracing::info!(pane_id, "PTY process exited");

            // Get exit code before removing
            let exit_code = self.ptys.kill_and_remove(pane_id);

            if let Some(ref registry) = self.webviews {
                if let Some(handle) = registry.get(pane_id) {
                    let payload = serde_json::json!({
                        "code": exit_code.unwrap_or(0)
                    });
                    if let Err(e) = handle.send_ipc("pty_exit", &payload) {
                        tracing::warn!(
                            pane_id,
                            error = %e,
                            "Failed to send pty_exit to webview"
                        );
                    }
                }
            }
        }
    }
}
