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
    pub effects: Option<ThemeEffectsOverrides>,
    pub terminal: Option<ThemeTerminalOverrides>,
    pub window: Option<ThemeWindowOverrides>,
}

/// Optional font overrides in a theme.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeFontOverrides {
    pub family: Option<String>,
    pub size: Option<u32>,
    pub title_size: Option<u32>,
    pub line_height: Option<f64>,
    pub nerd_font: Option<bool>,
    pub ligatures: Option<bool>,
    pub font_weight: Option<u32>,
    pub bold_weight: Option<u32>,
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

/// Optional effects overrides in a theme.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeEffectsOverrides {
    pub scanline_intensity: Option<f32>,
    pub vignette_intensity: Option<f32>,
    pub bloom_intensity: Option<f32>,
    pub glow_color: Option<String>,
    pub glow_width: Option<f32>,
}

/// Optional terminal overrides in a theme.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeTerminalOverrides {
    pub cursor_style: Option<String>,
    pub cursor_blink: Option<bool>,
}

/// Optional window overrides in a theme.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeWindowOverrides {
    pub opacity: Option<f64>,
    pub blur: Option<bool>,
}

/// Theme metadata for discovery and UI display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeInfo {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub author: Option<String>,
    pub preview_colors: ThemePreviewColors,
}

/// Preview colors for theme selection UI.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemePreviewColors {
    pub primary: String,
    pub background: String,
    pub text: String,
}
