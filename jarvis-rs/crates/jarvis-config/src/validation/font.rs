//! Font configuration validation (size, title_size, line_height).

use crate::schema::JarvisConfig;

use super::helpers::{validate_range, validate_range_f64};

/// Validate all font-related constraints.
pub(crate) fn validate_font(errors: &mut Vec<String>, config: &JarvisConfig) {
    validate_range(errors, "font.size", config.font.size, 8, 32);
    validate_range(errors, "font.title_size", config.font.title_size, 8, 48);
    validate_range_f64(
        errors,
        "font.line_height",
        config.font.line_height,
        1.0,
        3.0,
    );
}
