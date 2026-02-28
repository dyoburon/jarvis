use glyphon::{Attrs, Buffer as TextBuffer, Family, FontSystem, Metrics, Shaping};

/// Measure a single 'M' character to determine monospace cell dimensions.
pub(crate) fn measure_cell(
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
