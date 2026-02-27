use std::collections::HashMap;

use glyphon::{
    Attrs, Buffer as TextBuffer, Cache, Color as GlyphonColor, Family, FontSystem, Metrics,
    Shaping, SwashCache, TextAtlas, TextRenderer as GlyphonRenderer, Viewport,
};

use super::helpers::measure_cell;

/// GPU text renderer backed by glyphon.
pub struct TextRenderer {
    pub font_system: FontSystem,
    pub swash_cache: SwashCache,
    pub cache: Cache,
    pub atlas: TextAtlas,
    pub viewport: Viewport,
    pub renderer: GlyphonRenderer,
    pub cell_width: f32,
    pub cell_height: f32,
    pub font_size: f32,
    pub line_height: f32,
    /// Cached text buffers per pane for incremental rendering.
    pub(crate) pane_buffer_cache: HashMap<u32, Vec<TextBuffer>>,
}

impl TextRenderer {
    /// Create a new text renderer backed by glyphon, using system fonts.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        font_family: &str,
        font_size: f32,
        line_height: f32,
    ) -> Self {
        let mut font_system = FontSystem::new();

        let swash_cache = SwashCache::new();

        let cache = Cache::new(device);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let viewport = Viewport::new(device, &cache);

        let renderer =
            GlyphonRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        // line_height from config is a multiplier (e.g. 1.2); convert to pixels.
        let line_height_px = font_size * line_height;

        // Measure a single cell to determine grid dimensions.
        let (cell_width, cell_height) =
            measure_cell(&mut font_system, font_family, font_size, line_height_px);

        tracing::info!(
            "TextRenderer: font_size={font_size}, line_height_px={line_height_px}, cell=({cell_width:.1}x{cell_height:.1})"
        );

        Self {
            font_system,
            swash_cache,
            cache,
            atlas,
            viewport,
            renderer,
            cell_width,
            cell_height,
            font_size,
            line_height: line_height_px,
            pane_buffer_cache: HashMap::new(),
        }
    }

    /// Re-measure cell dimensions (e.g., after font change).
    pub fn measure_cell(&mut self) -> (f32, f32) {
        let (w, h) = measure_cell(
            &mut self.font_system,
            "monospace",
            self.font_size,
            self.line_height,
        );
        self.cell_width = w;
        self.cell_height = h;
        (w, h)
    }

    /// Remove cached buffers for a specific pane (e.g., after resize).
    pub fn invalidate_pane_cache(&mut self, pane_id: u32) {
        self.pane_buffer_cache.remove(&pane_id);
    }

    /// Create a text buffer for a single line of UI text (status bar, tab bar).
    ///
    /// Returns a `TextBuffer` that can be used in `extra_text_areas`.
    pub fn create_ui_text_buffer(
        &mut self,
        text: &str,
        color: GlyphonColor,
        width: f32,
    ) -> TextBuffer {
        let metrics = Metrics::new(self.font_size, self.line_height);
        let attrs = Attrs::new().family(Family::Monospace).color(color);
        let mut buffer = TextBuffer::new(&mut self.font_system, metrics);
        buffer.set_size(&mut self.font_system, Some(width), Some(self.line_height));
        buffer.set_text(&mut self.font_system, text, attrs, Shaping::Basic);
        buffer.shape_until_scroll(&mut self.font_system, false);
        buffer
    }

    /// Render the previously prepared text into the given render pass.
    pub fn render<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .unwrap_or_else(|e| {
                tracing::warn!("glyphon render error: {:?}", e);
            });
    }
}
