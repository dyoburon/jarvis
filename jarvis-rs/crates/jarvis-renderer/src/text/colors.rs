use glyphon::Color as GlyphonColor;
use jarvis_terminal::TerminalColor;

/// The standard ANSI 16-color palette as (R, G, B) tuples.
pub const ANSI_COLORS: [(u8, u8, u8); 16] = [
    (0, 0, 0),       // 0  Black
    (205, 49, 49),   // 1  Red
    (13, 188, 121),  // 2  Green
    (229, 229, 16),  // 3  Yellow
    (36, 114, 200),  // 4  Blue
    (188, 63, 188),  // 5  Magenta
    (17, 168, 205),  // 6  Cyan
    (229, 229, 229), // 7  White
    (102, 102, 102), // 8  Bright Black
    (241, 76, 76),   // 9  Bright Red
    (35, 209, 139),  // 10 Bright Green
    (245, 245, 67),  // 11 Bright Yellow
    (59, 142, 234),  // 12 Bright Blue
    (214, 112, 214), // 13 Bright Magenta
    (41, 184, 219),  // 14 Bright Cyan
    (255, 255, 255), // 15 Bright White
];

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
pub(crate) fn ansi_256_color(idx: u8) -> (u8, u8, u8) {
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
