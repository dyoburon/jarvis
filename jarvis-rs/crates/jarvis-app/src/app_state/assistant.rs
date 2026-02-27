//! AI assistant panel: key handling, runtime management, and event polling.

use jarvis_common::actions::Action;

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
                    AssistantEvent::StreamChunk(chunk) => {
                        if let Some(ref mut panel) = self.assistant_panel {
                            panel.append_streaming_chunk(&chunk);
                        }
                    }
                    AssistantEvent::Done => {
                        if let Some(ref mut panel) = self.assistant_panel {
                            panel.finish_streaming();
                        }
                    }
                    AssistantEvent::Error(msg) => {
                        tracing::warn!("Assistant error: {msg}");
                        if let Some(ref mut panel) = self.assistant_panel {
                            panel.set_error(msg);
                            panel.finish_streaming();
                        }
                    }
                }
                self.needs_redraw = true;
            }
        }
    }
}
