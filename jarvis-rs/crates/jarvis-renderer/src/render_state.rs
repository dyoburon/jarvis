use std::sync::Arc;
use winit::window::Window;

use glyphon::{Color as GlyphonColor, TextArea, TextBounds};
use jarvis_common::types::Rect;

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

        let mut encoder =
            self.gpu
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
        panes: &[(u32, Rect, &jarvis_terminal::Grid)],
        focused_id: u32,
        chrome: &UiChrome,
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

        let mut encoder =
            self.gpu
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
            for &(pane_id, ref rect, _) in panes {
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

        self.quad
            .prepare(&self.gpu.queue, &quads, viewport_width, viewport_height);

        // 2. Build text for terminal panes
        let pane_data: Vec<(&jarvis_terminal::Grid, f32, f32, f32, f32)> = panes
            .iter()
            .map(|(_, rect, grid)| {
                (
                    *grid,
                    rect.x as f32,
                    rect.y as f32,
                    rect.width as f32,
                    rect.height as f32,
                )
            })
            .collect();

        // 3. Build extra text areas for UI chrome text
        let mut ui_buffers = Vec::new();

        // Status bar text
        if let Some(ref sb) = chrome.status_bar {
            if let Some(sb_rect) = chrome.status_bar_rect(viewport_width, viewport_height) {
                let fg = GlyphonColor::rgba(
                    (sb.fg_color[0] * 255.0) as u8,
                    (sb.fg_color[1] * 255.0) as u8,
                    (sb.fg_color[2] * 255.0) as u8,
                    (sb.fg_color[3] * 255.0) as u8,
                );
                // Left-aligned text
                if !sb.left_text.is_empty() {
                    let buf = self.text.create_ui_text_buffer(
                        &sb.left_text,
                        fg,
                        sb_rect.width as f32 / 3.0,
                    );
                    ui_buffers.push((buf, sb_rect.x as f32 + 8.0, sb_rect.y as f32 + 2.0, sb_rect));
                }
                // Center text
                if !sb.center_text.is_empty() {
                    let buf = self.text.create_ui_text_buffer(
                        &sb.center_text,
                        fg,
                        sb_rect.width as f32 / 3.0,
                    );
                    ui_buffers.push((
                        buf,
                        sb_rect.x as f32 + sb_rect.width as f32 / 3.0,
                        sb_rect.y as f32 + 2.0,
                        sb_rect,
                    ));
                }
                // Right-aligned text
                if !sb.right_text.is_empty() {
                    let buf = self.text.create_ui_text_buffer(
                        &sb.right_text,
                        fg,
                        sb_rect.width as f32 / 3.0,
                    );
                    ui_buffers.push((
                        buf,
                        sb_rect.x as f32 + sb_rect.width as f32 * 2.0 / 3.0,
                        sb_rect.y as f32 + 2.0,
                        sb_rect,
                    ));
                }
            }
        }

        // Tab bar text
        if let Some(ref tb) = chrome.tab_bar {
            if let Some(tb_rect) = chrome.tab_bar_rect(viewport_width) {
                let tab_width = if tb.tabs.is_empty() {
                    tb_rect.width as f32
                } else {
                    (tb_rect.width as f32 / tb.tabs.len() as f32).min(200.0)
                };
                for (i, tab) in tb.tabs.iter().enumerate() {
                    let color = if tab.is_active {
                        GlyphonColor::rgba(255, 255, 255, 255)
                    } else {
                        GlyphonColor::rgba(150, 150, 150, 200)
                    };
                    let buf = self
                        .text
                        .create_ui_text_buffer(&tab.title, color, tab_width);
                    ui_buffers.push((
                        buf,
                        tb_rect.x as f32 + i as f32 * tab_width + 8.0,
                        tb_rect.y as f32 + 6.0,
                        tb_rect,
                    ));
                }
            }
        }

        let extra_text_areas: Vec<TextArea> = ui_buffers
            .iter()
            .map(|(buf, left, top, bounds_rect)| TextArea {
                buffer: buf,
                left: *left,
                top: *top,
                scale: scale_factor,
                bounds: TextBounds {
                    left: bounds_rect.x as i32,
                    top: bounds_rect.y as i32,
                    right: (bounds_rect.x + bounds_rect.width) as i32,
                    bottom: (bounds_rect.y + bounds_rect.height) as i32,
                },
                default_color: GlyphonColor::rgba(255, 255, 255, 255),
                custom_glyphs: &[],
            })
            .collect();

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
    static PRESENTED: std::sync::atomic::AtomicBool =
        std::sync::atomic::AtomicBool::new(false);
    if !PRESENTED.swap(true, std::sync::atomic::Ordering::Relaxed) {
        tracing::info!(
            "First frame presented ({}x{}, format={:?})",
            width,
            height,
            format,
        );
    }
}
