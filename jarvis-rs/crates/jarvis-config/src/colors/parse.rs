//! Internal color parsing helpers.
//!
//! Handles the low-level conversion of hex and rgba string formats
//! into [`Color`] values. Not part of the public API.

use jarvis_common::types::Color;
use regex::Regex;
use std::sync::LazyLock;

/// Regex for hex color: #RGB, #RRGGBB, or #RRGGBBAA.
pub(crate) static HEX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$").unwrap());

/// Regex for rgba() color with float or int alpha.
pub(crate) static RGBA_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^rgba?\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*([0-9]*\.?[0-9]+)\s*\)$",
    )
    .unwrap()
});

/// Parse a hex color string (#RGB, #RRGGBB, or #RRGGBBAA).
pub(super) fn parse_hex(s: &str) -> Option<Color> {
    let hex = s.strip_prefix('#')?;
    match hex.len() {
        3 => {
            // Expand #RGB to #RRGGBB
            let r = u8::from_str_radix(&hex[0..1], 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2], 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3], 16).ok()?;
            Some(Color::from_rgba(r * 17, g * 17, b * 17, 255))
        }
        6 => Color::from_hex(s),
        8 => Color::from_hex(s),
        _ => None,
    }
}

/// Parse an `rgba(r,g,b,a)` color string.
/// Alpha is interpreted as 0.0-1.0 (CSS convention) and converted to 0-255.
pub(super) fn parse_rgba(s: &str) -> Option<Color> {
    let caps = RGBA_RE.captures(s)?;
    let r: u8 = caps[1].parse().ok()?;
    let g: u8 = caps[2].parse().ok()?;
    let b: u8 = caps[3].parse().ok()?;
    let a_str = &caps[4];

    // Determine if alpha is float (0.0-1.0) or integer (0-255)
    let a: u8 = if a_str.contains('.') {
        let a_float: f64 = a_str.parse().ok()?;
        if !(0.0..=1.0).contains(&a_float) {
            return None;
        }
        (a_float * 255.0).round() as u8
    } else {
        // Integer alpha: if <= 1, treat as 0 or 1 scaled; otherwise 0-255
        let a_int: u32 = a_str.parse().ok()?;
        if a_int > 255 {
            return None;
        }
        a_int as u8
    };

    Some(Color::from_rgba(r, g, b, a))
}
