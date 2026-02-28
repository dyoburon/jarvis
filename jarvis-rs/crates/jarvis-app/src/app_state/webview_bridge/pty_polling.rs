//! PTY output polling: reads from all PTYs and sends output to webviews + mobile.

use crate::app_state::core::JarvisApp;
use crate::app_state::ws_server::protocol::ServerMessage;

// =============================================================================
// PTY OUTPUT POLLING
// =============================================================================

impl JarvisApp {
    /// Drain output from all PTYs and send to their corresponding webviews
    /// and any connected mobile clients.
    ///
    /// Called from the main poll loop. For each PTY with pending output,
    /// dispatches via IPC to the terminal's `pty_output` handler in xterm.js,
    /// and broadcasts to mobile clients via the WebSocket bridge.
    ///
    /// Also checks for finished PTYs and sends `pty_exit` notifications.
    pub(in crate::app_state) fn poll_pty_output(&mut self) {
        // Drain output from all PTYs
        let outputs = self.ptys.drain_all_output();

        for (pane_id, data) in &outputs {
            let text = String::from_utf8_lossy(data);

            // Send to local WebView
            if let Some(ref registry) = self.webviews {
                if let Some(handle) = registry.get(*pane_id) {
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

            // Broadcast to mobile clients
            if let Some(ref broadcaster) = self.mobile_broadcaster {
                broadcaster.send(ServerMessage::PtyOutput {
                    pane_id: *pane_id,
                    data: text.into_owned(),
                });
            }
        }

        // Check for finished PTYs and notify webviews + mobile
        let finished = self.ptys.check_finished();
        for pane_id in finished {
            tracing::info!(pane_id, "PTY process exited");

            let exit_code = self.ptys.kill_and_remove(pane_id);
            let code = exit_code.unwrap_or(0);

            if let Some(ref registry) = self.webviews {
                if let Some(handle) = registry.get(pane_id) {
                    let payload = serde_json::json!({ "code": code });
                    if let Err(e) = handle.send_ipc("pty_exit", &payload) {
                        tracing::warn!(
                            pane_id,
                            error = %e,
                            "Failed to send pty_exit to webview"
                        );
                    }
                }
            }

            if let Some(ref broadcaster) = self.mobile_broadcaster {
                broadcaster.send(ServerMessage::PtyExit { pane_id, code });
            }
        }
    }
}
