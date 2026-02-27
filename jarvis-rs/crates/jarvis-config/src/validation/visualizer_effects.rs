//! Visualizer effect validation (particle, waveform, per-state overrides).

use crate::schema::{JarvisConfig, VisualizerStateConfig};

use super::helpers::{validate_range, validate_range_f64};

/// Validate particle, waveform, and state-override constraints.
pub(crate) fn validate_visualizer_effects(errors: &mut Vec<String>, config: &JarvisConfig) {
    // Particle
    validate_range(
        errors,
        "visualizer.particle.count",
        config.visualizer.particle.count,
        10,
        5000,
    );
    validate_range_f64(
        errors,
        "visualizer.particle.size",
        config.visualizer.particle.size,
        0.5,
        10.0,
    );
    validate_range_f64(
        errors,
        "visualizer.particle.speed",
        config.visualizer.particle.speed,
        0.1,
        5.0,
    );
    validate_range_f64(
        errors,
        "visualizer.particle.lifetime",
        config.visualizer.particle.lifetime,
        0.5,
        10.0,
    );

    // Waveform
    validate_range(
        errors,
        "visualizer.waveform.bar_count",
        config.visualizer.waveform.bar_count,
        8,
        256,
    );
    validate_range_f64(
        errors,
        "visualizer.waveform.bar_width",
        config.visualizer.waveform.bar_width,
        1.0,
        10.0,
    );
    validate_range_f64(
        errors,
        "visualizer.waveform.bar_gap",
        config.visualizer.waveform.bar_gap,
        0.0,
        10.0,
    );
    validate_range(
        errors,
        "visualizer.waveform.height",
        config.visualizer.waveform.height,
        20,
        500,
    );
    validate_range_f64(
        errors,
        "visualizer.waveform.smoothing",
        config.visualizer.waveform.smoothing,
        0.0,
        1.0,
    );

    // Per-state overrides
    validate_state_config(
        errors,
        "visualizer.state_listening",
        &config.visualizer.state_listening,
    );
    validate_state_config(
        errors,
        "visualizer.state_speaking",
        &config.visualizer.state_speaking,
    );
    validate_state_config(
        errors,
        "visualizer.state_skill",
        &config.visualizer.state_skill,
    );
    validate_state_config(
        errors,
        "visualizer.state_chat",
        &config.visualizer.state_chat,
    );
    validate_state_config(
        errors,
        "visualizer.state_idle",
        &config.visualizer.state_idle,
    );
}

/// Validate a single visualizer state override block.
fn validate_state_config(errors: &mut Vec<String>, prefix: &str, state: &VisualizerStateConfig) {
    validate_range_f64(errors, &format!("{prefix}.scale"), state.scale, 0.1, 3.0);
    validate_range_f64(
        errors,
        &format!("{prefix}.intensity"),
        state.intensity,
        0.0,
        3.0,
    );
    if let Some(x) = state.position_x {
        validate_range_f64(errors, &format!("{prefix}.position_x"), x, -1.0, 1.0);
    }
    if let Some(y) = state.position_y {
        validate_range_f64(errors, &format!("{prefix}.position_y"), y, -1.0, 1.0);
    }
}
