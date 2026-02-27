//! Color parsing and validation utilities.
//!
//! Supports `#RRGGBB`, `#RRGGBBAA`, and `rgba(r,g,b,a)` formats.
//! In the `rgba()` format, the alpha component can be either 0-255 (integer)
//! or 0.0-1.0 (float), matching CSS conventions.

mod parse;

#[cfg(test)]
mod tests;

use jarvis_common::types::Color;
use jarvis_common::ConfigError;

use parse::{parse_hex, parse_rgba, HEX_RE, RGBA_RE};

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
        return Err(ConfigError::ParseError(format!("invalid hex color: {s}")));
    }

    // Try rgba() format
    if s.starts_with("rgba(") || s.starts_with("rgb(") {
        if let Some(color) = parse_rgba(s) {
            return Ok(color);
        }
        return Err(ConfigError::ParseError(format!("invalid rgba color: {s}")));
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
