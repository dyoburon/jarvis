use std::sync::Arc;
use winit::window::Window;

use glyphon::{Buffer as TextBuffer, Color as GlyphonColor, TextArea, TextBounds};
use jarvis_common::types::Rect;

use crate::assistant_panel::{AssistantPanel, ChatRole};
use crate::gpu::{GpuContext, RendererError};
use crate::quad::{QuadInstance, QuadRenderer};
use crate::text::TextRenderer;
use crate::ui::UiChrome;

// ---------------------------------------------------------------------------
// RenderState
// ---------------------------------------------------------------------------

pub struct RenderState {
    pub gpu: GpuContext,
    pub text: TextRenderer,
    pub quad: QuadRenderer,
    pub clear_color: wgpu::Color,
    // Cached chrome text buffers
    cached_status_left: Option<(String, TextBuffer)>,
    cached_status_center: Option<(String, TextBuffer)>,
    cached_status_right: Option<(String, TextBuffer)>,
    cached_tab_buffers: Vec<(String, bool, TextBuffer)>,
    // Cached assistant panel text buffers (rebuilt each frame when open)
    cached_assistant_buffers: Vec<TextBuffer>,
}

impl RenderState {
    /// Create a fully initialized render state from a window.
    pub async fn new(
        window: Arc<Window>,
        font_family: &str,
        font_size: f32,
        line_height: f32,
    ) -> Result<Self, RendererError> {
        let gpu = GpuContext::new(window).await?;

        let text = TextRenderer::new(
            &gpu.device,
            &gpu.queue,
            gpu.format(),
            font_family,
            font_size,
            line_height,
        );

        let quad = QuadRenderer::new(&gpu.device, gpu.format());

        Ok(Self {
            gpu,
            text,
            quad,
            clear_color: wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            cached_status_left: None,
            cached_status_center: None,
            cached_status_right: None,
            cached_tab_buffers: Vec::new(),
            cached_assistant_buffers: Vec::new(),
        })
    }

    /// Handle a window resize by reconfiguring the surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.gpu.resize(width, height);
    }

    /// Returns (cell_width, cell_height) for the current font configuration.
    pub fn cell_size(&self) -> (f32, f32) {
        (self.text.cell_width, self.text.cell_height)
    }

    /// Calculate terminal grid dimensions for a given pixel area.
    pub fn grid_dimensions_for_rect(&self, rect: &Rect) -> (usize, usize) {
        let (cell_w, cell_h) = self.cell_size();
        if cell_w <= 0.0 || cell_h <= 0.0 {
            return (1, 1);
        }
        let cols = (rect.width as f32 / cell_w).floor().max(1.0) as usize;
        let rows = (rect.height as f32 / cell_h).floor().max(1.0) as usize;
        (cols, rows)
    }

    /// Calculate terminal grid dimensions (cols, rows) based on window size and
    /// cell size.
    pub fn grid_dimensions(&self) -> (usize, usize) {
        let (cell_w, cell_h) = self.cell_size();
        if cell_w <= 0.0 || cell_h <= 0.0 {
            return (1, 1);
        }
        let cols = (self.gpu.size.width as f32 / cell_w).floor().max(1.0) as usize;
        let rows = (self.gpu.size.height as f32 / cell_h).floor().max(1.0) as usize;
        (cols, rows)
    }

    /// Set the background clear color for frame rendering.
    pub fn set_clear_color(&mut self, r: f64, g: f64, b: f64) {
        self.clear_color = wgpu::Color { r, g, b, a: 1.0 };
    }

    /// Render a complete frame: clear, prepare text, draw.
    pub fn render_frame(&mut self, grid: &jarvis_terminal::Grid) -> Result<(), RendererError> {
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
                label: Some("jarvis frame encoder"),
            });

        let viewport_width = self.gpu.size.width as f32;
        let viewport_height = self.gpu.size.height as f32;
        let scale_factor = self.gpu.scale_factor as f32;

        self.text.prepare_grid(
            &self.gpu.device,
            &self.gpu.queue,
            grid,
            0.0,
            0.0,
            viewport_width,
            viewport_height,
            scale_factor,
        );

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("jarvis main pass"),
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

            self.text.render(&mut pass);
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        log_first_frame(self.gpu.size.width, self.gpu.size.height, self.gpu.format());

        Ok(())
    }

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

        // 3. Update cached UI chrome text buffers (only rebuild when content changes)

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

        // Build extra text areas from cached buffers
        let mut extra_text_areas: Vec<TextArea> = Vec::new();

        if let Some(ref sb) = chrome.status_bar {
            if let Some(sb_rect) = chrome.status_bar_rect(viewport_width, viewport_height) {
                if !sb.left_text.is_empty() {
                    if let Some((_, ref buf)) = self.cached_status_left {
                        extra_text_areas.push(TextArea {
                            buffer: buf,
                            left: sb_rect.x as f32 + 8.0,
                            top: sb_rect.y as f32 + 2.0,
                            scale: scale_factor,
                            bounds: TextBounds {
                                left: sb_rect.x as i32,
                                top: sb_rect.y as i32,
                                right: (sb_rect.x + sb_rect.width) as i32,
                                bottom: (sb_rect.y + sb_rect.height) as i32,
                            },
                            default_color: GlyphonColor::rgba(255, 255, 255, 255),
                            custom_glyphs: &[],
                        });
                    }
                }
                if !sb.center_text.is_empty() {
                    if let Some((_, ref buf)) = self.cached_status_center {
                        extra_text_areas.push(TextArea {
                            buffer: buf,
                            left: sb_rect.x as f32 + sb_rect.width as f32 / 3.0,
                            top: sb_rect.y as f32 + 2.0,
                            scale: scale_factor,
                            bounds: TextBounds {
                                left: sb_rect.x as i32,
                                top: sb_rect.y as i32,
                                right: (sb_rect.x + sb_rect.width) as i32,
                                bottom: (sb_rect.y + sb_rect.height) as i32,
                            },
                            default_color: GlyphonColor::rgba(255, 255, 255, 255),
                            custom_glyphs: &[],
                        });
                    }
                }
                if !sb.right_text.is_empty() {
                    if let Some((_, ref buf)) = self.cached_status_right {
                        extra_text_areas.push(TextArea {
                            buffer: buf,
                            left: sb_rect.x as f32 + sb_rect.width as f32 * 2.0 / 3.0,
                            top: sb_rect.y as f32 + 2.0,
                            scale: scale_factor,
                            bounds: TextBounds {
                                left: sb_rect.x as i32,
                                top: sb_rect.y as i32,
                                right: (sb_rect.x + sb_rect.width) as i32,
                                bottom: (sb_rect.y + sb_rect.height) as i32,
                            },
                            default_color: GlyphonColor::rgba(255, 255, 255, 255),
                            custom_glyphs: &[],
                        });
                    }
                }
            }
        }

        if let Some(ref tb) = chrome.tab_bar {
            if let Some(tb_rect) = chrome.tab_bar_rect(viewport_width) {
                let tab_width = if tb.tabs.is_empty() {
                    tb_rect.width as f32
                } else {
                    (tb_rect.width as f32 / tb.tabs.len() as f32).min(200.0)
                };
                for (i, (_, _, ref buf)) in self.cached_tab_buffers.iter().enumerate() {
                    extra_text_areas.push(TextArea {
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

        // Assistant panel text
        self.cached_assistant_buffers.clear();
        if let Some(panel) = assistant {
            let panel_width = viewport_width * 0.4;
            let panel_x = viewport_width - panel_width;
            let text_width = panel_width - 32.0;
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
            let line_h = self.text.cell_height;

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

            // Now build TextArea references from cached buffers
            let panel_bounds = TextBounds {
                left: panel_x as i32,
                top: panel_y as i32,
                right: (panel_x + panel_width) as i32,
                bottom: (panel_y + panel_h) as i32,
            };

            let mut buf_idx = 0;

            // Title
            extra_text_areas.push(TextArea {
                buffer: &self.cached_assistant_buffers[buf_idx],
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
            extra_text_areas.push(TextArea {
                buffer: &self.cached_assistant_buffers[buf_idx],
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
                y_cursor -= line_h;
                if y_cursor > panel_y + 30.0 {
                    extra_text_areas.push(TextArea {
                        buffer: &self.cached_assistant_buffers[buf_idx],
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
            // Count remaining message + streaming buffers
            let remaining = self.cached_assistant_buffers.len() - buf_idx;
            // Render bottom-up: messages are stored most-recent-first
            for i in 0..remaining {
                let est_chars = if buf_idx + i < self.cached_assistant_buffers.len() {
                    // Estimate lines from buffer; rough heuristic
                    let buf = &self.cached_assistant_buffers[buf_idx + i];
                    let line_count = buf.layout_runs().count().max(1);
                    line_count
                } else {
                    1
                };
                let text_h = est_chars as f32 * line_h;
                y_cursor -= text_h + 4.0; // 4px spacing between messages
                if y_cursor < panel_y + 30.0 {
                    break; // off screen
                }
                extra_text_areas.push(TextArea {
                    buffer: &self.cached_assistant_buffers[buf_idx + i],
                    left: panel_x + 16.0,
                    top: y_cursor,
                    scale: scale_factor,
                    bounds: panel_bounds,
                    default_color: GlyphonColor::rgba(220, 220, 220, 255),
                    custom_glyphs: &[],
                });
            }
        }

        self.text.prepare_multi_grid(
            &self.gpu.device,
            &self.gpu.queue,
            &pane_data,
            extra_text_areas,
            viewport_width,
            viewport_height,
            scale_factor,
        );

        // 4. Render pass
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

fn log_first_frame(width: u32, height: u32, format: wgpu::TextureFormat) {
    static PRESENTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
    if !PRESENTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        tracing::info!(
            "First frame presented ({}x{}, format={:?})",
            width,
            height,
            format,
        );
    }
}
