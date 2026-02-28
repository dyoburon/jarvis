//! Auto-open panel configuration for startup.
//!
//! Defines which panels launch automatically when Jarvis starts.

use serde::{Deserialize, Serialize};

// =============================================================================
// TYPES
// =============================================================================

/// Panels to open automatically on startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AutoOpenConfig {
    pub panels: Vec<AutoOpenPanel>,
}

/// A single panel to open on startup.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AutoOpenPanel {
    pub kind: PanelKind,
    pub command: Option<String>,
    pub args: Vec<String>,
    pub title: Option<String>,
    pub working_directory: Option<String>,
}

/// The type of panel to open.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PanelKind {
    #[default]
    Terminal,
    Assistant,
    Chat,
    Settings,
    Presence,
}

// =============================================================================
// DEFAULTS
// =============================================================================

impl Default for AutoOpenConfig {
    fn default() -> Self {
        Self {
            panels: vec![AutoOpenPanel {
                kind: PanelKind::Terminal,
                title: Some("Terminal".into()),
                ..Default::default()
            }],
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_auto_open_has_one_terminal() {
        let config = AutoOpenConfig::default();
        assert_eq!(config.panels.len(), 1);
        assert_eq!(config.panels[0].kind, PanelKind::Terminal);
        assert_eq!(config.panels[0].title.as_deref(), Some("Terminal"));
        assert!(config.panels[0].command.is_none());
    }

    #[test]
    fn auto_open_panel_kind_default_is_terminal() {
        assert_eq!(PanelKind::default(), PanelKind::Terminal);
    }

    #[test]
    fn auto_open_round_trips_through_toml() {
        let config = AutoOpenConfig {
            panels: vec![
                AutoOpenPanel {
                    kind: PanelKind::Terminal,
                    command: Some("claude".into()),
                    title: Some("Claude Code".into()),
                    ..Default::default()
                },
                AutoOpenPanel {
                    kind: PanelKind::Terminal,
                    title: Some("Terminal".into()),
                    ..Default::default()
                },
            ],
        };
        let toml_str = toml::to_string(&config).unwrap();
        let parsed: AutoOpenConfig = toml::from_str(&toml_str).unwrap();
        assert_eq!(parsed.panels.len(), 2);
        assert_eq!(parsed.panels[0].command.as_deref(), Some("claude"));
        assert_eq!(parsed.panels[1].title.as_deref(), Some("Terminal"));
    }

    #[test]
    fn auto_open_empty_panels_works() {
        let toml_str = "panels = []";
        let parsed: AutoOpenConfig = toml::from_str(toml_str).unwrap();
        assert!(parsed.panels.is_empty());
    }

    #[test]
    fn auto_open_all_panel_kinds_serialize() {
        let kinds = vec![
            PanelKind::Terminal,
            PanelKind::Assistant,
            PanelKind::Chat,
            PanelKind::Settings,
            PanelKind::Presence,
        ];
        for kind in kinds {
            let panel = AutoOpenPanel {
                kind: kind.clone(),
                ..Default::default()
            };
            let toml_str = toml::to_string(&panel).unwrap();
            let parsed: AutoOpenPanel = toml::from_str(&toml_str).unwrap();
            assert_eq!(parsed.kind, kind);
        }
    }
}
