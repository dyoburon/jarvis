use glyphon::Color as GlyphonColor;
use jarvis_terminal::{Colors, VteColor};

/// Convert a VTE `Color` to a glyphon `Color` using the terminal's color palette.
///
/// * `is_fg`: when true, `Named(Foreground)` maps to white; when false,
///   `Named(Background)` maps to transparent.
pub fn vte_color_to_glyphon(color: VteColor, colors: &Colors, is_fg: bool) -> GlyphonColor {
    let [r, g, b, a] = jarvis_terminal::vte_color_to_rgba(&color, colors, is_fg);
    GlyphonColor::rgba(r, g, b, a)
}
