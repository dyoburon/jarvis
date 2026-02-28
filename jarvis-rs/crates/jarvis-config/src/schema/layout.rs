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
            panel: 0.93,
            orb: 1.0,
            hex_grid: 0.8,
            hud: 1.0,
        }
    }
}
