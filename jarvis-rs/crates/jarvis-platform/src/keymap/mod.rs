mod display;
mod parse;
mod types;

pub use display::keybind_to_display;
pub use parse::parse_keybind;
pub use types::{KeyBind, Modifier};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple_keybind() {
        let kb = parse_keybind("Ctrl+G").unwrap();
        assert_eq!(kb.modifiers, vec![Modifier::Ctrl]);
        assert_eq!(kb.key, "G");
    }

    #[test]
    fn parse_multi_modifier_keybind() {
        let kb = parse_keybind("Ctrl+Shift+T").unwrap();
        assert_eq!(kb.modifiers, vec![Modifier::Ctrl, Modifier::Shift]);
        assert_eq!(kb.key, "T");
    }

    #[test]
    fn parse_option_becomes_alt() {
        let kb = parse_keybind("Option+Period").unwrap();
        assert_eq!(kb.modifiers, vec![Modifier::Alt]);
        assert_eq!(kb.key, ".");
    }

    #[test]
    fn parse_cmd_modifier() {
        let kb = parse_keybind("Cmd+G").unwrap();
        if cfg!(target_os = "macos") {
            assert_eq!(kb.modifiers, vec![Modifier::Super]);
        } else {
            assert_eq!(kb.modifiers, vec![Modifier::Ctrl]);
        }
        assert_eq!(kb.key, "G");
    }

    #[test]
    fn parse_command_modifier() {
        let kb = parse_keybind("Command+Q").unwrap();
        if cfg!(target_os = "macos") {
            assert_eq!(kb.modifiers, vec![Modifier::Super]);
        } else {
            assert_eq!(kb.modifiers, vec![Modifier::Ctrl]);
        }
        assert_eq!(kb.key, "Q");
    }

    #[test]
    fn parse_super_modifier() {
        let kb = parse_keybind("Super+L").unwrap();
        assert_eq!(kb.modifiers, vec![Modifier::Super]);
        assert_eq!(kb.key, "L");
    }

    #[test]
    fn parse_single_key() {
        let kb = parse_keybind("F1").unwrap();
        assert!(kb.modifiers.is_empty());
        assert_eq!(kb.key, "F1");
    }

    #[test]
    fn parse_key_normalization() {
        let kb = parse_keybind("Ctrl+Enter").unwrap();
        assert_eq!(kb.key, "Enter");

        let kb = parse_keybind("Ctrl+Return").unwrap();
        assert_eq!(kb.key, "Enter");

        let kb = parse_keybind("Ctrl+Escape").unwrap();
        assert_eq!(kb.key, "Escape");

        let kb = parse_keybind("Ctrl+Esc").unwrap();
        assert_eq!(kb.key, "Escape");

        let kb = parse_keybind("Ctrl+Space").unwrap();
        assert_eq!(kb.key, "Space");
    }

    #[test]
    fn parse_empty_string_fails() {
        assert!(parse_keybind("").is_err());
    }

    #[test]
    fn parse_duplicate_modifiers_deduplicated() {
        let kb = parse_keybind("Ctrl+Ctrl+A").unwrap();
        assert_eq!(kb.modifiers, vec![Modifier::Ctrl]);
        assert_eq!(kb.key, "A");
    }

    #[test]
    fn display_keybind_platform() {
        let kb = KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: "G".into(),
        };
        let display = keybind_to_display(&kb);

        if cfg!(target_os = "macos") {
            assert_eq!(display, "\u{2303}G"); // ⌃G
        } else {
            assert_eq!(display, "Ctrl+G");
        }
    }

    #[test]
    fn display_keybind_multi_modifier() {
        let kb = KeyBind {
            modifiers: vec![Modifier::Ctrl, Modifier::Shift],
            key: "T".into(),
        };
        let display = keybind_to_display(&kb);

        if cfg!(target_os = "macos") {
            assert_eq!(display, "\u{2303}\u{21E7}T"); // ⌃⇧T
        } else {
            assert_eq!(display, "Ctrl+Shift+T");
        }
    }

    #[test]
    fn display_super_modifier_platform() {
        let kb = KeyBind {
            modifiers: vec![Modifier::Super],
            key: "Q".into(),
        };
        let display = keybind_to_display(&kb);

        if cfg!(target_os = "macos") {
            assert_eq!(display, "\u{2318}Q"); // ⌘Q
        } else if cfg!(target_os = "windows") {
            assert_eq!(display, "Win+Q");
        } else {
            assert_eq!(display, "Super+Q");
        }
    }

    #[test]
    fn display_special_keys_macos() {
        if !cfg!(target_os = "macos") {
            return;
        }

        let kb = KeyBind {
            modifiers: vec![Modifier::Super],
            key: "Enter".into(),
        };
        assert_eq!(keybind_to_display(&kb), "\u{2318}\u{21A9}"); // ⌘↩
    }

    #[test]
    fn keybind_serialization_roundtrip() {
        let kb = KeyBind {
            modifiers: vec![Modifier::Ctrl, Modifier::Shift],
            key: "T".into(),
        };

        let json = serde_json::to_string(&kb).unwrap();
        let deserialized: KeyBind = serde_json::from_str(&json).unwrap();
        assert_eq!(kb, deserialized);
    }
}
