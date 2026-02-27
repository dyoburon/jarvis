//! Color conversion between vte::ansi::Color and RGBA.
//!
//! @module terminal/color

mod palette;

pub use palette::{indexed_to_rgb, ANSI_COLORS};

use alacritty_terminal::term::color::Colors;
use vte::ansi::{Color, NamedColor};

// =============================================================================
// CONSTANTS
// =============================================================================

const DEFAULT_FG: [u8; 4] = [0xFF, 0xFF, 0xFF, 0xFF]; // white
const DEFAULT_BG: [u8; 4] = [0x00, 0x00, 0x00, 0x00]; // transparent

// =============================================================================
// EXPORTS
// =============================================================================

/// Convert a `vte::ansi::Color` to an RGBA byte array.
///
/// Uses the terminal's `Colors` palette for overrides (e.g. user themes).
/// Falls back to the built-in ANSI palette when no override is set.
///
/// `is_foreground` controls the default for `Named(Foreground)` and
/// `Named(Background)`: foreground defaults to white, background to
/// transparent black.
#[inline]
pub fn vte_color_to_rgba(color: &Color, colors: &Colors, is_foreground: bool) -> [u8; 4] {
    match color {
        Color::Named(named) => named_color_to_rgba(*named, colors, is_foreground),
        Color::Spec(rgb) => [rgb.r, rgb.g, rgb.b, 0xFF],
        Color::Indexed(idx) => {
            if let Some(rgb) = colors[*idx as usize] {
                return [rgb.r, rgb.g, rgb.b, 0xFF];
            }
            let [r, g, b] = palette::indexed_to_rgb(*idx);
            [r, g, b, 0xFF]
        }
    }
}

/// Resolve a `NamedColor` to RGBA.
///
/// Named colors 0-15 map to the standard ANSI palette. Special names
/// (Foreground, Background, Cursor, Dim variants, Bright variants) are
/// resolved from the `Colors` palette or fall back to sensible defaults.
#[inline]
fn named_color_to_rgba(named: NamedColor, colors: &Colors, is_foreground: bool) -> [u8; 4] {
    let idx = named as usize;

    // Check if the terminal has a custom color override for this index.
    if let Some(rgb) = colors[idx] {
        return [rgb.r, rgb.g, rgb.b, 0xFF];
    }

    // Fall back based on the named color variant.
    match named {
        // Standard ANSI 0-15: use built-in palette.
        NamedColor::Black
        | NamedColor::Red
        | NamedColor::Green
        | NamedColor::Yellow
        | NamedColor::Blue
        | NamedColor::Magenta
        | NamedColor::Cyan
        | NamedColor::White
        | NamedColor::BrightBlack
        | NamedColor::BrightRed
        | NamedColor::BrightGreen
        | NamedColor::BrightYellow
        | NamedColor::BrightBlue
        | NamedColor::BrightMagenta
        | NamedColor::BrightCyan
        | NamedColor::BrightWhite => {
            let [r, g, b] = palette::indexed_to_rgb(idx as u8);
            [r, g, b, 0xFF]
        }

        // Foreground/Background: use defaults.
        NamedColor::Foreground | NamedColor::BrightForeground => {
            if is_foreground {
                DEFAULT_FG
            } else {
                // Foreground used as background (e.g. inverse) → opaque white.
                [0xFF, 0xFF, 0xFF, 0xFF]
            }
        }
        NamedColor::Background => {
            if is_foreground {
                // Background used as foreground (e.g. inverse) → opaque black.
                [0x00, 0x00, 0x00, 0xFF]
            } else {
                DEFAULT_BG
            }
        }

        // Cursor: default to white.
        NamedColor::Cursor => [0xFF, 0xFF, 0xFF, 0xFF],

        // Dim variants: fall back to the normal color at half brightness.
        NamedColor::DimBlack => dim_color(NamedColor::Black),
        NamedColor::DimRed => dim_color(NamedColor::Red),
        NamedColor::DimGreen => dim_color(NamedColor::Green),
        NamedColor::DimYellow => dim_color(NamedColor::Yellow),
        NamedColor::DimBlue => dim_color(NamedColor::Blue),
        NamedColor::DimMagenta => dim_color(NamedColor::Magenta),
        NamedColor::DimCyan => dim_color(NamedColor::Cyan),
        NamedColor::DimWhite => dim_color(NamedColor::White),
        NamedColor::DimForeground => [0xC0, 0xC0, 0xC0, 0xFF],
    }
}

/// Produce a dimmed version of a standard ANSI color (roughly 2/3 brightness).
#[inline]
fn dim_color(base: NamedColor) -> [u8; 4] {
    let [r, g, b] = palette::indexed_to_rgb(base as u8);
    [r * 2 / 3, g * 2 / 3, b * 2 / 3, 0xFF]
}

/// Return the default foreground or background RGBA color.
#[inline]
pub fn default_color(is_foreground: bool) -> [u8; 4] {
    if is_foreground {
        DEFAULT_FG
    } else {
        DEFAULT_BG
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use vte::ansi::NamedColor;

    #[test]
    fn named_red_returns_red() {
        let colors = Colors::default();
        let rgba = vte_color_to_rgba(&Color::Named(NamedColor::Red), &colors, true);
        assert_eq!(rgba, [0xCD, 0x00, 0x00, 0xFF]);
    }

    #[test]
    fn spec_rgb_passes_through() {
        let colors = Colors::default();
        let rgb = vte::ansi::Rgb {
            r: 128,
            g: 64,
            b: 32,
        };
        let rgba = vte_color_to_rgba(&Color::Spec(rgb), &colors, true);
        assert_eq!(rgba, [128, 64, 32, 0xFF]);
    }

    #[test]
    fn indexed_42_resolves() {
        let colors = Colors::default();
        let rgba = vte_color_to_rgba(&Color::Indexed(42), &colors, true);
        // 42 = 16 + 26 → r=0, g=4, b=2 → [0, 215, 135]
        let [r, g, b] = palette::indexed_to_rgb(42);
        assert_eq!(rgba, [r, g, b, 0xFF]);
    }

    #[test]
    fn default_fg_is_white() {
        assert_eq!(default_color(true), [0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn default_bg_is_transparent() {
        assert_eq!(default_color(false), [0x00, 0x00, 0x00, 0x00]);
    }
}
