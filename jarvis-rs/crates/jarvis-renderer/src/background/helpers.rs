/// Parse a "#RRGGBB" hex string into normalized `[f64; 3]` values in 0.0..=1.0.
///
/// Returns `None` if the string is not a valid 6-digit hex color (with or
/// without the leading `#`).
pub fn hex_to_rgb(hex: &str) -> Option<[f64; 3]> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0])
}
