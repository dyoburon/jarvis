//! Tests for the full validation pipeline.

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
