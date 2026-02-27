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
    use jarvis_terminal::{Colors, NamedColor, VteColor, VteRgb};

    fn default_colors() -> Colors {
        Colors::default()
    }

    #[test]
    fn vte_color_default_fg_is_white() {
        let colors = default_colors();
        let color = vte_color_to_glyphon(VteColor::Named(NamedColor::Foreground), &colors, true);
        assert_eq!(color, GlyphonColor::rgba(255, 255, 255, 255));
    }

    #[test]
    fn vte_color_default_bg_is_transparent() {
        let colors = default_colors();
        let color = vte_color_to_glyphon(VteColor::Named(NamedColor::Background), &colors, false);
        assert_eq!(color, GlyphonColor::rgba(0, 0, 0, 0));
    }

    #[test]
    fn vte_color_named_red() {
        let colors = default_colors();
        let color = vte_color_to_glyphon(VteColor::Named(NamedColor::Red), &colors, true);
        // Red from our ANSI palette: (0xCD, 0x00, 0x00) = (205, 0, 0)
        assert_eq!(color, GlyphonColor::rgba(205, 0, 0, 255));
    }

    #[test]
    fn vte_color_indexed_0_is_black() {
        let colors = default_colors();
        let color = vte_color_to_glyphon(VteColor::Indexed(0), &colors, true);
        assert_eq!(color, GlyphonColor::rgba(0, 0, 0, 255));
    }

    #[test]
    fn vte_color_indexed_15_is_bright_white() {
        let colors = default_colors();
        let color = vte_color_to_glyphon(VteColor::Indexed(15), &colors, true);
        assert_eq!(color, GlyphonColor::rgba(255, 255, 255, 255));
    }

    #[test]
    fn vte_color_rgb_maps_directly() {
        let colors = default_colors();
        let color = vte_color_to_glyphon(
            VteColor::Spec(VteRgb {
                r: 128,
                g: 64,
                b: 32,
            }),
            &colors,
            true,
        );
        assert_eq!(color, GlyphonColor::rgba(128, 64, 32, 255));
    }

    #[test]
    fn vte_color_indexed_cube_16_is_black() {
        let colors = default_colors();
        // Index 16 = r=0, g=0, b=0 in the 6x6x6 cube
        let color = vte_color_to_glyphon(VteColor::Indexed(16), &colors, true);
        assert_eq!(color, GlyphonColor::rgba(0, 0, 0, 255));
    }

    #[test]
    fn vte_color_indexed_cube_231_is_white() {
        let colors = default_colors();
        // Index 231 = r=5, g=5, b=5 -> (255, 255, 255)
        let color = vte_color_to_glyphon(VteColor::Indexed(231), &colors, true);
        assert_eq!(color, GlyphonColor::rgba(255, 255, 255, 255));
    }

    #[test]
    fn vte_color_grayscale_ramp() {
        let colors = default_colors();
        // Index 232 = first grayscale = 8
        let color = vte_color_to_glyphon(VteColor::Indexed(232), &colors, true);
        assert_eq!(color, GlyphonColor::rgba(8, 8, 8, 255));

        // Index 255 = last grayscale = 238
        let color = vte_color_to_glyphon(VteColor::Indexed(255), &colors, true);
        assert_eq!(color, GlyphonColor::rgba(238, 238, 238, 255));
    }
}
