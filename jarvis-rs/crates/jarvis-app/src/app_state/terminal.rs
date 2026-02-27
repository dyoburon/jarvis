//! Terminal/PTY management: spawning, polling, resizing, clipboard.

use jarvis_common::notifications::Notification;
use jarvis_terminal::pty::PtyManager;
use jarvis_terminal::{JarvisEventProxy, SizeInfo, TermConfig, VteProcessor};

use super::core::JarvisApp;
use super::types::PaneState;

impl JarvisApp {
    /// Drain all pending PTY output for every pane.
    /// Returns true if any data was read.
    pub(super) fn poll_pty_output(&mut self) -> bool {
        let mut got_data = false;
        for (_, pane) in self.panes.iter_mut() {
            let mut buf = [0u8; 8192];
            loop {
                match pane.pty.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        pane.processor.advance(&mut pane.term, &buf[..n]);
                        got_data = true;
                    }
                    Err(_) => break,
                }
            }
        }
        got_data
    }

    /// Spawn a PTY for the currently focused tiling pane.
    pub(super) fn spawn_pty_for_focused(&mut self) {
        let id = self.tiling.focused_id();
        if self.panes.contains_key(&id) {
            return;
        }
        let shell = jarvis_terminal::shell::detect_shell();
        let (cols, rows) = self.pane_dimensions(id);
        match PtyManager::spawn(&shell, cols as u16, rows as u16, None) {
            Ok(pty) => {
                let size = SizeInfo::new(cols, rows, 10.0, 20.0);
                let (event_proxy, _event_rx) = JarvisEventProxy::new();
                let term = jarvis_terminal::Term::new(TermConfig::default(), &size, event_proxy);
                let processor = VteProcessor::new();
                self.panes.insert(
                    id,
                    PaneState {
                        term,
                        processor,
                        pty,
                    },
                );
                self.event_bus
                    .publish(jarvis_common::events::Event::PaneOpened(
                        jarvis_common::PaneId(id),
                    ));
                tracing::info!("Spawned PTY for pane {id} ({cols}x{rows})");
            }
            Err(e) => {
                tracing::error!("Failed to spawn PTY: {e}");
                self.notifications.push(Notification::error(
                    "PTY Error",
                    format!("Failed to spawn shell: {e}"),
                ));
            }
        }
    }

    /// Get terminal grid dimensions for a specific pane based on its layout rect.
    pub(super) fn pane_dimensions(&self, pane_id: u32) -> (usize, usize) {
        if let Some(ref rs) = self.render_state {
            let vw = rs.gpu.size.width as f32;
            let vh = rs.gpu.size.height as f32;
            let content = self.chrome.content_rect(vw, vh);
            let layout = self.tiling.compute_layout(content);

            if let Some((_, rect)) = layout.iter().find(|(id, _)| *id == pane_id) {
                return rs.grid_dimensions_for_rect(rect);
            }
            rs.grid_dimensions()
        } else {
            (80, 24)
        }
    }

    /// Resize all panes' PTYs based on their current layout rects.
    pub(super) fn resize_all_panes(&mut self) {
        if let Some(ref mut rs) = self.render_state {
            let vw = rs.gpu.size.width as f32;
            let vh = rs.gpu.size.height as f32;
            let content = self.chrome.content_rect(vw, vh);
            let layout = self.tiling.compute_layout(content);

            for (id, rect) in &layout {
                if let Some(pane) = self.panes.get_mut(id) {
                    let (cols, rows) = rs.grid_dimensions_for_rect(rect);
                    let _ = pane.pty.resize(cols as u16, rows as u16);
                    let size = SizeInfo::new(cols, rows, 10.0, 20.0);
                    pane.term.resize(size);
                }
                // Invalidate cached text buffers for resized panes
                rs.text.invalidate_pane_cache(*id);
            }
        }
    }

    pub(super) fn copy_selection(&mut self) {
        tracing::debug!("copy_selection: not yet implemented");
    }

    pub(super) fn paste_from_clipboard(&mut self) {
        match jarvis_platform::Clipboard::new() {
            Ok(mut clip) => match clip.get_text() {
                Ok(text) => {
                    let bytes = self.input.encode_paste(&text);
                    let focused = self.tiling.focused_id();
                    if let Some(pane) = self.panes.get_mut(&focused) {
                        let _ = pane.pty.write(&bytes);
                    }
                }
                Err(e) => tracing::debug!("clipboard read failed: {e}"),
            },
            Err(e) => tracing::debug!("clipboard init failed: {e}"),
        }
    }
}
