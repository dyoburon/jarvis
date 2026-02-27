//! Keybind registry — maps parsed `KeyBind` values to runtime `Action`s.
//!
//! Built from [`KeybindConfig`] at startup and rebuilt on config reload.

use std::collections::HashMap;

use jarvis_common::actions::Action;
use jarvis_config::schema::KeybindConfig;

use crate::keymap::{keybind_to_display, parse_keybind, KeyBind, Modifier};

/// A canonical key representation for fast HashMap lookup.
///
/// Modifiers are stored as a bitmask for O(1) comparison rather than
/// sorting a `Vec<Modifier>` on every event.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct KeyCombo {
    /// Bitmask: Ctrl=1, Alt=2, Shift=4, Super=8.
    pub mods: u8,
    /// Normalized key name (e.g. "G", "Enter", "F1").
    pub key: String,
}

const MOD_CTRL: u8 = 0b0001;
const MOD_ALT: u8 = 0b0010;
const MOD_SHIFT: u8 = 0b0100;
const MOD_SUPER: u8 = 0b1000;

impl KeyCombo {
    /// Build from a parsed [`KeyBind`].
    pub fn from_keybind(kb: &KeyBind) -> Self {
        let mut mods = 0u8;
        for m in &kb.modifiers {
            mods |= match m {
                Modifier::Ctrl => MOD_CTRL,
                Modifier::Alt => MOD_ALT,
                Modifier::Shift => MOD_SHIFT,
                Modifier::Super => MOD_SUPER,
            };
        }
        Self {
            mods,
            key: kb.key.clone(),
        }
    }

    /// Build from raw modifier booleans and a normalized key name.
    ///
    /// Use this to convert winit keyboard events into a `KeyCombo` for lookup.
    pub fn from_winit(ctrl: bool, alt: bool, shift: bool, super_key: bool, key: String) -> Self {
        let mut mods = 0u8;
        if ctrl {
            mods |= MOD_CTRL;
        }
        if alt {
            mods |= MOD_ALT;
        }
        if shift {
            mods |= MOD_SHIFT;
        }
        if super_key {
            mods |= MOD_SUPER;
        }
        Self { mods, key }
    }

    /// Reconstruct a [`KeyBind`] for display purposes.
    fn to_keybind(&self) -> KeyBind {
        let mut modifiers = Vec::new();
        if self.mods & MOD_CTRL != 0 {
            modifiers.push(Modifier::Ctrl);
        }
        if self.mods & MOD_ALT != 0 {
            modifiers.push(Modifier::Alt);
        }
        if self.mods & MOD_SHIFT != 0 {
            modifiers.push(Modifier::Shift);
        }
        if self.mods & MOD_SUPER != 0 {
            modifiers.push(Modifier::Super);
        }
        KeyBind {
            modifiers,
            key: self.key.clone(),
        }
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keycombo_from_keybind() {
        let kb = parse_keybind("Ctrl+G").unwrap();
        let combo = KeyCombo::from_keybind(&kb);
        assert_eq!(combo.mods & MOD_CTRL, MOD_CTRL);
        assert_eq!(combo.key, "G");
    }

    #[test]
    fn keycombo_from_winit() {
        let combo = KeyCombo::from_winit(true, false, true, false, "A".into());
        assert_eq!(combo.mods, MOD_CTRL | MOD_SHIFT);
        assert_eq!(combo.key, "A");
    }

    #[test]
    fn keycombo_equality() {
        let a = KeyCombo::from_winit(true, false, false, false, "G".into());
        let b = KeyCombo::from_keybind(&parse_keybind("Ctrl+G").unwrap());
        assert_eq!(a, b);
    }

    #[test]
    fn keycombo_hash_consistency() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let a = KeyCombo::from_winit(true, true, false, false, "X".into());
        let b = KeyCombo::from_winit(true, true, false, false, "X".into());

        let hash = |c: &KeyCombo| {
            let mut h = DefaultHasher::new();
            c.hash(&mut h);
            h.finish()
        };

        assert_eq!(hash(&a), hash(&b));
    }

    #[test]
    fn registry_from_default_config() {
        let config = KeybindConfig::default();
        let registry = KeybindRegistry::from_config(&config);
        // 16 of 17 keybinds are single-combo (modifier+key).
        // "Escape+Escape" (close_panel) is a double-press pattern that
        // parse_keybind can't handle — it's skipped with a warning.
        // Note: close_pane (Cmd+W) also maps to ClosePane, giving 16 valid.
        assert_eq!(registry.len(), 16);
    }

    #[test]
    fn registry_lookup() {
        let config = KeybindConfig::default();
        let registry = KeybindRegistry::from_config(&config);

        // "Cmd+T" => NewPane (on macOS: Super+T, on Linux: Ctrl+T)
        let kb = parse_keybind("Cmd+T").unwrap();
        let combo = KeyCombo::from_keybind(&kb);
        let action = registry.lookup(&combo);
        assert_eq!(action, Some(&Action::NewPane));
    }

    #[test]
    fn registry_lookup_miss() {
        let config = KeybindConfig::default();
        let registry = KeybindRegistry::from_config(&config);

        let combo = KeyCombo::from_winit(false, false, false, false, "Z".into());
        assert_eq!(registry.lookup(&combo), None);
    }

    #[test]
    fn registry_reverse_lookup() {
        let config = KeybindConfig::default();
        let registry = KeybindRegistry::from_config(&config);

        let display = registry.keybind_for_action(&Action::NewPane);
        assert!(display.is_some());
    }

    #[test]
    fn registry_reverse_lookup_miss() {
        let config = KeybindConfig::default();
        let registry = KeybindRegistry::from_config(&config);

        let display = registry.keybind_for_action(&Action::Quit);
        assert!(display.is_none()); // Quit has no default keybind
    }

    #[test]
    fn keycombo_to_keybind_roundtrip() {
        let original = parse_keybind("Ctrl+Shift+T").unwrap();
        let combo = KeyCombo::from_keybind(&original);
        let back = combo.to_keybind();
        // Modifiers may be in different order, but the combo should match
        assert_eq!(
            KeyCombo::from_keybind(&back),
            KeyCombo::from_keybind(&original)
        );
    }
}
