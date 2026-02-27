//! Default ANSI color palette (indices 0-255).
//!
//! @module terminal/color/palette

/// Standard ANSI 16-color palette (indices 0-15) as `[r, g, b]`.
pub const ANSI_COLORS: [[u8; 3]; 16] = [
    [0x00, 0x00, 0x00], // 0  Black
    [0xCD, 0x00, 0x00], // 1  Red
    [0x00, 0xCD, 0x00], // 2  Green
    [0xCD, 0xCD, 0x00], // 3  Yellow
    [0x00, 0x00, 0xEE], // 4  Blue
    [0xCD, 0x00, 0xCD], // 5  Magenta
    [0x00, 0xCD, 0xCD], // 6  Cyan
    [0xE5, 0xE5, 0xE5], // 7  White
    [0x7F, 0x7F, 0x7F], // 8  Bright Black
    [0xFF, 0x00, 0x00], // 9  Bright Red
    [0x00, 0xFF, 0x00], // 10 Bright Green
    [0xFF, 0xFF, 0x00], // 11 Bright Yellow
    [0x5C, 0x5C, 0xFF], // 12 Bright Blue
    [0xFF, 0x00, 0xFF], // 13 Bright Magenta
    [0x00, 0xFF, 0xFF], // 14 Bright Cyan
    [0xFF, 0xFF, 0xFF], // 15 Bright White
];

/// Resolve an indexed color (0-255) to `[r, g, b]`.
///
/// - 0-15: Standard ANSI palette
/// - 16-231: 6×6×6 color cube
/// - 232-255: 24-step grayscale ramp
pub fn indexed_to_rgb(idx: u8) -> [u8; 3] {
    match idx {
        0..=15 => ANSI_COLORS[idx as usize],
        16..=231 => {
            let i = idx - 16;
            let r = (i / 36) % 6;
            let g = (i / 6) % 6;
            let b = i % 6;
            let to_val = |c: u8| if c == 0 { 0 } else { 55 + c * 40 };
            [to_val(r), to_val(g), to_val(b)]
        }
        232..=255 => {
            let v = 8 + (idx - 232) * 10;
            [v, v, v]
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_black_is_zero() {
        assert_eq!(indexed_to_rgb(0), [0x00, 0x00, 0x00]);
    }

    #[test]
    fn ansi_bright_white_is_ff() {
        assert_eq!(indexed_to_rgb(15), [0xFF, 0xFF, 0xFF]);
    }

    #[test]
    fn color_cube_index_16_is_black() {
        assert_eq!(indexed_to_rgb(16), [0, 0, 0]);
    }

    #[test]
    fn color_cube_index_196_is_red() {
        // 196 = 16 + 5*36 + 0*6 + 0 → r=5, g=0, b=0
        assert_eq!(indexed_to_rgb(196), [255, 0, 0]);
    }

    #[test]
    fn grayscale_232_is_dark() {
        assert_eq!(indexed_to_rgb(232), [8, 8, 8]);
    }

    #[test]
    fn grayscale_255_is_light() {
        assert_eq!(indexed_to_rgb(255), [238, 238, 238]);
    }
}
