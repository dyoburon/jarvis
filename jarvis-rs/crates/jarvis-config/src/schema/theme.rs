//! Theme and color configuration types.

use serde::{Deserialize, Serialize};

/// Theme selection configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ThemeConfig {
    /// Built-in theme name or path to custom theme YAML.
    pub name: String,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            name: "jarvis-dark".into(),
        }
    }
}

/// Color palette configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorConfig {
    pub primary: String,
    pub secondary: String,
    pub background: String,
    pub panel_bg: String,
    pub text: String,
    pub text_muted: String,
    pub border: String,
    pub border_focused: String,
    pub user_text: String,
    pub tool_read: String,
    pub tool_edit: String,
    pub tool_write: String,
    pub tool_run: String,
    pub tool_search: String,
    pub success: String,
    pub warning: String,
    pub error: String,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            // Catppuccin Mocha palette (https://github.com/catppuccin/catppuccin)
            primary: "#cba6f7".into(),                       // Mauve
            secondary: "#f5c2e7".into(),                     // Pink
            background: "#1e1e2e".into(),                    // Base
            panel_bg: "rgba(30,30,46,0.88)".into(),          // Base with alpha
            text: "#cdd6f4".into(),                          // Text
            text_muted: "#6c7086".into(),                    // Overlay0
            border: "#181825".into(),                        // Mantle
            border_focused: "rgba(203,166,247,0.15)".into(), // Mauve glow
            user_text: "rgba(137,180,250,0.85)".into(),      // Blue
            tool_read: "rgba(137,180,250,0.9)".into(),       // Blue
            tool_edit: "rgba(249,226,175,0.9)".into(),       // Yellow
            tool_write: "rgba(250,179,135,0.9)".into(),      // Peach
            tool_run: "rgba(166,227,161,0.9)".into(),        // Green
            tool_search: "rgba(203,166,247,0.9)".into(),     // Mauve
            success: "#a6e3a1".into(),                       // Green
            warning: "#f9e2af".into(),                       // Yellow
            error: "#f38ba8".into(),                         // Red
        }
    }
}
