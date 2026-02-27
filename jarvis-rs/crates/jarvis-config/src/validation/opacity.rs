//! Opacity configuration validation (all 0.0–1.0 range checks).

use crate::schema::JarvisConfig;

use super::helpers::validate_range_f64;

/// Validate all top-level opacity constraints.
pub(crate) fn validate_opacity(errors: &mut Vec<String>, config: &JarvisConfig) {
    validate_range_f64(
        errors,
        "opacity.background",
        config.opacity.background,
        0.0,
        1.0,
    );
    validate_range_f64(errors, "opacity.panel", config.opacity.panel, 0.0, 1.0);
    validate_range_f64(errors, "opacity.orb", config.opacity.orb, 0.0, 1.0);
    validate_range_f64(
        errors,
        "opacity.hex_grid",
        config.opacity.hex_grid,
        0.0,
        1.0,
    );
    validate_range_f64(errors, "opacity.hud", config.opacity.hud, 0.0, 1.0);
}
