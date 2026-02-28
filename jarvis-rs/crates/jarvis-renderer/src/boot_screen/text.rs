//! Thin wrapper around glyphon for GPU text rendering.
//!
//! Provides a simplified API: create once, prepare text areas each frame,
//! render into an existing render pass. All glyphon internals are hidden.

use crate::gpu::RendererError;
use glyphon::{
    Buffer, Cache, Color, FontSystem, Metrics, Resolution, SwashCache, TextArea, TextAtlas,
    TextBounds, TextRenderer, Viewport,
};

/// A single text item to render in one frame.
pub struct TextEntry<'a> {
    /// The text content to display.
    pub text: &'a str,
    /// Left edge in pixels.
    pub left: f32,
    /// Top edge in pixels.
    pub top: f32,
    /// Font size in pixels.
    pub font_size: f32,
    /// Line height in pixels.
    pub line_height: f32,
    /// RGBA color (0–255 per channel).
    pub color: Color,
    /// Maximum width before wrapping (pixels). `None` = no wrap.
    pub max_width: Option<f32>,
}

/// Manages glyphon resources for rendering text on the GPU.
///
/// Owns the font system, swash cache, text atlas, viewport, and
/// text renderer. Call [`prepare`] each frame with the text entries
/// you want, then [`render`] inside a render pass.
pub struct BootTextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: TextAtlas,
    viewport: Viewport,
    renderer: TextRenderer,
    /// Reusable buffer pool — grown as needed, never shrunk.
    buffers: Vec<Buffer>,
}

impl BootTextRenderer {
    /// Create a new text renderer tied to the given GPU device and format.
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, format);
        let renderer =
            TextRenderer::new(&mut atlas, device, wgpu::MultisampleState::default(), None);

        Self {
            font_system,
            swash_cache,
            atlas,
            viewport,
            renderer,
            buffers: Vec::new(),
        }
    }

    /// Prepare text entries for rendering this frame.
    ///
    /// Must be called before [`render`]. Replaces any previously
    /// prepared text.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        width: u32,
        height: u32,
        entries: &[TextEntry<'_>],
    ) -> Result<(), RendererError> {
        self.viewport.update(queue, Resolution { width, height });

        // Ensure we have enough buffers.
        while self.buffers.len() < entries.len() {
            let buf = Buffer::new(
                &mut self.font_system,
                Metrics::new(16.0, 20.0), // placeholder, overwritten below
            );
            self.buffers.push(buf);
        }

        // Configure each buffer with the entry's text and metrics.
        for (i, entry) in entries.iter().enumerate() {
            let buf = &mut self.buffers[i];
            buf.set_metrics(
                &mut self.font_system,
                Metrics::new(entry.font_size, entry.line_height),
            );
            let max_w = entry.max_width.unwrap_or(width as f32);
            buf.set_size(&mut self.font_system, Some(max_w), None);
            buf.set_text(
                &mut self.font_system,
                entry.text,
                glyphon::Attrs::new().family(glyphon::Family::Monospace),
                glyphon::Shaping::Basic,
            );
            buf.shape_until_scroll(&mut self.font_system, false);
        }

        // Build TextArea slice for glyphon.
        let text_areas: Vec<TextArea<'_>> = entries
            .iter()
            .enumerate()
            .map(|(i, entry)| TextArea {
                buffer: &self.buffers[i],
                left: entry.left,
                top: entry.top,
                scale: 1.0,
                bounds: TextBounds {
                    left: 0,
                    top: 0,
                    right: width as i32,
                    bottom: height as i32,
                },
                default_color: entry.color,
                custom_glyphs: &[],
            })
            .collect();

        self.renderer
            .prepare(
                device,
                queue,
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                text_areas,
                &mut self.swash_cache,
            )
            .map_err(|e| RendererError::TextError(e.to_string()))?;

        Ok(())
    }

    /// Render previously prepared text into the given render pass.
    pub fn render<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
    ) -> Result<(), RendererError> {
        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .map_err(|e| RendererError::TextError(e.to_string()))?;
        Ok(())
    }

    /// Trim the atlas cache. Call periodically (e.g. once per second)
    /// to free unused glyph allocations.
    pub fn trim_atlas(&mut self) {
        self.atlas.trim();
    }
}
