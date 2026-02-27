//! Color parsing and validation utilities.
//!
//! Supports `#RRGGBB`, `#RRGGBBAA`, and `rgba(r,g,b,a)` formats.
//! In the `rgba()` format, the alpha component can be either 0-255 (integer)
//! or 0.0-1.0 (float), matching CSS conventions.

use jarvis_common::types::Color;
use jarvis_common::ConfigError;
use regex::Regex;
use std::sync::LazyLock;

/// Regex for hex color: #RGB, #RRGGBB, or #RRGGBBAA.
static HEX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^#([0-9a-fA-F]{3}|[0-9a-fA-F]{6}|[0-9a-fA-F]{8})$").unwrap());

/// Regex for rgba() color with float or int alpha.
static RGBA_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^rgba?\(\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*(\d{1,3})\s*,\s*([0-9]*\.?[0-9]+)\s*\)$")
        .unwrap()
});

/// Parse a color string into a [`Color`].
///
/// Accepted formats:
/// - `#RRGGBB` (e.g. `#00d4ff`)
/// - `#RRGGBBAA` (e.g. `#00d4ff80`)
/// - `rgba(r,g,b,a)` where `a` is 0.0-1.0 (e.g. `rgba(0,212,255,0.12)`)
pub fn parse_color(s: &str) -> Result<Color, ConfigError> {
    let s = s.trim();

    // Try hex formats first
    if s.starts_with('#') {
        if let Some(color) = parse_hex(s) {
            return Ok(color);
        }
        return Err(ConfigError::ParseError(format!(
            "invalid hex color: {s}"
        )));
    }

    // Try rgba() format
    if s.starts_with("rgba(") || s.starts_with("rgb(") {
        if let Some(color) = parse_rgba(s) {
            return Ok(color);
        }
        return Err(ConfigError::ParseError(format!(
            "invalid rgba color: {s}"
        )));
    }

    Err(ConfigError::ParseError(format!(
        "unrecognized color format: {s}"
    )))
}

/// Validate that a string is a recognized color format.
pub fn validate_color(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }
    if s.starts_with('#') {
        return HEX_RE.is_match(s);
    }
    if s.starts_with("rgba(") || s.starts_with("rgb(") {
        return RGBA_RE.is_match(s);
    }
    false
}

/// Parse a hex color string (#RGB, #RRGGBB, or #RRGGBBAA).
fn parse_hex(s: &str) -> Option<Color> {
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
fn parse_rgba(s: &str) -> Option<Color> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_6_digit() {
        let c = parse_color("#00d4ff").unwrap();
        assert_eq!(c, Color::from_rgba(0, 212, 255, 255));
    }

    #[test]
    fn parse_hex_8_digit() {
        let c = parse_color("#00d4ff80").unwrap();
        assert_eq!(c, Color::from_rgba(0, 212, 255, 128));
    }

    #[test]
    fn parse_hex_3_digit() {
        let c = parse_color("#f00").unwrap();
        assert_eq!(c, Color::from_rgba(255, 0, 0, 255));
    }

    #[test]
    fn parse_rgba_float_alpha() {
        let c = parse_color("rgba(0,212,255,0.12)").unwrap();
        assert_eq!(c.r, 0);
        assert_eq!(c.g, 212);
        assert_eq!(c.b, 255);
        // 0.12 * 255 = 30.6 -> 31
        assert_eq!(c.a, 31);
    }

    #[test]
    fn parse_rgba_half_alpha() {
        let c = parse_color("rgba(0,0,0,0.5)").unwrap();
        assert_eq!(c.a, 128);
    }

    #[test]
    fn parse_rgba_full_alpha() {
        let c = parse_color("rgba(255,255,255,1.0)").unwrap();
        assert_eq!(c, Color::from_rgba(255, 255, 255, 255));
    }

    #[test]
    fn parse_rgba_zero_alpha() {
        let c = parse_color("rgba(0,0,0,0.0)").unwrap();
        assert_eq!(c.a, 0);
    }

    #[test]
    fn parse_rgba_with_spaces() {
        let c = parse_color("rgba( 100 , 180 , 255 , 0.9 )").unwrap();
        assert_eq!(c.r, 100);
        assert_eq!(c.g, 180);
        assert_eq!(c.b, 255);
        // 0.9 * 255 = 229.5 -> 230
        assert_eq!(c.a, 230);
    }

    #[test]
    fn parse_color_invalid_format() {
        assert!(parse_color("not-a-color").is_err());
        assert!(parse_color("").is_err());
        assert!(parse_color("#xyz").is_err());
        assert!(parse_color("rgba(300,0,0,1.0)").is_err());
    }

    #[test]
    fn validate_color_accepts_valid() {
        assert!(validate_color("#00d4ff"));
        assert!(validate_color("#00d4ff80"));
        assert!(validate_color("#f00"));
        assert!(validate_color("rgba(0,212,255,0.12)"));
        assert!(validate_color("rgba(255,255,255,1.0)"));
    }

    #[test]
    fn validate_color_rejects_invalid() {
        assert!(!validate_color(""));
        assert!(!validate_color("not-a-color"));
        assert!(!validate_color("#12345")); // 5 digits
        assert!(!validate_color("rgb(10,20)"));
    }

    #[test]
    fn parse_all_default_colors() {
        // Verify all default colors from the schema are parseable
        let colors = [
            "#00d4ff",
            "#ff6b00",
            "#000000",
            "rgba(0,0,0,0.93)",
            "#f0ece4",
            "#888888",
            "rgba(0,212,255,0.12)",
            "rgba(0,212,255,0.5)",
            "rgba(140,190,220,0.65)",
            "rgba(100,180,255,0.9)",
            "rgba(255,180,80,0.9)",
            "rgba(80,220,120,0.9)",
            "rgba(200,150,255,0.9)",
            "#00ff88",
            "#ff4444",
        ];
        for c in &colors {
            assert!(
                parse_color(c).is_ok(),
                "failed to parse default color: {c}"
            );
        }
    }
}
