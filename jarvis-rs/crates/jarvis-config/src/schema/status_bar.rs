//! Status bar configuration types.

use serde::{Deserialize, Serialize};

/// Status bar appearance and behavior settings.
///
/// The status bar is a fixed-height bar at the bottom of the window
/// with panel toggle buttons (left) and app info (right).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StatusBarConfig {
    /// Whether the status bar is visible.
    pub enabled: bool,
    /// Height in pixels (valid range: 20-48).
    pub height: u32,
    /// Show panel toggle buttons on the left side.
    pub show_panel_buttons: bool,
    /// Show online user count on the right side.
    pub show_online_count: bool,
    /// Background color (CSS color string).
    pub bg: String,
}

impl Default for StatusBarConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            height: 28,
            show_panel_buttons: true,
            show_online_count: true,
            bg: "rgba(23,27,36,0.95)".into(),
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
    fn status_bar_config_defaults() {
        let config = StatusBarConfig::default();
        assert!(config.enabled);
        assert_eq!(config.height, 28);
        assert!(config.show_panel_buttons);
        assert!(config.show_online_count);
        assert_eq!(config.bg, "rgba(23,27,36,0.95)");
    }

    #[test]
    fn status_bar_config_partial_toml() {
        let toml_str = r#"
enabled = false
height = 32
"#;
        let config: StatusBarConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.enabled);
        assert_eq!(config.height, 32);
        // Defaults preserved
        assert!(config.show_panel_buttons);
        assert!(config.show_online_count);
        assert_eq!(config.bg, "rgba(23,27,36,0.95)");
    }

    #[test]
    fn status_bar_config_serialization_roundtrip() {
        let config = StatusBarConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: StatusBarConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.enabled, config.enabled);
        assert_eq!(deserialized.height, config.height);
        assert_eq!(deserialized.bg, config.bg);
    }
}
