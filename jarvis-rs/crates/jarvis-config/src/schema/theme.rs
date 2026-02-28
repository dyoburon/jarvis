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
            primary: "#00d4ff".into(),
            secondary: "#ff6b00".into(),
            background: "#000000".into(),
            panel_bg: "rgba(10,14,20,0.72)".into(),
            text: "#f0ece4".into(),
            text_muted: "#888888".into(),
            border: "rgba(0,212,255,0.06)".into(),
            border_focused: "rgba(0,212,255,0.20)".into(),
            user_text: "rgba(140,190,220,0.65)".into(),
            tool_read: "rgba(100,180,255,0.9)".into(),
            tool_edit: "rgba(255,180,80,0.9)".into(),
            tool_write: "rgba(255,180,80,0.9)".into(),
            tool_run: "rgba(80,220,120,0.9)".into(),
            tool_search: "rgba(200,150,255,0.9)".into(),
            success: "#00ff88".into(),
            warning: "#ff6b00".into(),
            error: "#ff4444".into(),
        }
    }
}
