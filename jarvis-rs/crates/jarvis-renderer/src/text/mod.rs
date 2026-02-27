mod colors;
mod helpers;
mod prepare;
mod renderer;

pub use colors::*;
pub use renderer::*;

#[cfg(test)]
mod tests {
    use super::*;
    use glyphon::Color as GlyphonColor;
    use jarvis_terminal::TerminalColor;

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
        let (r, g, b) = colors::ansi_256_color(16);
        assert_eq!((r, g, b), (0, 0, 0));
    }

    #[test]
    fn ansi_256_color_cube_index_231_is_white() {
        // Index 231 = r=5, g=5, b=5 -> (255, 255, 255)
        let (r, g, b) = colors::ansi_256_color(231);
        assert_eq!((r, g, b), (255, 255, 255));
    }

    #[test]
    fn ansi_256_grayscale_ramp() {
        // Index 232 = first grayscale = 8 + 10*0 = 8
        let (r, g, b) = colors::ansi_256_color(232);
        assert_eq!((r, g, b), (8, 8, 8));

        // Index 255 = last grayscale = 8 + 10*23 = 238
        let (r, g, b) = colors::ansi_256_color(255);
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
