use glyphon::{Color as GlyphonColor, TextArea, TextBounds};
use jarvis_common::types::Rect;

use crate::assistant_panel::AssistantPanel;
use crate::gpu::RendererError;
use crate::quad::QuadInstance;
use crate::ui::UiChrome;

use super::helpers::log_first_frame;
use super::state::RenderState;

impl RenderState {
    /// Render multiple terminal panes with UI chrome (status bar, tab bar, borders).
    pub fn render_frame_multi(
        &mut self,
        panes: &[(u32, Rect, &jarvis_terminal::Grid, Vec<bool>)],
        focused_id: u32,
        chrome: &UiChrome,
        assistant: Option<&AssistantPanel>,
    ) -> Result<(), RendererError> {
        let output = match self.gpu.current_texture() {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("Failed to get surface texture: {e}");
                return Err(RendererError::SurfaceError(e.to_string()));
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("jarvis multi-pane encoder"),
            });

        let viewport_width = self.gpu.size.width as f32;
        let viewport_height = self.gpu.size.height as f32;
        let scale_factor = self.gpu.scale_factor as f32;

        // 1. Build quads for UI chrome backgrounds and pane borders
        let mut quads: Vec<QuadInstance> = Vec::new();

        // Status bar background
        if let Some(ref sb) = chrome.status_bar {
            if let Some(sb_rect) = chrome.status_bar_rect(viewport_width, viewport_height) {
                quads.push(QuadInstance {
                    rect: [
                        sb_rect.x as f32,
                        sb_rect.y as f32,
                        sb_rect.width as f32,
                        sb_rect.height as f32,
                    ],
                    color: sb.bg_color,
                });
            }
        }

        // Tab bar background
        if let Some(ref _tb) = chrome.tab_bar {
            if let Some(tb_rect) = chrome.tab_bar_rect(viewport_width) {
                quads.push(QuadInstance {
                    rect: [
                        tb_rect.x as f32,
                        tb_rect.y as f32,
                        tb_rect.width as f32,
                        tb_rect.height as f32,
                    ],
                    color: [0.08, 0.08, 0.10, 0.95],
                });
            }
        }

        // Pane borders (thin colored lines around each pane)
        if panes.len() > 1 {
            for &(pane_id, ref rect, _, _) in panes {
                let is_focused = pane_id == focused_id;
                let border_color = if is_focused {
                    [0.0, 0.83, 1.0, 0.5] // cyan glow for focused
                } else {
                    [0.3, 0.3, 0.35, 0.3] // dim for unfocused
                };
                let bw = if is_focused { 2.0 } else { 1.0 };
                let x = rect.x as f32;
                let y = rect.y as f32;
                let w = rect.width as f32;
                let h = rect.height as f32;

                // Top border
                quads.push(QuadInstance {
                    rect: [x, y, w, bw],
                    color: border_color,
                });
                // Bottom border
                quads.push(QuadInstance {
                    rect: [x, y + h - bw, w, bw],
                    color: border_color,
                });
                // Left border
                quads.push(QuadInstance {
                    rect: [x, y, bw, h],
                    color: border_color,
                });
                // Right border
                quads.push(QuadInstance {
                    rect: [x + w - bw, y, bw, h],
                    color: border_color,
                });
            }
        }

        // Assistant panel overlay quads
        if assistant.is_some() {
            build_assistant_quads(&mut quads, chrome, viewport_width, viewport_height);
        }

        self.quad
            .prepare(&self.gpu.queue, &quads, viewport_width, viewport_height);

        // 2. Build text for terminal panes
        #[allow(clippy::type_complexity)]
        let pane_data: Vec<(u32, &jarvis_terminal::Grid, &[bool], f32, f32, f32, f32)> = panes
            .iter()
            .map(|(id, rect, grid, dirty)| {
                (
                    *id,
                    *grid,
                    dirty.as_slice(),
                    rect.x as f32,
                    rect.y as f32,
                    rect.width as f32,
                    rect.height as f32,
                )
            })
            .collect();

        // 3. All mutable buffer creation happens first
        self.update_chrome_text_buffers(chrome, viewport_width, viewport_height, scale_factor);
        self.build_assistant_text_buffers(assistant, chrome, viewport_width, viewport_height);

        // 4. Build TextArea references from cached buffers, then call prepare.
        // We use direct field access to avoid borrow conflicts between
        // cached buffers (immutable) and self.text (mutable).
        let extra_text_areas = build_all_text_areas(
            &self.cached_status_left,
            &self.cached_status_center,
            &self.cached_status_right,
            &self.cached_tab_buffers,
            &self.cached_assistant_buffers,
            chrome,
            assistant,
            viewport_width,
            viewport_height,
            scale_factor,
            self.text.cell_height,
        );

        self.text.prepare_multi_grid(
            &self.gpu.device,
            &self.gpu.queue,
            &pane_data,
            extra_text_areas,
            viewport_width,
            viewport_height,
            scale_factor,
        );

        // 5. Render pass
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("jarvis multi-pane pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Draw quad backgrounds first
            self.quad.render(&mut pass);

            // Draw text on top
            self.text.render(&mut pass);
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        log_first_frame(self.gpu.size.width, self.gpu.size.height, self.gpu.format());

        Ok(())
    }
}

/// Build all extra TextArea references from cached buffers.
///
/// This is a free function to avoid borrow conflicts: it borrows the cached
/// buffer fields immutably while `self.text` can later be borrowed mutably.
#[allow(clippy::too_many_arguments)]
fn build_all_text_areas<'a>(
    cached_status_left: &'a Option<(String, glyphon::Buffer)>,
    cached_status_center: &'a Option<(String, glyphon::Buffer)>,
    cached_status_right: &'a Option<(String, glyphon::Buffer)>,
    cached_tab_buffers: &'a [(String, bool, glyphon::Buffer)],
    cached_assistant_buffers: &'a [glyphon::Buffer],
    chrome: &UiChrome,
    assistant: Option<&AssistantPanel>,
    viewport_width: f32,
    viewport_height: f32,
    scale_factor: f32,
    cell_height: f32,
) -> Vec<TextArea<'a>> {
    let mut areas: Vec<TextArea<'a>> = Vec::new();

    // Status bar text areas
    if let Some(ref sb) = chrome.status_bar {
        if let Some(sb_rect) = chrome.status_bar_rect(viewport_width, viewport_height) {
            let bounds = TextBounds {
                left: sb_rect.x as i32,
                top: sb_rect.y as i32,
                right: (sb_rect.x + sb_rect.width) as i32,
                bottom: (sb_rect.y + sb_rect.height) as i32,
            };
            if !sb.left_text.is_empty() {
                if let Some((_, ref buf)) = cached_status_left {
                    areas.push(TextArea {
                        buffer: buf,
                        left: sb_rect.x as f32 + 8.0,
                        top: sb_rect.y as f32 + 2.0,
                        scale: scale_factor,
                        bounds,
                        default_color: GlyphonColor::rgba(255, 255, 255, 255),
                        custom_glyphs: &[],
                    });
                }
            }
            if !sb.center_text.is_empty() {
                if let Some((_, ref buf)) = cached_status_center {
                    areas.push(TextArea {
                        buffer: buf,
                        left: sb_rect.x as f32 + sb_rect.width as f32 / 3.0,
                        top: sb_rect.y as f32 + 2.0,
                        scale: scale_factor,
                        bounds,
                        default_color: GlyphonColor::rgba(255, 255, 255, 255),
                        custom_glyphs: &[],
                    });
                }
            }
            if !sb.right_text.is_empty() {
                if let Some((_, ref buf)) = cached_status_right {
                    areas.push(TextArea {
                        buffer: buf,
                        left: sb_rect.x as f32 + sb_rect.width as f32 * 2.0 / 3.0,
                        top: sb_rect.y as f32 + 2.0,
                        scale: scale_factor,
                        bounds,
                        default_color: GlyphonColor::rgba(255, 255, 255, 255),
                        custom_glyphs: &[],
                    });
                }
            }
        }
    }

    // Tab bar text areas
    if let Some(ref tb) = chrome.tab_bar {
        if let Some(tb_rect) = chrome.tab_bar_rect(viewport_width) {
            let tab_width = if tb.tabs.is_empty() {
                tb_rect.width as f32
            } else {
                (tb_rect.width as f32 / tb.tabs.len() as f32).min(200.0)
            };
            for (i, (_, _, ref buf)) in cached_tab_buffers.iter().enumerate() {
                areas.push(TextArea {
                    buffer: buf,
                    left: tb_rect.x as f32 + i as f32 * tab_width + 8.0,
                    top: tb_rect.y as f32 + 6.0,
                    scale: scale_factor,
                    bounds: TextBounds {
                        left: tb_rect.x as i32,
                        top: tb_rect.y as i32,
                        right: (tb_rect.x + tb_rect.width) as i32,
                        bottom: (tb_rect.y + tb_rect.height) as i32,
                    },
                    default_color: GlyphonColor::rgba(255, 255, 255, 255),
                    custom_glyphs: &[],
                });
            }
        }
    }

    // Assistant panel text areas
    if let Some(panel) = assistant {
        if !cached_assistant_buffers.is_empty() {
            let panel_width = viewport_width * 0.4;
            let panel_x = viewport_width - panel_width;
            let tab_height = chrome
                .tab_bar
                .as_ref()
                .map(|_| {
                    chrome
                        .tab_bar_rect(viewport_width)
                        .map(|r| r.height as f32)
                        .unwrap_or(0.0)
                })
                .unwrap_or(0.0);
            let status_height = chrome
                .status_bar
                .as_ref()
                .map(|_| {
                    chrome
                        .status_bar_rect(viewport_width, viewport_height)
                        .map(|r| r.height as f32)
                        .unwrap_or(0.0)
                })
                .unwrap_or(0.0);
            let panel_y = tab_height;
            let panel_h = viewport_height - tab_height - status_height;
            let input_height = 32.0;

            let panel_bounds = TextBounds {
                left: panel_x as i32,
                top: panel_y as i32,
                right: (panel_x + panel_width) as i32,
                bottom: (panel_y + panel_h) as i32,
            };

            let mut buf_idx = 0;

            // Title
            areas.push(TextArea {
                buffer: &cached_assistant_buffers[buf_idx],
                left: panel_x + 16.0,
                top: panel_y + 8.0,
                scale: scale_factor,
                bounds: panel_bounds,
                default_color: GlyphonColor::rgba(0, 212, 255, 255),
                custom_glyphs: &[],
            });
            buf_idx += 1;

            // Input text
            let input_y = panel_y + panel_h - input_height - 8.0;
            areas.push(TextArea {
                buffer: &cached_assistant_buffers[buf_idx],
                left: panel_x + 16.0,
                top: input_y + 6.0,
                scale: scale_factor,
                bounds: TextBounds {
                    left: (panel_x + 8.0) as i32,
                    top: input_y as i32,
                    right: (panel_x + panel_width - 8.0) as i32,
                    bottom: (input_y + input_height) as i32,
                },
                default_color: GlyphonColor::rgba(255, 255, 255, 255),
                custom_glyphs: &[],
            });
            buf_idx += 1;

            // Error text (if any)
            let messages_bottom = input_y - 8.0;
            let mut y_cursor = messages_bottom;
            if panel.error().is_some() {
                y_cursor -= cell_height;
                if y_cursor > panel_y + 30.0 {
                    areas.push(TextArea {
                        buffer: &cached_assistant_buffers[buf_idx],
                        left: panel_x + 16.0,
                        top: y_cursor,
                        scale: scale_factor,
                        bounds: panel_bounds,
                        default_color: GlyphonColor::rgba(255, 100, 100, 255),
                        custom_glyphs: &[],
                    });
                }
                buf_idx += 1;
            }

            // Messages (reversed — most recent at bottom)
            let remaining = cached_assistant_buffers.len() - buf_idx;
            for i in 0..remaining {
                let est_chars = if buf_idx + i < cached_assistant_buffers.len() {
                    let buf = &cached_assistant_buffers[buf_idx + i];
                    buf.layout_runs().count().max(1)
                } else {
                    1
                };
                let text_h = est_chars as f32 * cell_height;
                y_cursor -= text_h + 4.0;
                if y_cursor < panel_y + 30.0 {
                    break;
                }
                areas.push(TextArea {
                    buffer: &cached_assistant_buffers[buf_idx + i],
                    left: panel_x + 16.0,
                    top: y_cursor,
                    scale: scale_factor,
                    bounds: panel_bounds,
                    default_color: GlyphonColor::rgba(220, 220, 220, 255),
                    custom_glyphs: &[],
                });
            }
        }
    }

    areas
}

/// Build assistant panel overlay quads (background, border, input field).
fn build_assistant_quads(
    quads: &mut Vec<QuadInstance>,
    chrome: &UiChrome,
    viewport_width: f32,
    viewport_height: f32,
) {
    let panel_width = viewport_width * 0.4;
    let panel_x = viewport_width - panel_width;
    let tab_height = chrome
        .tab_bar
        .as_ref()
        .map(|_| {
            chrome
                .tab_bar_rect(viewport_width)
                .map(|r| r.height as f32)
                .unwrap_or(0.0)
        })
        .unwrap_or(0.0);
    let status_height = chrome
        .status_bar
        .as_ref()
        .map(|_| {
            chrome
                .status_bar_rect(viewport_width, viewport_height)
                .map(|r| r.height as f32)
                .unwrap_or(0.0)
        })
        .unwrap_or(0.0);
    let panel_y = tab_height;
    let panel_h = viewport_height - tab_height - status_height;
    let input_height = 32.0;

    // Panel background
    quads.push(QuadInstance {
        rect: [panel_x, panel_y, panel_width, panel_h],
        color: [0.05, 0.05, 0.08, 0.95],
    });
    // Left border glow
    quads.push(QuadInstance {
        rect: [panel_x, panel_y, 2.0, panel_h],
        color: [0.0, 0.83, 1.0, 0.4],
    });
    // Input field background
    quads.push(QuadInstance {
        rect: [
            panel_x + 8.0,
            panel_y + panel_h - input_height - 8.0,
            panel_width - 16.0,
            input_height,
        ],
        color: [0.1, 0.1, 0.14, 0.95],
    });
    // Input field border
    quads.push(QuadInstance {
        rect: [
            panel_x + 8.0,
            panel_y + panel_h - input_height - 8.0,
            panel_width - 16.0,
            1.0,
        ],
        color: [0.0, 0.83, 1.0, 0.3],
    });
}
