//! TOML config file loading and creation.

use crate::schema::JarvisConfig;
use crate::validation;
use jarvis_common::ConfigError;
use std::path::Path;
use tracing::{info, warn};

/// Load config from a specific TOML file path.
///
/// Deserializes the file using serde defaults for any missing fields.
/// After loading, the config is validated; if validation fails, a warning
/// is logged and the default config is returned.
pub fn load_from_path(path: &Path) -> Result<JarvisConfig, ConfigError> {
    if !path.exists() {
        return Err(ConfigError::FileNotFound(path.to_path_buf()));
    }

    let content = std::fs::read_to_string(path).map_err(|e| {
        ConfigError::ParseError(format!("failed to read {}: {e}", path.display()))
    })?;

    let config: JarvisConfig = toml::from_str(&content).map_err(|e| {
        ConfigError::ParseError(format!("failed to parse TOML: {e}"))
    })?;

    // Validate and warn on errors, but still return the parsed config
    if let Err(e) = validation::validate(&config) {
        warn!("config validation warning: {e}");
        warn!("falling back to default config");
        return Ok(JarvisConfig::default());
    }

    info!("loaded config from {}", path.display());
    Ok(config)
}

/// Load config from the platform-specific default path.
///
/// On macOS: `~/Library/Application Support/jarvis/config.toml`
/// On Linux: `~/.config/jarvis/config.toml`
///
/// If the file does not exist, creates a default config file and returns defaults.
pub fn load_default() -> Result<JarvisConfig, ConfigError> {
    let path = default_config_path()?;

    if !path.exists() {
        info!("no config found at {}, creating default", path.display());
        create_default_config(&path)?;
        return Ok(JarvisConfig::default());
    }

    load_from_path(&path)
}

/// Get the platform-specific default config file path.
pub fn default_config_path() -> Result<std::path::PathBuf, ConfigError> {
    let config_dir = dirs::config_dir().ok_or_else(|| {
        ConfigError::ParseError("could not determine config directory".into())
    })?;
    Ok(config_dir.join("jarvis").join("config.toml"))
}

/// Create a default TOML config file with documentation comments.
pub fn create_default_config(path: &Path) -> Result<(), ConfigError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            ConfigError::ParseError(format!(
                "failed to create config directory {}: {e}",
                parent.display()
            ))
        })?;
    }

    let content = default_config_toml();

    std::fs::write(path, content).map_err(|e| {
        ConfigError::ParseError(format!(
            "failed to write default config to {}: {e}",
            path.display()
        ))
    })?;

    info!("created default config at {}", path.display());
    Ok(())
}

/// Generate the default TOML config content with comments.
fn default_config_toml() -> String {
    r##"# Jarvis Configuration
# Schema version 1
# Only override what you want to change -- missing fields use defaults.

[theme]
name = "jarvis-dark"

[colors]
# primary = "#00d4ff"
# secondary = "#ff6b00"
# background = "#000000"
# panel_bg = "rgba(0,0,0,0.93)"
# text = "#f0ece4"
# text_muted = "#888888"
# border = "rgba(0,212,255,0.12)"
# border_focused = "rgba(0,212,255,0.5)"
# success = "#00ff88"
# warning = "#ff6b00"
# error = "#ff4444"

[font]
# family = "Menlo"
# size = 13              # 8-32
# title_size = 15        # 8-48
# line_height = 1.6      # 1.0-3.0

[layout]
# panel_gap = 2          # 0-20
# border_radius = 4      # 0-20
# padding = 14           # 0-40
# max_panels = 5         # 1-10
# default_panel_width = 0.72  # 0.3-1.0
# scrollbar_width = 3    # 1-10

[opacity]
# background = 1.0       # 0.0-1.0
# panel = 0.93
# orb = 1.0
# hex_grid = 0.8
# hud = 1.0

[background]
# mode = "hex_grid"      # hex_grid, solid, image, video, gradient, none

[background.hex_grid]
# color = "#00d4ff"
# opacity = 0.08
# animation_speed = 1.0
# glow_intensity = 0.5

[visualizer]
# enabled = true
# type = "orb"           # orb, image, video, particle, waveform, none
# position_x = 0.0       # -1.0 to 1.0
# position_y = 0.0
# scale = 1.0            # 0.1 to 3.0
# anchor = "center"      # center, top-left, top-right, bottom-left, bottom-right

[startup.boot_animation]
# enabled = true
# duration = 27.0
# skip_on_key = true

[startup.fast_start]
# enabled = false
# delay = 0.5

[startup.on_ready]
# action = "listening"   # listening, panels, chat, game, skill

[voice]
# enabled = true
# mode = "ptt"           # ptt, vad
# input_device = "default"
# sample_rate = 24000

[keybinds]
# push_to_talk = "Option+Period"
# open_assistant = "Cmd+G"
# new_panel = "Cmd+T"
# close_panel = "Escape+Escape"
# toggle_fullscreen = "Cmd+F"
# open_settings = "Cmd+,"
# focus_panel_1 = "Cmd+1"
# focus_panel_2 = "Cmd+2"
# focus_panel_3 = "Cmd+3"
# focus_panel_4 = "Cmd+4"
# focus_panel_5 = "Cmd+5"
# cycle_panels = "Tab"
# cycle_panels_reverse = "Shift+Tab"

[panels.history]
# enabled = true
# max_messages = 1000

[panels.input]
# multiline = true
# auto_grow = true
# max_height = 300

[panels.focus]
# restore_on_activate = true
# show_indicator = true
# border_glow = true

[games.enabled]
# wordle = true
# connections = true
# asteroids = true
# tetris = true
# pinball = true
# doodlejump = true
# minesweeper = true
# draw = true
# subway = true
# videoplayer = true

[livechat]
# enabled = true
# server_port = 19847
# connection_timeout = 10

[presence]
# enabled = true
# server_url = ""
# heartbeat_interval = 30

[performance]
# preset = "high"        # low, medium, high, ultra
# frame_rate = 60        # 30-120
# orb_quality = "high"   # low, medium, high
# bloom_passes = 2       # 1-4

[updates]
# check_automatically = true
# channel = "stable"     # stable, beta
# check_interval = 86400 # seconds (3600-604800)

[logging]
# level = "INFO"         # DEBUG, INFO, WARNING, ERROR
# file_logging = true
# max_file_size_mb = 5
# backup_count = 3
# redact_secrets = true

[advanced.experimental]
# web_rendering = false
# metal_debug = false

[advanced.developer]
# show_fps = false
# show_debug_hud = false
# inspector_enabled = false
"##
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_from_nonexistent_returns_file_not_found() {
        let result = load_from_path(Path::new("/tmp/nonexistent_jarvis_config.toml"));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::FileNotFound(_)));
    }

    #[test]
    fn load_valid_partial_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r##"
[font]
family = "SF Mono"
size = 14

[colors]
primary = "#ff0000"
"##,
        )
        .unwrap();

        let config = load_from_path(&path).unwrap();
        assert_eq!(config.font.family, "SF Mono");
        assert_eq!(config.font.size, 14);
        assert_eq!(config.colors.primary, "#ff0000");
        // Defaults preserved
        assert_eq!(config.colors.background, "#000000");
        assert_eq!(config.theme.name, "jarvis-dark");
    }

    #[test]
    fn load_invalid_toml_returns_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "this is not valid toml {{{").unwrap();

        let result = load_from_path(&path);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ConfigError::ParseError(_)));
    }

    #[test]
    fn load_config_with_invalid_values_falls_back_to_default() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(
            &path,
            r#"
[font]
size = 100
"#,
        )
        .unwrap();

        let config = load_from_path(&path).unwrap();
        // Should fall back to default since validation fails
        assert_eq!(config.font.size, 13);
    }

    #[test]
    fn create_and_load_default_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("jarvis").join("config.toml");

        create_default_config(&path).unwrap();
        assert!(path.exists());

        let config = load_from_path(&path).unwrap();
        assert_eq!(config.theme.name, "jarvis-dark");
        assert_eq!(config.colors.primary, "#00d4ff");
    }

    #[test]
    fn default_config_toml_is_valid() {
        let content = default_config_toml();
        let config: JarvisConfig = toml::from_str(&content).unwrap();
        assert_eq!(config.theme.name, "jarvis-dark");
    }

    #[test]
    fn default_config_path_is_reasonable() {
        // This may not work in all CI environments, but should work locally
        if let Ok(path) = default_config_path() {
            let path_str = path.to_string_lossy();
            assert!(path_str.contains("jarvis"));
            assert!(path_str.ends_with("config.toml"));
        }
    }
}
