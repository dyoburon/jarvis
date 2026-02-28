//! Window configuration types.

use serde::{Deserialize, Serialize};

/// Window decoration style.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum WindowDecorations {
    #[default]
    Full,
    None,
    Transparent,
}

/// Window startup mode.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum StartupMode {
    #[default]
    Windowed,
    Maximized,
    Fullscreen,
}

/// Window edge padding in pixels.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct WindowPadding {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

/// Window appearance and behavior settings.
///
/// Controls decorations, opacity, blur, startup mode, title bar,
/// and content padding.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct WindowConfig {
    pub decorations: WindowDecorations,
    /// Window-level opacity (valid range: 0.0-1.0).
    pub opacity: f64,
    /// Enable macOS vibrancy / background blur.
    pub blur: bool,
    pub startup_mode: StartupMode,
    /// Static window title.
    pub title: String,
    /// Update title bar with shell-reported title.
    pub dynamic_title: bool,
    pub padding: WindowPadding,
    /// Height of the custom titlebar area in pixels (macOS).
    /// Traffic lights render in this space. Set 0 to disable.
    pub titlebar_height: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            decorations: WindowDecorations::Full,
            opacity: 1.0,
            blur: false,
            startup_mode: StartupMode::Windowed,
            title: "Jarvis".into(),
            dynamic_title: true,
            padding: WindowPadding::default(),
            titlebar_height: 38,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_config_defaults() {
        let config = WindowConfig::default();
        assert_eq!(config.decorations, WindowDecorations::Full);
        assert!((config.opacity - 1.0).abs() < f64::EPSILON);
        assert!(!config.blur);
        assert_eq!(config.startup_mode, StartupMode::Windowed);
        assert_eq!(config.title, "Jarvis");
        assert!(config.dynamic_title);
        assert_eq!(config.titlebar_height, 38);
    }

    #[test]
    fn window_padding_defaults() {
        let padding = WindowPadding::default();
        assert_eq!(padding.top, 0);
        assert_eq!(padding.right, 0);
        assert_eq!(padding.bottom, 0);
        assert_eq!(padding.left, 0);
    }

    #[test]
    fn window_decorations_serialization() {
        let json = serde_json::to_string(&WindowDecorations::Transparent).unwrap();
        assert_eq!(json, "\"transparent\"");
        let deserialized: WindowDecorations = serde_json::from_str("\"none\"").unwrap();
        assert_eq!(deserialized, WindowDecorations::None);
    }

    #[test]
    fn startup_mode_serialization() {
        let json = serde_json::to_string(&StartupMode::Fullscreen).unwrap();
        assert_eq!(json, "\"fullscreen\"");
        let deserialized: StartupMode = serde_json::from_str("\"maximized\"").unwrap();
        assert_eq!(deserialized, StartupMode::Maximized);
    }

    #[test]
    fn window_config_partial_toml() {
        let toml_str = r#"
decorations = "transparent"
opacity = 0.85
blur = true
title = "My Terminal"

[padding]
top = 8
bottom = 8
"#;
        let config: WindowConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.decorations, WindowDecorations::Transparent);
        assert!((config.opacity - 0.85).abs() < f64::EPSILON);
        assert!(config.blur);
        assert_eq!(config.title, "My Terminal");
        // Defaults preserved
        assert_eq!(config.startup_mode, StartupMode::Windowed);
        assert!(config.dynamic_title);
        assert_eq!(config.padding.top, 8);
        assert_eq!(config.padding.bottom, 8);
        assert_eq!(config.padding.left, 0); // default
    }
}
