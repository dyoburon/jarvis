//! CSS value sanitization to prevent CSS injection attacks.
//!
//! Only allows safe CSS value formats:
//! - Hex colors: `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`
//! - `rgba(r, g, b, a)` with numeric arguments
//! - `rgb(r, g, b)` with numeric arguments
//! - Font families: quoted or unquoted alphanumeric names, comma-separated
//! - Numeric values with units: `14px`, `1.6`, `1.2em`
//!
//! Rejects anything containing: `expression(`, `url(`, `javascript:`,
//! `eval(`, `import`, `;`, `}`, `{`, `@`, `<`, `>`

// =============================================================================
// VALIDATION
// =============================================================================

/// Validate a CSS color value.
///
/// Accepts hex (`#rgb`, `#rrggbb`, etc.) and `rgb()`/`rgba()` with numeric args.
/// Rejects everything else, including named colors (to prevent injection).
pub fn validate_css_color(value: &str) -> Result<(), String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err("Empty CSS color value".to_string());
    }

    // Check for injection patterns first
    check_injection_patterns(trimmed)?;

    // Hex color: #rgb, #rgba, #rrggbb, #rrggbbaa
    if trimmed.starts_with('#') {
        return validate_hex_color(trimmed);
    }

    // rgba(r, g, b, a) or rgb(r, g, b)
    if trimmed.starts_with("rgba(") || trimmed.starts_with("rgb(") {
        return validate_rgb_function(trimmed);
    }

    Err(format!(
        "Invalid CSS color: only hex (#rrggbb) and rgb()/rgba() allowed, got '{trimmed}'"
    ))
}

/// Validate a CSS font-family value.
///
/// Accepts quoted or unquoted font names separated by commas.
/// Only allows: letters, digits, spaces, hyphens, quotes, commas.
pub fn validate_css_font_family(value: &str) -> Result<(), String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err("Empty font-family value".to_string());
    }

    check_injection_patterns(trimmed)?;

    // Only allow safe characters for font names
    for ch in trimmed.chars() {
        if !ch.is_alphanumeric()
            && ch != ' '
            && ch != '-'
            && ch != '_'
            && ch != '\''
            && ch != '"'
            && ch != ','
        {
            return Err(format!(
                "Invalid character '{ch}' in font-family: '{trimmed}'"
            ));
        }
    }

    Ok(())
}

/// Validate a CSS numeric value (font-size, line-height, etc.).
///
/// Accepts: integers, floats, with optional unit (px, em, rem, %).
pub fn validate_css_numeric(value: &str) -> Result<(), String> {
    let trimmed = value.trim();

    if trimmed.is_empty() {
        return Err("Empty CSS numeric value".to_string());
    }

    check_injection_patterns(trimmed)?;

    // Strip known units
    let numeric_part = trimmed
        .trim_end_matches("px")
        .trim_end_matches("em")
        .trim_end_matches("rem")
        .trim_end_matches('%');

    // Must parse as a number
    if numeric_part.parse::<f64>().is_err() {
        return Err(format!("Invalid CSS numeric value: '{trimmed}'"));
    }

    Ok(())
}

// =============================================================================
// HELPERS
// =============================================================================

/// Check for common CSS injection patterns.
fn check_injection_patterns(value: &str) -> Result<(), String> {
    let lower = value.to_lowercase();

    let dangerous = [
        "expression(",
        "url(",
        "javascript:",
        "eval(",
        "import",
        "@import",
        "@charset",
        "behavior:",
        "-moz-binding",
    ];

    for pattern in &dangerous {
        if lower.contains(pattern) {
            return Err(format!("CSS injection blocked: contains '{pattern}'"));
        }
    }

    // Block structural characters that could escape CSS context
    for ch in [';', '{', '}', '<', '>'] {
        if value.contains(ch) {
            return Err(format!("CSS injection blocked: contains '{ch}'"));
        }
    }

    Ok(())
}

/// Validate a hex color string.
fn validate_hex_color(value: &str) -> Result<(), String> {
    let hex = &value[1..]; // skip '#'

    // Must be 3, 4, 6, or 8 hex digits
    let valid_len = matches!(hex.len(), 3 | 4 | 6 | 8);
    if !valid_len {
        return Err(format!(
            "Invalid hex color length: expected 3/4/6/8 digits, got {} in '{value}'",
            hex.len()
        ));
    }

    if !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("Invalid hex color: non-hex character in '{value}'"));
    }

    Ok(())
}

/// Validate an `rgb()` or `rgba()` function call.
fn validate_rgb_function(value: &str) -> Result<(), String> {
    // Extract content between parens
    let inner = value
        .strip_prefix("rgba(")
        .or_else(|| value.strip_prefix("rgb("))
        .and_then(|s| s.strip_suffix(')'))
        .ok_or_else(|| format!("Malformed rgb/rgba: '{value}'"))?;

    // Split by comma, validate each part is numeric
    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

    let expected_count = if value.starts_with("rgba(") { 4 } else { 3 };
    if parts.len() != expected_count {
        return Err(format!(
            "Expected {expected_count} arguments in {}, got {}",
            if expected_count == 4 {
                "rgba()"
            } else {
                "rgb()"
            },
            parts.len()
        ));
    }

    for (i, part) in parts.iter().enumerate() {
        if part.parse::<f64>().is_err() {
            return Err(format!(
                "Non-numeric argument at position {i} in '{value}': '{part}'"
            ));
        }
    }

    Ok(())
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Hex colors ---

    #[test]
    fn valid_hex_3_digit() {
        assert!(validate_css_color("#fff").is_ok());
        assert!(validate_css_color("#000").is_ok());
        assert!(validate_css_color("#abc").is_ok());
    }

    #[test]
    fn valid_hex_4_digit() {
        assert!(validate_css_color("#fffa").is_ok());
    }

    #[test]
    fn valid_hex_6_digit() {
        assert!(validate_css_color("#00d4ff").is_ok());
        assert!(validate_css_color("#ff6b00").is_ok());
        assert!(validate_css_color("#000000").is_ok());
        assert!(validate_css_color("#ffffff").is_ok());
    }

    #[test]
    fn valid_hex_8_digit() {
        assert!(validate_css_color("#00d4ff80").is_ok());
    }

    #[test]
    fn invalid_hex_wrong_length() {
        assert!(validate_css_color("#ff").is_err());
        assert!(validate_css_color("#fffff").is_err());
        assert!(validate_css_color("#fffffff").is_err());
    }

    #[test]
    fn invalid_hex_non_hex_chars() {
        assert!(validate_css_color("#gggggg").is_err());
        assert!(validate_css_color("#xyz").is_err());
    }

    // --- rgb/rgba ---

    #[test]
    fn valid_rgba() {
        assert!(validate_css_color("rgba(0, 212, 255, 0.12)").is_ok());
        assert!(validate_css_color("rgba(0,0,0,0.93)").is_ok());
        assert!(validate_css_color("rgba(140,190,220,0.65)").is_ok());
    }

    #[test]
    fn valid_rgb() {
        assert!(validate_css_color("rgb(255, 0, 0)").is_ok());
        assert!(validate_css_color("rgb(0,0,0)").is_ok());
    }

    #[test]
    fn invalid_rgba_wrong_arg_count() {
        assert!(validate_css_color("rgba(0, 0, 0)").is_err());
        assert!(validate_css_color("rgba(0, 0, 0, 0, 0)").is_err());
    }

    #[test]
    fn invalid_rgb_wrong_arg_count() {
        assert!(validate_css_color("rgb(0, 0)").is_err());
    }

    #[test]
    fn invalid_rgba_non_numeric() {
        assert!(validate_css_color("rgba(red, 0, 0, 1)").is_err());
    }

    // --- Injection attempts ---

    #[test]
    fn rejects_css_injection_expression() {
        assert!(validate_css_color("expression(alert(1))").is_err());
    }

    #[test]
    fn rejects_css_injection_url() {
        assert!(validate_css_color("url(https://evil.com)").is_err());
    }

    #[test]
    fn rejects_css_injection_javascript() {
        assert!(validate_css_color("javascript:alert(1)").is_err());
    }

    #[test]
    fn rejects_css_injection_semicolon() {
        assert!(validate_css_color("red; background: url(evil)").is_err());
    }

    #[test]
    fn rejects_css_injection_braces() {
        assert!(validate_css_color("#fff } body { background: red").is_err());
    }

    #[test]
    fn rejects_css_injection_import() {
        assert!(validate_css_color("@import url(evil.css)").is_err());
    }

    #[test]
    fn rejects_named_colors() {
        // Named colors are rejected because they could mask injection
        assert!(validate_css_color("red").is_err());
        assert!(validate_css_color("blue").is_err());
        assert!(validate_css_color("transparent").is_err());
    }

    #[test]
    fn rejects_empty() {
        assert!(validate_css_color("").is_err());
    }

    // --- Font family ---

    #[test]
    fn valid_font_family() {
        assert!(validate_css_font_family("Menlo").is_ok());
        assert!(validate_css_font_family("'Courier New', monospace").is_ok());
        assert!(validate_css_font_family("SF Mono").is_ok());
        assert!(validate_css_font_family("\"JetBrains Mono\"").is_ok());
    }

    #[test]
    fn invalid_font_family_injection() {
        assert!(validate_css_font_family("Menlo; } body { color: red").is_err());
        assert!(validate_css_font_family("url(evil)").is_err());
    }

    #[test]
    fn invalid_font_family_special_chars() {
        assert!(validate_css_font_family("font<script>").is_err());
        assert!(validate_css_font_family("font{evil}").is_err());
    }

    // --- Numeric ---

    #[test]
    fn valid_numeric() {
        assert!(validate_css_numeric("14px").is_ok());
        assert!(validate_css_numeric("1.6").is_ok());
        assert!(validate_css_numeric("13").is_ok());
        assert!(validate_css_numeric("1.2em").is_ok());
        assert!(validate_css_numeric("100%").is_ok());
    }

    #[test]
    fn invalid_numeric() {
        assert!(validate_css_numeric("abc").is_err());
        assert!(validate_css_numeric("").is_err());
        assert!(validate_css_numeric("14px; color: red").is_err());
    }
}
