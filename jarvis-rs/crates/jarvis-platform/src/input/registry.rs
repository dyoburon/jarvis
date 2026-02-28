use std::collections::HashMap;

use jarvis_common::actions::Action;
use jarvis_config::schema::KeybindConfig;

use crate::keymap::{keybind_to_display, parse_keybind};

use super::key_combo::KeyCombo;

/// Maps key combinations to [`Action`]s.
///
/// Built from [`KeybindConfig`] at startup and rebuilt on config reload.
pub struct KeybindRegistry {
    bindings: HashMap<KeyCombo, Action>,
}

impl KeybindRegistry {
    /// Build the registry from the config keybind section.
    ///
    /// Uses [`parse_keybind`] to convert config strings into [`KeyCombo`]s.
    /// Invalid keybind strings are logged as warnings and skipped.
    pub fn from_config(config: &KeybindConfig) -> Self {
        let mut bindings = HashMap::new();

        let mappings: Vec<(&str, Action)> = vec![
            (&config.push_to_talk, Action::PushToTalk),
            (&config.open_assistant, Action::OpenAssistant),
            (&config.new_panel, Action::NewPane),
            (&config.close_panel, Action::ClosePane),
            (&config.toggle_fullscreen, Action::ToggleFullscreen),
            (&config.open_settings, Action::OpenSettings),
            (&config.focus_panel_1, Action::FocusPane(1)),
            (&config.focus_panel_2, Action::FocusPane(2)),
            (&config.focus_panel_3, Action::FocusPane(3)),
            (&config.focus_panel_4, Action::FocusPane(4)),
            (&config.focus_panel_5, Action::FocusPane(5)),
            (&config.cycle_panels, Action::FocusNextPane),
            (&config.cycle_panels_reverse, Action::FocusPrevPane),
            (&config.split_vertical, Action::SplitVertical),
            (&config.split_horizontal, Action::SplitHorizontal),
            (&config.close_pane, Action::ClosePane),
            (&config.command_palette, Action::OpenCommandPalette),
            (&config.copy, Action::Copy),
            (&config.paste, Action::Paste),
        ];

        for (binding_str, action) in mappings {
            match parse_keybind(binding_str) {
                Ok(kb) => {
                    bindings.insert(KeyCombo::from_keybind(&kb), action);
                }
                Err(e) => {
                    tracing::warn!("invalid keybind '{binding_str}': {e}");
                }
            }
        }

        Self { bindings }
    }

    /// Look up an action for a key combination.
    pub fn lookup(&self, combo: &KeyCombo) -> Option<&Action> {
        self.bindings.get(combo)
    }

    /// Get all bindings (for command palette display).
    pub fn all_bindings(&self) -> &HashMap<KeyCombo, Action> {
        &self.bindings
    }

    /// Find the display string for a given action's keybind (reverse lookup).
    ///
    /// Returns the first matching keybind found. If no binding exists for the
    /// action, returns `None`.
    pub fn keybind_for_action(&self, action: &Action) -> Option<String> {
        for (combo, a) in &self.bindings {
            if a == action {
                return Some(keybind_to_display(&combo.to_keybind()));
            }
        }
        None
    }

    /// Number of registered bindings.
    pub fn len(&self) -> usize {
        self.bindings.len()
    }

    /// Whether the registry has no bindings.
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty()
    }
}
