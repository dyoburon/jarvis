use glyphon::{
    Attrs, Buffer as TextBuffer, Cache, Color as GlyphonColor, Family, FontSystem, Metrics,
    Resolution, Shaping, SwashCache, TextArea, TextAtlas, TextBounds,
    TextRenderer as GlyphonRenderer, Viewport,
};
use jarvis_terminal::TerminalColor;

// ---------------------------------------------------------------------------
// Standard ANSI 16-color palette
// ---------------------------------------------------------------------------

/// The standard ANSI 16-color palette as (R, G, B) tuples.
pub const ANSI_COLORS: [(u8, u8, u8); 16] = [
    (0, 0, 0),       // 0  Black
    (205, 49, 49),    // 1  Red
    (13, 188, 121),   // 2  Green
    (229, 229, 16),   // 3  Yellow
    (36, 114, 200),   // 4  Blue
    (188, 63, 188),   // 5  Magenta
    (17, 168, 205),   // 6  Cyan
    (229, 229, 229),  // 7  White
    (102, 102, 102),  // 8  Bright Black
    (241, 76, 76),    // 9  Bright Red
    (35, 209, 139),   // 10 Bright Green
    (245, 245, 67),   // 11 Bright Yellow
    (59, 142, 234),   // 12 Bright Blue
    (214, 112, 214),  // 13 Bright Magenta
    (41, 184, 219),   // 14 Bright Cyan
    (255, 255, 255),  // 15 Bright White
];

// ---------------------------------------------------------------------------
// Color conversion
// ---------------------------------------------------------------------------

/// Convert a `TerminalColor` to a glyphon `Color`.
///
/// * `is_fg`: when true, `Default` maps to white; when false, to transparent.
pub fn terminal_color_to_glyphon(color: &TerminalColor, is_fg: bool) -> GlyphonColor {
    match color {
        TerminalColor::Default => {
            if is_fg {
                GlyphonColor::rgba(255, 255, 255, 255)
            } else {
                GlyphonColor::rgba(0, 0, 0, 0)
            }
        }
        TerminalColor::Indexed(idx) => {
            let (r, g, b) = ansi_256_color(*idx);
            GlyphonColor::rgba(r, g, b, 255)
        }
        TerminalColor::Rgb(r, g, b) => GlyphonColor::rgba(*r, *g, *b, 255),
    }
}

/// Look up a color from the ANSI 256-color palette.
///
/// * 0..15   -> standard 16 colors
/// * 16..231 -> 6x6x6 color cube
/// * 232..255 -> grayscale ramp
fn ansi_256_color(idx: u8) -> (u8, u8, u8) {
    if idx < 16 {
        ANSI_COLORS[idx as usize]
    } else if idx < 232 {
        // 6x6x6 color cube: index = 16 + 36*r + 6*g + b where each component is 0..5
        let idx = idx - 16;
        let b = idx % 6;
        let g = (idx / 6) % 6;
        let r = idx / 36;
        let to_channel = |c: u8| -> u8 {
            if c == 0 {
                0
            } else {
                55 + 40 * c
            }
        };
        (to_channel(r), to_channel(g), to_channel(b))
    } else {
        // Grayscale ramp: 232..255 -> 24 shades from dark to light
        let shade = 8 + 10 * (idx - 232);
        (shade, shade, shade)
    }
}

// ---------------------------------------------------------------------------
// TextRenderer
// ---------------------------------------------------------------------------

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

        let renderer = GlyphonRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState::default(),
            None,
        );

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

    /// Prepare all visible rows of the terminal grid for rendering.
    ///
    /// Creates a glyphon `TextBuffer` per visible row with color-batched spans.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_grid(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        grid: &jarvis_terminal::Grid,
        offset_x: f32,
        offset_y: f32,
        viewport_width: f32,
        viewport_height: f32,
        scale_factor: f32,
    ) {
        self.viewport.update(
            queue,
            Resolution {
                width: viewport_width as u32,
                height: viewport_height as u32,
            },
        );

        self.atlas.trim();

        let metrics = Metrics::new(self.font_size, self.line_height);
        let mono_attrs = Attrs::new().family(Family::Monospace);
        let mut buffers: Vec<TextBuffer> = Vec::with_capacity(grid.rows);

        for row_idx in 0..grid.rows {
            let row = &grid.cells[row_idx];
            let mut buffer = TextBuffer::new(&mut self.font_system, metrics);
            buffer.set_size(&mut self.font_system, Some(viewport_width), Some(self.line_height));

            // Build color-batched spans: merge consecutive cells with the same
            // foreground color into a single span to minimize shaping work.
            let mut row_text = String::with_capacity(grid.cols);
            let mut spans: Vec<(usize, usize, GlyphonColor)> = Vec::new();
            let mut current_color: Option<GlyphonColor> = None;
            let mut span_start = 0;

            let mut col = 0;
            while col < row.len() {
                let cell = &row[col];
                if cell.width == 0 {
                    col += 1;
                    continue;
                }
                let fg = if cell.attrs.inverse {
                    terminal_color_to_glyphon(&cell.attrs.bg, false)
                } else {
                    terminal_color_to_glyphon(&cell.attrs.fg, true)
                };

                if let Some(cur) = current_color {
                    if cur != fg {
                        // Close previous span
                        spans.push((span_start, row_text.len(), cur));
                        span_start = row_text.len();
                        current_color = Some(fg);
                    }
                } else {
                    current_color = Some(fg);
                    span_start = row_text.len();
                }

                row_text.push(cell.c);
                col += 1;
            }
            // Close final span
            if let Some(cur) = current_color {
                if row_text.len() > span_start {
                    spans.push((span_start, row_text.len(), cur));
                }
            }

            if spans.is_empty() {
                buffer.set_text(
                    &mut self.font_system,
                    " ",
                    mono_attrs,
                    Shaping::Basic,
                );
            } else {
                let rich: Vec<(&str, Attrs)> = spans
                    .iter()
                    .map(|(s, e, color)| (&row_text[*s..*e], mono_attrs.color(*color)))
                    .collect();
                buffer.set_rich_text(
                    &mut self.font_system,
                    rich,
                    mono_attrs,
                    Shaping::Basic,
                );
            }
            buffer.shape_until_scroll(&mut self.font_system, false);
            buffers.push(buffer);
        }

        let text_areas: Vec<TextArea> = buffers
            .iter()
            .enumerate()
            .map(|(row_idx, buffer)| {
                let top = offset_y + row_idx as f32 * self.line_height;
                TextArea {
                    buffer,
                    left: offset_x,
                    top,
                    scale: scale_factor,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: viewport_width as i32,
                        bottom: viewport_height as i32,
                    },
                    default_color: GlyphonColor::rgba(255, 255, 255, 255),
                    custom_glyphs: &[],
                }
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
            .unwrap_or_else(|e| {
                tracing::warn!("glyphon prepare error: {:?}", e);
            });
    }

    /// Prepare multiple terminal grids for rendering in a single pass.
    ///
    /// Each pane is described by its grid, pixel offset, and clip dimensions.
    /// All text areas are batched into a single `prepare()` call.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_multi_grid(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        panes: &[(&jarvis_terminal::Grid, f32, f32, f32, f32)], // (grid, offset_x, offset_y, pane_w, pane_h)
        extra_text_areas: Vec<TextArea<'_>>,
        viewport_width: f32,
        viewport_height: f32,
        scale_factor: f32,
    ) {
        self.viewport.update(
            queue,
            Resolution {
                width: viewport_width as u32,
                height: viewport_height as u32,
            },
        );

        self.atlas.trim();

        let metrics = Metrics::new(self.font_size, self.line_height);
        let mono_attrs = Attrs::new().family(Family::Monospace);

        // Build buffers for ALL panes' rows.
        // We store (buffer, left, top, bounds) for each row across all panes.
        struct RowInfo {
            left: f32,
            top: f32,
            bounds_left: i32,
            bounds_top: i32,
            bounds_right: i32,
            bounds_bottom: i32,
        }

        let mut all_buffers: Vec<TextBuffer> = Vec::new();
        let mut all_row_info: Vec<RowInfo> = Vec::new();

        for &(grid, offset_x, offset_y, pane_w, pane_h) in panes {
            // How many rows fit in this pane?
            let visible_rows = ((pane_h / self.line_height).floor() as usize).min(grid.rows);

            for row_idx in 0..visible_rows {
                let row = &grid.cells[row_idx];
                let mut buffer = TextBuffer::new(&mut self.font_system, metrics);
                buffer.set_size(&mut self.font_system, Some(pane_w), Some(self.line_height));

                // Build color-batched spans
                let mut row_text = String::with_capacity(grid.cols);
                let mut spans: Vec<(usize, usize, GlyphonColor)> = Vec::new();
                let mut current_color: Option<GlyphonColor> = None;
                let mut span_start = 0;

                let mut col = 0;
                while col < row.len() {
                    let cell = &row[col];
                    if cell.width == 0 {
                        col += 1;
                        continue;
                    }
                    let fg = if cell.attrs.inverse {
                        terminal_color_to_glyphon(&cell.attrs.bg, false)
                    } else {
                        terminal_color_to_glyphon(&cell.attrs.fg, true)
                    };

                    if let Some(cur) = current_color {
                        if cur != fg {
                            spans.push((span_start, row_text.len(), cur));
                            span_start = row_text.len();
                            current_color = Some(fg);
                        }
                    } else {
                        current_color = Some(fg);
                        span_start = row_text.len();
                    }

                    row_text.push(cell.c);
                    col += 1;
                }
                if let Some(cur) = current_color {
                    if row_text.len() > span_start {
                        spans.push((span_start, row_text.len(), cur));
                    }
                }

                if spans.is_empty() {
                    buffer.set_text(
                        &mut self.font_system,
                        " ",
                        mono_attrs,
                        Shaping::Basic,
                    );
                } else {
                    let rich: Vec<(&str, Attrs)> = spans
                        .iter()
                        .map(|(s, e, color)| (&row_text[*s..*e], mono_attrs.color(*color)))
                        .collect();
                    buffer.set_rich_text(
                        &mut self.font_system,
                        rich,
                        mono_attrs,
                        Shaping::Basic,
                    );
                }
                buffer.shape_until_scroll(&mut self.font_system, false);

                all_row_info.push(RowInfo {
                    left: offset_x,
                    top: offset_y + row_idx as f32 * self.line_height,
                    bounds_left: offset_x as i32,
                    bounds_top: offset_y as i32,
                    bounds_right: (offset_x + pane_w) as i32,
                    bounds_bottom: (offset_y + pane_h) as i32,
                });
                all_buffers.push(buffer);
            }
        }

        let mut text_areas: Vec<TextArea> = all_buffers
            .iter()
            .zip(all_row_info.iter())
            .map(|(buffer, info)| TextArea {
                buffer,
                left: info.left,
                top: info.top,
                scale: scale_factor,
                bounds: TextBounds {
                    left: info.bounds_left,
                    top: info.bounds_top,
                    right: info.bounds_right,
                    bottom: info.bounds_bottom,
                },
                default_color: GlyphonColor::rgba(255, 255, 255, 255),
                custom_glyphs: &[],
            })
            .collect();

        // Append extra text areas (status bar, tab bar text, etc.)
        text_areas.extend(extra_text_areas);

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
            .unwrap_or_else(|e| {
                tracing::warn!("glyphon prepare error: {:?}", e);
            });
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

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Measure a single 'M' character to determine monospace cell dimensions.
fn measure_cell(
    font_system: &mut FontSystem,
    _font_family: &str,
    font_size: f32,
    line_height: f32,
) -> (f32, f32) {
    let metrics = Metrics::new(font_size, line_height);
    let mut buffer = TextBuffer::new(font_system, metrics);
    buffer.set_size(font_system, Some(font_size * 10.0), Some(line_height * 2.0));
    buffer.set_text(
        font_system,
        "M",
        Attrs::new().family(Family::Monospace),
        Shaping::Advanced,
    );
    buffer.shape_until_scroll(font_system, false);

    // Walk layout runs to find the advance width of the glyph.
    let mut width = font_size * 0.6; // sensible fallback
    if let Some(run) = buffer.layout_runs().next() {
        if let Some(glyph) = run.glyphs.iter().next() {
            width = glyph.w;
        }
    }

    (width, line_height)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_palette_has_16_entries() {
        assert_eq!(ANSI_COLORS.len(), 16);
    }

    #[test]
    fn terminal_color_default_fg_is_white() {
        let color = terminal_color_to_glyphon(&TerminalColor::Default, true);
        assert_eq!(color, GlyphonColor::rgba(255, 255, 255, 255));
    }

    #[test]
    fn terminal_color_default_bg_is_transparent() {
        let color = terminal_color_to_glyphon(&TerminalColor::Default, false);
        assert_eq!(color, GlyphonColor::rgba(0, 0, 0, 0));
    }

    #[test]
    fn terminal_color_indexed_maps_correctly() {
        // Index 0 = black
        let color = terminal_color_to_glyphon(&TerminalColor::Indexed(0), true);
        assert_eq!(color, GlyphonColor::rgba(0, 0, 0, 255));

        // Index 1 = red
        let color = terminal_color_to_glyphon(&TerminalColor::Indexed(1), true);
        assert_eq!(color, GlyphonColor::rgba(205, 49, 49, 255));

        // Index 7 = white
        let color = terminal_color_to_glyphon(&TerminalColor::Indexed(7), true);
        assert_eq!(color, GlyphonColor::rgba(229, 229, 229, 255));

        // Index 15 = bright white
        let color = terminal_color_to_glyphon(&TerminalColor::Indexed(15), true);
        assert_eq!(color, GlyphonColor::rgba(255, 255, 255, 255));
    }

    #[test]
    fn terminal_color_rgb_maps_directly() {
        let color = terminal_color_to_glyphon(&TerminalColor::Rgb(128, 64, 32), true);
        assert_eq!(color, GlyphonColor::rgba(128, 64, 32, 255));
    }

    #[test]
    fn ansi_256_color_cube_index_16_is_black() {
        // Index 16 = r=0, g=0, b=0 in the 6x6x6 cube -> (0, 0, 0)
        let (r, g, b) = ansi_256_color(16);
        assert_eq!((r, g, b), (0, 0, 0));
    }

    #[test]
    fn ansi_256_color_cube_index_231_is_white() {
        // Index 231 = r=5, g=5, b=5 -> (255, 255, 255)
        let (r, g, b) = ansi_256_color(231);
        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn ansi_256_grayscale_ramp() {
        // Index 232 = first grayscale = 8 + 10*0 = 8
        let (r, g, b) = ansi_256_color(232);
        assert_eq!((r, g, b), (8, 8, 8));

        // Index 255 = last grayscale = 8 + 10*23 = 238
        let (r, g, b) = ansi_256_color(255);
        assert_eq!((r, g, b), (238, 238, 238));
    }

    #[test]
    fn indexed_bright_colors_in_range() {
        for idx in 8u8..16 {
            let color = terminal_color_to_glyphon(&TerminalColor::Indexed(idx), true);
            // Should produce valid non-transparent colors
            let (r, g, b) = ANSI_COLORS[idx as usize];
            assert_eq!(color, GlyphonColor::rgba(r, g, b, 255));
        }
    }
}
