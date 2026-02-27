//! Theme type definitions and constants.
//!
//! Contains the override structs used to partially replace config values
//! when a theme is applied, plus the list of built-in theme names.

use crate::schema::ColorConfig;
use serde::{Deserialize, Serialize};

/// Built-in theme names.
pub const BUILT_IN_THEMES: &[&str] = &[
    "jarvis-dark",
    "jarvis-light",
    "catppuccin-mocha",
    "dracula",
    "gruvbox-dark",
    "nord",
    "solarized-dark",
    "tokyo-night",
];

/// Theme override structure.
///
/// All fields are optional; only present fields override the base config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeOverrides {
    pub name: Option<String>,
    pub colors: Option<ColorConfig>,
    pub font: Option<ThemeFontOverrides>,
    pub visualizer: Option<ThemeVisualizerOverrides>,
    pub background: Option<ThemeBackgroundOverrides>,
}

/// Optional font overrides in a theme.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeFontOverrides {
    pub family: Option<String>,
    pub size: Option<u32>,
    pub title_size: Option<u32>,
    pub line_height: Option<f64>,
}

/// Optional visualizer overrides in a theme.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeVisualizerOverrides {
    pub orb_color: Option<String>,
    pub orb_secondary_color: Option<String>,
}

/// Optional background overrides in a theme.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeBackgroundOverrides {
    pub hex_grid_color: Option<String>,
    pub solid_color: Option<String>,
}
