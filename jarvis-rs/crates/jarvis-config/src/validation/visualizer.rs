//! Visualizer configuration validation (position, scale, orb, image, video).

use crate::schema::JarvisConfig;

use super::helpers::validate_range_f64;
use super::visualizer_effects;

/// Validate all visualizer-related constraints.
pub(crate) fn validate_visualizer(errors: &mut Vec<String>, config: &JarvisConfig) {
    // Top-level visualizer position / scale
    validate_range_f64(
        errors,
        "visualizer.position_x",
        config.visualizer.position_x,
        -1.0,
        1.0,
    );
    validate_range_f64(
        errors,
        "visualizer.position_y",
        config.visualizer.position_y,
        -1.0,
        1.0,
    );
    validate_range_f64(
        errors,
        "visualizer.scale",
        config.visualizer.scale,
        0.1,
        3.0,
    );

    // Orb
    validate_range_f64(
        errors,
        "visualizer.orb.intensity_base",
        config.visualizer.orb.intensity_base,
        0.0,
        3.0,
    );
    validate_range_f64(
        errors,
        "visualizer.orb.bloom_intensity",
        config.visualizer.orb.bloom_intensity,
        0.0,
        3.0,
    );
    validate_range_f64(
        errors,
        "visualizer.orb.rotation_speed",
        config.visualizer.orb.rotation_speed,
        0.0,
        5.0,
    );

    // Image
    validate_range_f64(
        errors,
        "visualizer.image.opacity",
        config.visualizer.image.opacity,
        0.0,
        1.0,
    );
    validate_range_f64(
        errors,
        "visualizer.image.animation_speed",
        config.visualizer.image.animation_speed,
        0.0,
        5.0,
    );

    // Video
    validate_range_f64(
        errors,
        "visualizer.video.opacity",
        config.visualizer.video.opacity,
        0.0,
        1.0,
    );

    // Particle, waveform, and per-state overrides
    visualizer_effects::validate_visualizer_effects(errors, config);
}
