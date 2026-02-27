use glyphon::Color as GlyphonColor;

use crate::assistant_panel::{AssistantPanel, ChatRole};
use crate::ui::UiChrome;

use super::state::RenderState;

impl RenderState {
    /// Update cached status bar and tab bar text buffers when content changes.
    pub(crate) fn update_chrome_text_buffers(
        &mut self,
        chrome: &UiChrome,
        viewport_width: f32,
        _viewport_height: f32,
        _scale_factor: f32,
    ) {
        // Status bar text
        if let Some(ref sb) = chrome.status_bar {
            let fg = GlyphonColor::rgba(
                (sb.fg_color[0] * 255.0) as u8,
                (sb.fg_color[1] * 255.0) as u8,
                (sb.fg_color[2] * 255.0) as u8,
                (sb.fg_color[3] * 255.0) as u8,
            );
            let width = viewport_width / 3.0;
            if !sb.left_text.is_empty() {
                let needs = match &self.cached_status_left {
                    Some((prev, _)) => *prev != sb.left_text,
                    None => true,
                };
                if needs {
                    let buf = self.text.create_ui_text_buffer(&sb.left_text, fg, width);
                    self.cached_status_left = Some((sb.left_text.clone(), buf));
                }
            } else {
                self.cached_status_left = None;
            }
            if !sb.center_text.is_empty() {
                let needs = match &self.cached_status_center {
                    Some((prev, _)) => *prev != sb.center_text,
                    None => true,
                };
                if needs {
                    let buf = self.text.create_ui_text_buffer(&sb.center_text, fg, width);
                    self.cached_status_center = Some((sb.center_text.clone(), buf));
                }
            } else {
                self.cached_status_center = None;
            }
            if !sb.right_text.is_empty() {
                let needs = match &self.cached_status_right {
                    Some((prev, _)) => *prev != sb.right_text,
                    None => true,
                };
                if needs {
                    let buf = self.text.create_ui_text_buffer(&sb.right_text, fg, width);
                    self.cached_status_right = Some((sb.right_text.clone(), buf));
                }
            } else {
                self.cached_status_right = None;
            }
        }

        // Tab bar text
        if let Some(ref tb) = chrome.tab_bar {
            let tab_width = if tb.tabs.is_empty() {
                viewport_width
            } else {
                (viewport_width / tb.tabs.len() as f32).min(200.0)
            };
            let tabs_changed = self.cached_tab_buffers.len() != tb.tabs.len()
                || self
                    .cached_tab_buffers
                    .iter()
                    .zip(tb.tabs.iter())
                    .any(|((t, a, _), tab)| *t != tab.title || *a != tab.is_active);
            if tabs_changed {
                self.cached_tab_buffers.clear();
                for tab in &tb.tabs {
                    let color = if tab.is_active {
                        GlyphonColor::rgba(255, 255, 255, 255)
                    } else {
                        GlyphonColor::rgba(150, 150, 150, 200)
                    };
                    let buf = self
                        .text
                        .create_ui_text_buffer(&tab.title, color, tab_width);
                    self.cached_tab_buffers
                        .push((tab.title.clone(), tab.is_active, buf));
                }
            }
        }
    }

    /// Create assistant panel text buffers (mutable phase — creates buffers).
    pub(crate) fn build_assistant_text_buffers(
        &mut self,
        assistant: Option<&AssistantPanel>,
        _chrome: &UiChrome,
        viewport_width: f32,
        _viewport_height: f32,
    ) {
        self.cached_assistant_buffers.clear();
        let Some(panel) = assistant else {
            return;
        };

        let panel_width = viewport_width * 0.4;
        let text_width = panel_width - 32.0;

        // Title text
        let title_buf = self.text.create_ui_text_buffer(
            "Jarvis Assistant",
            GlyphonColor::rgba(0, 212, 255, 255),
            text_width,
        );
        self.cached_assistant_buffers.push(title_buf);

        // Input field text
        let (input_display, input_color) = if panel.input_text().is_empty() {
            if panel.is_streaming() {
                (
                    "Waiting for response...",
                    GlyphonColor::rgba(100, 100, 120, 180),
                )
            } else {
                ("Ask Jarvis...", GlyphonColor::rgba(100, 100, 120, 180))
            }
        } else {
            (panel.input_text(), GlyphonColor::rgba(255, 255, 255, 255))
        };
        let input_buf = self
            .text
            .create_ui_text_buffer(input_display, input_color, text_width);
        self.cached_assistant_buffers.push(input_buf);

        // Error text
        if let Some(err) = panel.error() {
            let err_buf = self.text.create_ui_text_buffer(
                err,
                GlyphonColor::rgba(255, 100, 100, 255),
                text_width,
            );
            self.cached_assistant_buffers.push(err_buf);
        }

        // Build message buffers (most recent first, rendered bottom-up)
        for msg in panel.messages().iter().rev() {
            let (prefix, color) = match msg.role {
                ChatRole::User => ("You: ", GlyphonColor::rgba(120, 180, 220, 230)),
                ChatRole::Assistant => ("Jarvis: ", GlyphonColor::rgba(220, 220, 230, 255)),
            };
            let display = format!("{}{}", prefix, msg.content);
            let buf = self.text.create_ui_text_buffer(&display, color, text_width);
            self.cached_assistant_buffers.push(buf);
        }

        // Streaming text (currently being received)
        if panel.is_streaming() && !panel.streaming_text().is_empty() {
            let display = format!("Jarvis: {}", panel.streaming_text());
            let buf = self.text.create_ui_text_buffer(
                &display,
                GlyphonColor::rgba(200, 220, 255, 255),
                text_width,
            );
            self.cached_assistant_buffers.push(buf);
        }
    }
}
