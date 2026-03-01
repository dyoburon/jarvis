//! AI assistant panel: key handling, runtime management, and event polling.

use jarvis_common::actions::Action;
use jarvis_common::types::PaneKind;

use super::assistant_task::assistant_task;
use super::core::JarvisApp;
use super::types::AssistantEvent;

impl JarvisApp {
    /// Handle key events for the assistant panel.
    pub(super) fn handle_assistant_key(&mut self, key_name: &str, is_press: bool) -> bool {
        if !is_press || !self.assistant_open {
            return false;
        }

        let panel = match self.assistant_panel.as_mut() {
            Some(p) => p,
            None => return false,
        };

        match key_name {
            "Escape" => {
                self.dispatch(Action::CloseOverlay);
                true
            }
            "Enter" => {
                let input = panel.take_input();
                if !input.is_empty() && !panel.is_streaming() {
                    panel.push_user_message(input.clone());
                    if let Some(ref tx) = self.assistant_tx {
                        let _ = tx.send(input);
                    }
                }
                true
            }
            "Backspace" => {
                panel.backspace();
                true
            }
            "Up" => {
                panel.scroll_up(3);
                true
            }
            "Down" => {
                panel.scroll_down(3);
                true
            }
            _ => {
                if key_name.len() == 1 {
                    let ch = key_name.chars().next().unwrap();
                    if ch.is_ascii_graphic() || ch == ' ' {
                        panel.append_char(ch);
                        return true;
                    }
                }
                false
            }
        }
    }

    /// Lazily initialize the async AI task and communication channels.
    pub(super) fn ensure_assistant_runtime(&mut self) {
        if self.assistant_tx.is_some() {
            return;
        }

        let (user_tx, user_rx) = std::sync::mpsc::channel::<String>();
        let (event_tx, event_rx) = std::sync::mpsc::channel::<AssistantEvent>();

        self.assistant_tx = Some(user_tx);
        self.assistant_rx = Some(event_rx);

        if self.tokio_runtime.is_none() {
            match tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
            {
                Ok(rt) => self.tokio_runtime = Some(rt),
                Err(e) => {
                    tracing::error!("Failed to create tokio runtime: {e}");
                    return;
                }
            }
        }

        let rt = self.tokio_runtime.as_ref().unwrap();
        rt.spawn(async move {
            assistant_task(user_rx, event_tx).await;
        });
    }

    /// Poll for assistant events from the async task (non-blocking).
    pub(super) fn poll_assistant(&mut self) {
        if let Some(ref rx) = self.assistant_rx {
            while let Ok(event) = rx.try_recv() {
                match event {
                    AssistantEvent::Initialized { model_name } => {
                        self.send_assistant_ipc(
                            "assistant_config",
                            &serde_json::json!({ "model_name": model_name }),
                        );
                    }
                    AssistantEvent::StreamChunk(ref chunk) => {
                        if let Some(ref mut panel) = self.assistant_panel {
                            panel.append_streaming_chunk(chunk);
                        }
                        self.send_assistant_ipc(
                            "assistant_chunk",
                            &serde_json::json!({ "text": chunk }),
                        );
                    }
                    AssistantEvent::Done => {
                        // Capture the accumulated text before finishing
                        let full_text = self
                            .assistant_panel
                            .as_ref()
                            .map(|p| p.streaming_text().to_string())
                            .unwrap_or_default();
                        if let Some(ref mut panel) = self.assistant_panel {
                            panel.finish_streaming();
                        }
                        self.send_assistant_ipc(
                            "assistant_output",
                            &serde_json::json!({ "text": full_text }),
                        );
                    }
                    AssistantEvent::Error(ref msg) => {
                        tracing::warn!("Assistant error: {msg}");
                        if let Some(ref mut panel) = self.assistant_panel {
                            panel.set_error(msg.clone());
                            panel.finish_streaming();
                        }
                        self.send_assistant_ipc(
                            "assistant_error",
                            &serde_json::json!({ "message": msg }),
                        );
                    }
                }
                self.needs_redraw = true;
            }
        }
    }

    /// Send an IPC message to all assistant webview panes.
    fn send_assistant_ipc(&self, kind: &str, payload: &serde_json::Value) {
        let pane_ids = self.tiling.panes_by_kind(PaneKind::Assistant);
        if let Some(ref registry) = self.webviews {
            for pane_id in pane_ids {
                if let Some(handle) = registry.get(pane_id) {
                    if let Err(e) = handle.send_ipc(kind, payload) {
                        tracing::warn!(pane_id, kind, error = %e, "Failed to send assistant IPC");
                    }
                }
            }
        }
    }
}
