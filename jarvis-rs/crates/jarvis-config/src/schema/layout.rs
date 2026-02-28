//! Panel layout and opacity configuration types.

use serde::{Deserialize, Serialize};

/// Panel layout configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct LayoutConfig {
    /// Gap between panels in pixels (valid range: 0-20).
    pub panel_gap: u32,
    /// Border radius in pixels (valid range: 0-20).
    pub border_radius: u32,
    /// Padding in pixels (valid range: 0-40).
    pub padding: u32,
    /// Maximum number of panels (valid range: 1-10).
    pub max_panels: u32,
    /// Default panel width as fraction of screen (valid range: 0.3-1.0).
    pub default_panel_width: f64,
    /// Scrollbar width in pixels (valid range: 1-10).
    pub scrollbar_width: u32,
    /// Panel border width in pixels (valid range: 0.0-3.0).
    pub border_width: f64,
    /// Screen-edge padding in pixels (valid range: 0-40).
    pub outer_padding: u32,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            panel_gap: 8,
            border_radius: 8,
            padding: 10,
            max_panels: 5,
            default_panel_width: 0.72,
            scrollbar_width: 3,
            border_width: 0.5,
            outer_padding: 10,
        }
    }
}

/// Transparency settings (all values in range 0.0-1.0).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct OpacityConfig {
    pub background: f64,
    pub panel: f64,
    pub orb: f64,
    pub hex_grid: f64,
    pub hud: f64,
}

impl Default for OpacityConfig {
    fn default() -> Self {
        Self {
            background: 1.0,
            panel: 0.72,
            orb: 1.0,
            hex_grid: 0.8,
            hud: 1.0,
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
    fn layout_config_defaults() {
        let config = LayoutConfig::default();
        assert_eq!(config.panel_gap, 8);
        assert_eq!(config.border_radius, 8);
        assert_eq!(config.padding, 10);
        assert_eq!(config.max_panels, 5);
        assert!((config.default_panel_width - 0.72).abs() < f64::EPSILON);
        assert_eq!(config.scrollbar_width, 3);
        assert!((config.border_width - 0.5).abs() < f64::EPSILON);
        assert_eq!(config.outer_padding, 10);
    }

    #[test]
    fn opacity_config_defaults() {
        let config = OpacityConfig::default();
        assert!((config.background - 1.0).abs() < f64::EPSILON);
        assert!((config.panel - 0.72).abs() < f64::EPSILON);
        assert!((config.orb - 1.0).abs() < f64::EPSILON);
        assert!((config.hex_grid - 0.8).abs() < f64::EPSILON);
        assert!((config.hud - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn layout_config_partial_toml() {
        let toml_str = r#"
panel_gap = 12
border_width = 1.0
outer_padding = 20
"#;
        let config: LayoutConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.panel_gap, 12);
        assert!((config.border_width - 1.0).abs() < f64::EPSILON);
        assert_eq!(config.outer_padding, 20);
        // Defaults preserved
        assert_eq!(config.border_radius, 8);
        assert_eq!(config.padding, 10);
        assert_eq!(config.scrollbar_width, 3);
    }

    #[test]
    fn opacity_config_partial_toml() {
        let toml_str = r#"
panel = 0.85
hex_grid = 0.5
"#;
        let config: OpacityConfig = toml::from_str(toml_str).unwrap();
        assert!((config.panel - 0.85).abs() < f64::EPSILON);
        assert!((config.hex_grid - 0.5).abs() < f64::EPSILON);
        // Defaults preserved
        assert!((config.background - 1.0).abs() < f64::EPSILON);
    }
}
