//! Background sub-config validation (hex_grid, image, gradient).

use crate::schema::JarvisConfig;

use super::helpers::{validate_range, validate_range_f64};

/// Validate all background-related constraints.
pub(crate) fn validate_background(errors: &mut Vec<String>, config: &JarvisConfig) {
    validate_range_f64(
        errors,
        "background.hex_grid.opacity",
        config.background.hex_grid.opacity,
        0.0,
        1.0,
    );
    validate_range_f64(
        errors,
        "background.hex_grid.animation_speed",
        config.background.hex_grid.animation_speed,
        0.0,
        5.0,
    );
    validate_range_f64(
        errors,
        "background.hex_grid.glow_intensity",
        config.background.hex_grid.glow_intensity,
        0.0,
        1.0,
    );
    validate_range(
        errors,
        "background.image.blur",
        config.background.image.blur,
        0,
        50,
    );
    validate_range_f64(
        errors,
        "background.image.opacity",
        config.background.image.opacity,
        0.0,
        1.0,
    );
    validate_range(
        errors,
        "background.gradient.angle",
        config.background.gradient.angle,
        0,
        360,
    );
}
