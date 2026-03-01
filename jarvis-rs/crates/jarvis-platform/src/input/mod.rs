//! Keybind registry — maps parsed `KeyBind` values to runtime `Action`s.
//!
//! Built from [`KeybindConfig`] at startup and rebuilt on config reload.

mod key_combo;
mod registry;

pub use key_combo::KeyCombo;
pub use registry::KeybindRegistry;

#[cfg(test)]
mod tests {
    use super::key_combo::*;
    use super::*;
    use crate::keymap::parse_keybind;
    use jarvis_common::actions::Action;
    use jarvis_config::schema::KeybindConfig;

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
        assert_eq!(registry.len(), 19);
    }

    #[test]
    fn registry_lookup() {
        let config = KeybindConfig::default();
        let registry = KeybindRegistry::from_config(&config);

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
        assert!(display.is_none());
    }

    #[test]
    fn keycombo_to_keybind_roundtrip() {
        let original = parse_keybind("Ctrl+Shift+T").unwrap();
        let combo = KeyCombo::from_keybind(&original);
        let back = combo.to_keybind();
        assert_eq!(
            KeyCombo::from_keybind(&back),
            KeyCombo::from_keybind(&original)
        );
    }
}
