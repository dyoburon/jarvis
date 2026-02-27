//! Full configuration validation.
//!
//! Validates all numeric ranges, keybind uniqueness, and color formats.

use crate::keybinds;
use crate::schema::JarvisConfig;
use jarvis_common::ConfigError;

/// Run all validations on a config, collecting all errors.
pub fn validate(config: &JarvisConfig) -> Result<(), ConfigError> {
    let mut errors: Vec<String> = Vec::new();

    // Keybind duplicates
    if let Err(e) = keybinds::validate_no_duplicates(&config.keybinds) {
        errors.push(e.to_string());
    }

    // Font constraints
    validate_range(&mut errors, "font.size", config.font.size, 8, 32);
    validate_range(&mut errors, "font.title_size", config.font.title_size, 8, 48);
    validate_range_f64(&mut errors, "font.line_height", config.font.line_height, 1.0, 3.0);

    // Layout constraints
    validate_range(&mut errors, "layout.panel_gap", config.layout.panel_gap, 0, 20);
    validate_range(&mut errors, "layout.border_radius", config.layout.border_radius, 0, 20);
    validate_range(&mut errors, "layout.padding", config.layout.padding, 0, 40);
    validate_range(&mut errors, "layout.max_panels", config.layout.max_panels, 1, 10);
    validate_range_f64(
        &mut errors,
        "layout.default_panel_width",
        config.layout.default_panel_width,
        0.3,
        1.0,
    );
    validate_range(&mut errors, "layout.scrollbar_width", config.layout.scrollbar_width, 1, 10);

    // Opacity constraints (all 0.0-1.0)
    validate_range_f64(&mut errors, "opacity.background", config.opacity.background, 0.0, 1.0);
    validate_range_f64(&mut errors, "opacity.panel", config.opacity.panel, 0.0, 1.0);
    validate_range_f64(&mut errors, "opacity.orb", config.opacity.orb, 0.0, 1.0);
    validate_range_f64(&mut errors, "opacity.hex_grid", config.opacity.hex_grid, 0.0, 1.0);
    validate_range_f64(&mut errors, "opacity.hud", config.opacity.hud, 0.0, 1.0);

    // Background sub-config constraints
    validate_range_f64(
        &mut errors,
        "background.hex_grid.opacity",
        config.background.hex_grid.opacity,
        0.0,
        1.0,
    );
    validate_range_f64(
        &mut errors,
        "background.hex_grid.animation_speed",
        config.background.hex_grid.animation_speed,
        0.0,
        5.0,
    );
    validate_range_f64(
        &mut errors,
        "background.hex_grid.glow_intensity",
        config.background.hex_grid.glow_intensity,
        0.0,
        1.0,
    );
    validate_range(
        &mut errors,
        "background.image.blur",
        config.background.image.blur,
        0,
        50,
    );
    validate_range_f64(
        &mut errors,
        "background.image.opacity",
        config.background.image.opacity,
        0.0,
        1.0,
    );
    validate_range(
        &mut errors,
        "background.gradient.angle",
        config.background.gradient.angle,
        0,
        360,
    );

    // Visualizer constraints
    validate_range_f64(
        &mut errors,
        "visualizer.position_x",
        config.visualizer.position_x,
        -1.0,
        1.0,
    );
    validate_range_f64(
        &mut errors,
        "visualizer.position_y",
        config.visualizer.position_y,
        -1.0,
        1.0,
    );
    validate_range_f64(&mut errors, "visualizer.scale", config.visualizer.scale, 0.1, 3.0);

    // Orb visualizer
    validate_range_f64(
        &mut errors,
        "visualizer.orb.intensity_base",
        config.visualizer.orb.intensity_base,
        0.0,
        3.0,
    );
    validate_range_f64(
        &mut errors,
        "visualizer.orb.bloom_intensity",
        config.visualizer.orb.bloom_intensity,
        0.0,
        3.0,
    );
    validate_range_f64(
        &mut errors,
        "visualizer.orb.rotation_speed",
        config.visualizer.orb.rotation_speed,
        0.0,
        5.0,
    );

    // Image visualizer
    validate_range_f64(
        &mut errors,
        "visualizer.image.opacity",
        config.visualizer.image.opacity,
        0.0,
        1.0,
    );
    validate_range_f64(
        &mut errors,
        "visualizer.image.animation_speed",
        config.visualizer.image.animation_speed,
        0.0,
        5.0,
    );

    // Video visualizer
    validate_range_f64(
        &mut errors,
        "visualizer.video.opacity",
        config.visualizer.video.opacity,
        0.0,
        1.0,
    );

    // Particle visualizer
    validate_range(
        &mut errors,
        "visualizer.particle.count",
        config.visualizer.particle.count,
        10,
        5000,
    );
    validate_range_f64(
        &mut errors,
        "visualizer.particle.size",
        config.visualizer.particle.size,
        0.5,
        10.0,
    );
    validate_range_f64(
        &mut errors,
        "visualizer.particle.speed",
        config.visualizer.particle.speed,
        0.1,
        5.0,
    );
    validate_range_f64(
        &mut errors,
        "visualizer.particle.lifetime",
        config.visualizer.particle.lifetime,
        0.5,
        10.0,
    );

    // Waveform visualizer
    validate_range(
        &mut errors,
        "visualizer.waveform.bar_count",
        config.visualizer.waveform.bar_count,
        8,
        256,
    );
    validate_range_f64(
        &mut errors,
        "visualizer.waveform.bar_width",
        config.visualizer.waveform.bar_width,
        1.0,
        10.0,
    );
    validate_range_f64(
        &mut errors,
        "visualizer.waveform.bar_gap",
        config.visualizer.waveform.bar_gap,
        0.0,
        10.0,
    );
    validate_range(
        &mut errors,
        "visualizer.waveform.height",
        config.visualizer.waveform.height,
        20,
        500,
    );
    validate_range_f64(
        &mut errors,
        "visualizer.waveform.smoothing",
        config.visualizer.waveform.smoothing,
        0.0,
        1.0,
    );

    // Visualizer state constraints
    validate_state_config(&mut errors, "visualizer.state_listening", &config.visualizer.state_listening);
    validate_state_config(&mut errors, "visualizer.state_speaking", &config.visualizer.state_speaking);
    validate_state_config(&mut errors, "visualizer.state_skill", &config.visualizer.state_skill);
    validate_state_config(&mut errors, "visualizer.state_chat", &config.visualizer.state_chat);
    validate_state_config(&mut errors, "visualizer.state_idle", &config.visualizer.state_idle);

    // Startup constraints
    validate_range(
        &mut errors,
        "startup.on_ready.panels.count",
        config.startup.on_ready.panels.count,
        1,
        5,
    );

    // Voice constraints
    validate_range_f64(
        &mut errors,
        "voice.sounds.volume",
        config.voice.sounds.volume,
        0.0,
        1.0,
    );

    // Performance constraints
    validate_range(
        &mut errors,
        "performance.frame_rate",
        config.performance.frame_rate,
        30,
        120,
    );
    validate_range(
        &mut errors,
        "performance.bloom_passes",
        config.performance.bloom_passes,
        1,
        4,
    );

    // Livechat constraints
    validate_range(
        &mut errors,
        "livechat.server_port",
        config.livechat.server_port,
        1024,
        65535,
    );
    validate_range(
        &mut errors,
        "livechat.connection_timeout",
        config.livechat.connection_timeout,
        5,
        60,
    );
    validate_range(
        &mut errors,
        "livechat.nickname.validation.min_length",
        config.livechat.nickname.validation.min_length,
        1,
        10,
    );
    validate_range(
        &mut errors,
        "livechat.nickname.validation.max_length",
        config.livechat.nickname.validation.max_length,
        5,
        50,
    );
    validate_range(
        &mut errors,
        "livechat.automod.rate_limit",
        config.livechat.automod.rate_limit,
        1,
        20,
    );
    validate_range(
        &mut errors,
        "livechat.automod.max_message_length",
        config.livechat.automod.max_message_length,
        100,
        2000,
    );

    // Presence constraints
    validate_range(
        &mut errors,
        "presence.heartbeat_interval",
        config.presence.heartbeat_interval,
        10,
        300,
    );

    // Updates constraints
    validate_range(
        &mut errors,
        "updates.check_interval",
        config.updates.check_interval,
        3600,
        604800,
    );

    // Logging constraints
    validate_range(
        &mut errors,
        "logging.max_file_size_mb",
        config.logging.max_file_size_mb,
        1,
        50,
    );
    validate_range(
        &mut errors,
        "logging.backup_count",
        config.logging.backup_count,
        1,
        10,
    );

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ConfigError::ValidationError(errors.join("; ")))
    }
}

fn validate_state_config(
    errors: &mut Vec<String>,
    prefix: &str,
    state: &crate::schema::VisualizerStateConfig,
) {
    validate_range_f64(errors, &format!("{prefix}.scale"), state.scale, 0.1, 3.0);
    validate_range_f64(errors, &format!("{prefix}.intensity"), state.intensity, 0.0, 3.0);
    if let Some(x) = state.position_x {
        validate_range_f64(errors, &format!("{prefix}.position_x"), x, -1.0, 1.0);
    }
    if let Some(y) = state.position_y {
        validate_range_f64(errors, &format!("{prefix}.position_y"), y, -1.0, 1.0);
    }
}

fn validate_range(errors: &mut Vec<String>, name: &str, value: u32, min: u32, max: u32) {
    if value < min || value > max {
        errors.push(format!(
            "{name} = {value} is out of range [{min}, {max}]"
        ));
    }
}

fn validate_range_f64(errors: &mut Vec<String>, name: &str, value: f64, min: f64, max: f64) {
    if value < min || value > max {
        errors.push(format!(
            "{name} = {value} is out of range [{min}, {max}]"
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::*;

    #[test]
    fn default_config_validates() {
        let config = JarvisConfig::default();
        assert!(validate(&config).is_ok());
    }

    #[test]
    fn catches_font_size_too_small() {
        let mut config = JarvisConfig::default();
        config.font.size = 5;
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("font.size"));
    }

    #[test]
    fn catches_font_size_too_large() {
        let mut config = JarvisConfig::default();
        config.font.size = 50;
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("font.size"));
    }

    #[test]
    fn catches_line_height_out_of_range() {
        let mut config = JarvisConfig::default();
        config.font.line_height = 5.0;
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("font.line_height"));
    }

    #[test]
    fn catches_panel_gap_too_large() {
        let mut config = JarvisConfig::default();
        config.layout.panel_gap = 25;
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("layout.panel_gap"));
    }

    #[test]
    fn catches_max_panels_zero() {
        let mut config = JarvisConfig::default();
        config.layout.max_panels = 0;
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("layout.max_panels"));
    }

    #[test]
    fn catches_panel_width_too_small() {
        let mut config = JarvisConfig::default();
        config.layout.default_panel_width = 0.1;
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("layout.default_panel_width"));
    }

    #[test]
    fn catches_opacity_over_one() {
        let mut config = JarvisConfig::default();
        config.opacity.background = 1.5;
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("opacity.background"));
    }

    #[test]
    fn catches_opacity_negative() {
        let mut config = JarvisConfig::default();
        config.opacity.panel = -0.1;
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("opacity.panel"));
    }

    #[test]
    fn catches_frame_rate_too_low() {
        let mut config = JarvisConfig::default();
        config.performance.frame_rate = 15;
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("performance.frame_rate"));
    }

    #[test]
    fn catches_frame_rate_too_high() {
        let mut config = JarvisConfig::default();
        config.performance.frame_rate = 200;
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("performance.frame_rate"));
    }

    #[test]
    fn catches_bloom_passes_out_of_range() {
        let mut config = JarvisConfig::default();
        config.performance.bloom_passes = 0;
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("performance.bloom_passes"));
    }

    #[test]
    fn catches_server_port_too_low() {
        let mut config = JarvisConfig::default();
        config.livechat.server_port = 80;
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("livechat.server_port"));
    }

    #[test]
    fn catches_particle_count_out_of_range() {
        let mut config = JarvisConfig::default();
        config.visualizer.particle.count = 5;
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("visualizer.particle.count"));
    }

    #[test]
    fn catches_check_interval_too_small() {
        let mut config = JarvisConfig::default();
        config.updates.check_interval = 100;
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("updates.check_interval"));
    }

    #[test]
    fn catches_keybind_duplicates() {
        let mut config = JarvisConfig::default();
        config.keybinds.push_to_talk = "Cmd+G".into();
        config.keybinds.open_assistant = "Cmd+G".into();
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("duplicate keybind"));
    }

    #[test]
    fn collects_multiple_errors() {
        let mut config = JarvisConfig::default();
        config.font.size = 100;
        config.opacity.background = 2.0;
        config.performance.frame_rate = 5;
        let err = validate(&config).unwrap_err().to_string();
        assert!(err.contains("font.size"));
        assert!(err.contains("opacity.background"));
        assert!(err.contains("performance.frame_rate"));
    }
}
