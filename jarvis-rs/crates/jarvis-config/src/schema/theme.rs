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
            // Ayu Mirage palette (https://github.com/ayu-theme/ayu-colors)
            primary: "#ffcc66".into(),
            secondary: "#ffa659".into(),
            background: "#1f2430".into(),
            panel_bg: "rgba(36,41,54,0.88)".into(),
            text: "#cccac2".into(),
            text_muted: "#707a8c".into(),
            border: "#171B24".into(),
            border_focused: "rgba(255,204,102,0.12)".into(),
            user_text: "rgba(115,208,255,0.75)".into(),
            tool_read: "rgba(115,208,255,0.9)".into(),
            tool_edit: "rgba(255,213,128,0.9)".into(),
            tool_write: "rgba(255,166,89,0.9)".into(),
            tool_run: "rgba(186,230,126,0.9)".into(),
            tool_search: "rgba(223,191,255,0.9)".into(),
            success: "#87d96c".into(),
            warning: "#ffa659".into(),
            error: "#ff6666".into(),
        }
    }
}
