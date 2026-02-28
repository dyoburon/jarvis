use std::sync::Arc;
use winit::window::Window;

use glyphon::Buffer as TextBuffer;
use jarvis_common::types::Rect;

use crate::gpu::{GpuContext, RendererError};
use crate::quad::QuadRenderer;
use crate::text::TextRenderer;

/// Core rendering state holding GPU context, text renderer, and quad renderer.
pub struct RenderState {
    pub gpu: GpuContext,
    pub text: TextRenderer,
    pub quad: QuadRenderer,
    pub clear_color: wgpu::Color,
    // Cached chrome text buffers
    pub(crate) cached_status_left: Option<(String, TextBuffer)>,
    pub(crate) cached_status_center: Option<(String, TextBuffer)>,
    pub(crate) cached_status_right: Option<(String, TextBuffer)>,
    pub(crate) cached_tab_buffers: Vec<(String, bool, TextBuffer)>,
    // Cached assistant panel text buffers (rebuilt each frame when open)
    pub(crate) cached_assistant_buffers: Vec<TextBuffer>,
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
}
