//! Terminal emulation configuration types.

use serde::{Deserialize, Serialize};

/// Cursor visual style.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum CursorStyle {
    #[default]
    Block,
    Underline,
    Beam,
}

/// Bell notification mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BellConfig {
    pub visual: bool,
    pub audio: bool,
    /// Duration of visual bell flash in milliseconds (valid range: 50-1000).
    pub duration_ms: u32,
}

impl Default for BellConfig {
    fn default() -> Self {
        Self {
            visual: true,
            audio: false,
            duration_ms: 150,
        }
    }
}

/// Mouse behavior configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct MouseConfig {
    pub copy_on_select: bool,
    pub url_detection: bool,
    pub click_to_focus: bool,
}

impl Default for MouseConfig {
    fn default() -> Self {
        Self {
            copy_on_select: false,
            url_detection: true,
            click_to_focus: true,
        }
    }
}

/// Search overlay configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SearchConfig {
    pub wrap_around: bool,
    pub regex: bool,
    pub case_sensitive: bool,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            wrap_around: true,
            regex: false,
            case_sensitive: false,
        }
    }
}

/// Terminal emulation settings.
///
/// Controls scrollback depth, cursor appearance, bell behavior,
/// mouse interaction, and search overlay.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct TerminalConfig {
    /// Number of scrollback lines (valid range: 0-100_000).
    pub scrollback_lines: u32,
    pub cursor_style: CursorStyle,
    pub cursor_blink: bool,
    /// Cursor blink interval in milliseconds (valid range: 100-2000).
    pub cursor_blink_interval_ms: u32,
    pub bell: BellConfig,
    /// Characters treated as word boundaries for double-click selection.
    pub word_separators: String,
    /// Enable 24-bit true color support.
    pub true_color: bool,
    pub mouse: MouseConfig,
    pub search: SearchConfig,
}

impl Default for TerminalConfig {
    fn default() -> Self {
        Self {
            scrollback_lines: 10_000,
            cursor_style: CursorStyle::Block,
            cursor_blink: true,
            cursor_blink_interval_ms: 500,
            bell: BellConfig::default(),
            word_separators: r#" /\()\"'-.,:;<>~!@#$%^&*|+=[]{}~?"#.into(),
            true_color: true,
            mouse: MouseConfig::default(),
            search: SearchConfig::default(),
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
    fn terminal_config_defaults() {
        let config = TerminalConfig::default();
        assert_eq!(config.scrollback_lines, 10_000);
        assert_eq!(config.cursor_style, CursorStyle::Block);
        assert!(config.cursor_blink);
        assert_eq!(config.cursor_blink_interval_ms, 500);
        assert!(config.true_color);
    }

    #[test]
    fn bell_config_defaults() {
        let config = BellConfig::default();
        assert!(config.visual);
        assert!(!config.audio);
        assert_eq!(config.duration_ms, 150);
    }

    #[test]
    fn mouse_config_defaults() {
        let config = MouseConfig::default();
        assert!(!config.copy_on_select);
        assert!(config.url_detection);
        assert!(config.click_to_focus);
    }

    #[test]
    fn search_config_defaults() {
        let config = SearchConfig::default();
        assert!(config.wrap_around);
        assert!(!config.regex);
        assert!(!config.case_sensitive);
    }

    #[test]
    fn cursor_style_serialization() {
        let json = serde_json::to_string(&CursorStyle::Beam).unwrap();
        assert_eq!(json, "\"beam\"");
        let deserialized: CursorStyle = serde_json::from_str("\"underline\"").unwrap();
        assert_eq!(deserialized, CursorStyle::Underline);
    }

    #[test]
    fn terminal_config_partial_toml() {
        let toml_str = r#"
scrollback_lines = 50000
cursor_blink = false

[bell]
audio = true
"#;
        let config: TerminalConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.scrollback_lines, 50_000);
        assert!(!config.cursor_blink);
        assert!(config.bell.audio);
        // Defaults preserved
        assert_eq!(config.cursor_style, CursorStyle::Block);
        assert!(config.bell.visual);
        assert!(config.mouse.url_detection);
    }
}
