use glyphon::{
    Attrs, Buffer as TextBuffer, Color as GlyphonColor, Family, Metrics, Resolution, Shaping,
    TextArea, TextBounds,
};

use jarvis_terminal::{Cell, CellFlags, Colors, Column, Dimensions, Grid, Line};

use super::colors::vte_color_to_glyphon;
use super::renderer::TextRenderer;

impl TextRenderer {
    /// Prepare all visible rows of the terminal grid for rendering.
    ///
    /// Creates a glyphon `TextBuffer` per visible row with color-batched spans.
    #[allow(clippy::too_many_arguments)]
    pub fn prepare_grid(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        grid: &Grid<Cell>,
        colors: &Colors,
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
        let screen_lines = grid.screen_lines();
        let cols = grid.columns();
        let mut buffers: Vec<TextBuffer> = Vec::with_capacity(screen_lines);

        for row_idx in 0..screen_lines {
            let row = &grid[Line(row_idx as i32)];
            let mut buffer = TextBuffer::new(&mut self.font_system, metrics);
            buffer.set_size(
                &mut self.font_system,
                Some(viewport_width),
                Some(self.line_height),
            );

            // Build color-batched spans: merge consecutive cells with the same
            // foreground color into a single span to minimize shaping work.
            let mut row_text = String::with_capacity(cols);
            let mut spans: Vec<(usize, usize, GlyphonColor)> = Vec::new();
            let mut current_color: Option<GlyphonColor> = None;
            let mut span_start = 0;

            for col_idx in 0..cols {
                let cell = &row[Column(col_idx)];
                // Skip wide-char spacers (the trailing cell of a double-width char).
                if cell.flags.contains(CellFlags::WIDE_CHAR_SPACER) {
                    continue;
                }
                let is_inverse = cell.flags.contains(CellFlags::INVERSE);
                let fg = if is_inverse {
                    vte_color_to_glyphon(cell.bg, colors, false)
                } else {
                    vte_color_to_glyphon(cell.fg, colors, true)
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
            }
            // Close final span
            if let Some(cur) = current_color {
                if row_text.len() > span_start {
                    spans.push((span_start, row_text.len(), cur));
                }
            }

            if spans.is_empty() {
                buffer.set_text(&mut self.font_system, " ", mono_attrs, Shaping::Basic);
            } else {
                let rich: Vec<(&str, Attrs)> = spans
                    .iter()
                    .map(|(s, e, color)| (&row_text[*s..*e], mono_attrs.color(*color)))
                    .collect();
                buffer.set_rich_text(&mut self.font_system, rich, mono_attrs, Shaping::Basic);
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

    /// Prepare multiple terminal grids for rendering, only reshaping dirty rows.
    ///
    /// Each pane is described by its id, grid, colors, dirty flags, pixel offset,
    /// and clip dimensions. Cached `TextBuffer`s are reused for clean rows.
    #[allow(clippy::too_many_arguments, clippy::type_complexity)]
    pub fn prepare_multi_grid(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        panes: &[(u32, &Grid<Cell>, &Colors, &[bool], f32, f32, f32, f32)],
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

        struct RowInfo {
            left: f32,
            top: f32,
            bounds_left: i32,
            bounds_top: i32,
            bounds_right: i32,
            bounds_bottom: i32,
        }

        let mut all_row_info: Vec<(u32, usize)> = Vec::new(); // (pane_id, row_idx)
        let mut info_list: Vec<RowInfo> = Vec::new();
        let mut active_pane_ids: Vec<u32> = Vec::new();

        // First pass: update dirty buffers in the cache
        for &(pane_id, grid, colors, dirty_rows, offset_x, offset_y, pane_w, pane_h) in panes {
            active_pane_ids.push(pane_id);
            let screen_lines = grid.screen_lines();
            let cols = grid.columns();
            let visible_rows = ((pane_h / self.line_height).floor() as usize).min(screen_lines);

            let cached = self.pane_buffer_cache.entry(pane_id).or_default();

            // If cache size doesn't match, invalidate entirely
            if cached.len() != visible_rows {
                cached.clear();
                for _ in 0..visible_rows {
                    let mut buffer = TextBuffer::new(&mut self.font_system, metrics);
                    buffer.set_size(&mut self.font_system, Some(pane_w), Some(self.line_height));
                    buffer.set_text(&mut self.font_system, " ", mono_attrs, Shaping::Basic);
                    buffer.shape_until_scroll(&mut self.font_system, false);
                    cached.push(buffer);
                }
            }

            for (row_idx, buffer) in cached.iter_mut().enumerate().take(visible_rows) {
                let is_dirty = dirty_rows.get(row_idx).copied().unwrap_or(true);

                if is_dirty {
                    let row = &grid[Line(row_idx as i32)];
                    buffer.set_size(&mut self.font_system, Some(pane_w), Some(self.line_height));

                    // Build color-batched spans
                    let mut row_text = String::with_capacity(cols);
                    let mut spans: Vec<(usize, usize, GlyphonColor)> = Vec::new();
                    let mut current_color: Option<GlyphonColor> = None;
                    let mut span_start = 0;

                    for col_idx in 0..cols {
                        let cell = &row[Column(col_idx)];
                        if cell.flags.contains(CellFlags::WIDE_CHAR_SPACER) {
                            continue;
                        }
                        let is_inverse = cell.flags.contains(CellFlags::INVERSE);
                        let fg = if is_inverse {
                            vte_color_to_glyphon(cell.bg, colors, false)
                        } else {
                            vte_color_to_glyphon(cell.fg, colors, true)
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
                    }
                    if let Some(cur) = current_color {
                        if row_text.len() > span_start {
                            spans.push((span_start, row_text.len(), cur));
                        }
                    }

                    if spans.is_empty() {
                        buffer.set_text(&mut self.font_system, " ", mono_attrs, Shaping::Basic);
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
                }

                all_row_info.push((pane_id, row_idx));
                info_list.push(RowInfo {
                    left: offset_x,
                    top: offset_y + row_idx as f32 * self.line_height,
                    bounds_left: offset_x as i32,
                    bounds_top: offset_y as i32,
                    bounds_right: (offset_x + pane_w) as i32,
                    bounds_bottom: (offset_y + pane_h) as i32,
                });
            }
        }

        // Evict cache entries for closed panes
        self.pane_buffer_cache
            .retain(|id, _| active_pane_ids.contains(id));

        // Second pass: build TextArea references from cached buffers
        let mut text_areas: Vec<TextArea> = all_row_info
            .iter()
            .zip(info_list.iter())
            .filter_map(|(&(pane_id, row_idx), info)| {
                let cached = self.pane_buffer_cache.get(&pane_id)?;
                let buffer = cached.get(row_idx)?;
                Some(TextArea {
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
            })
            .collect();

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
}
