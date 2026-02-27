use jarvis_common::PlatformError;
use serde::{Deserialize, Serialize};

/// A keyboard modifier key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modifier {
    /// Ctrl key on all platforms.
    Ctrl,
    /// Alt key (Option on macOS).
    Alt,
    /// Shift key.
    Shift,
    /// Super key: Cmd on macOS, Win on Windows, Super on Linux.
    Super,
}

/// A key binding consisting of zero or more modifiers and a key name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBind {
    pub modifiers: Vec<Modifier>,
    pub key: String,
}

/// Parses a human-readable keybind string like `"Cmd+G"`, `"Ctrl+Shift+T"`,
/// or `"Option+Period"` into a [`KeyBind`].
///
/// Platform-specific normalization rules:
/// - `"Cmd"` / `"Command"` -> `Super` on macOS, `Ctrl` on Linux/Windows
/// - `"Option"` -> `Alt`
/// - `"Control"` / `"Ctrl"` -> `Ctrl`
/// - `"Win"` / `"Super"` / `"Meta"` -> `Super`
/// - `"Shift"` -> `Shift`
/// - `"Alt"` -> `Alt`
///
/// The last token that is not a recognized modifier becomes the key.
pub fn parse_keybind(s: &str) -> Result<KeyBind, PlatformError> {
    let tokens: Vec<&str> = s.split('+').map(|t| t.trim()).collect();

    if tokens.is_empty() || (tokens.len() == 1 && tokens[0].is_empty()) {
        return Err(PlatformError::NotSupported(
            "empty keybind string".into(),
        ));
    }

    let mut modifiers = Vec::new();
    let mut key: Option<String> = None;

    for (i, token) in tokens.iter().enumerate() {
        let is_last = i == tokens.len() - 1;

        match normalize_modifier(token) {
            Some(modifier) if !is_last => {
                if !modifiers.contains(&modifier) {
                    modifiers.push(modifier);
                }
            }
            Some(modifier) => {
                // Last token could be either a modifier used as a key or an
                // actual modifier. We treat it as a modifier if it is clearly
                // one and there are prior tokens, but since it is the last
                // token, we accept it as the key when no other key is present.
                // However, the common pattern is that the last part is the key.
                // If someone writes "Ctrl+Shift", the key is "Shift" which
                // could be ambiguous. We follow the convention that the final
                // token is always the key unless there is only one token and
                // it is a modifier. In single-token modifier case, treat it as
                // a key too (e.g. "Shift" alone is valid as a keybind).
                //
                // Re-check: if there are multiple tokens, the last one might
                // still be a modifier key used intentionally as the trigger.
                // We keep it as the key name.
                key = Some(normalize_key_name(token));
                // But if there is more than one token, also check if it should
                // be treated as a modifier. We opt for: always treat the last
                // token as the key.
                let _ = modifier; // suppress unused warning
            }
            None => {
                if is_last {
                    key = Some(normalize_key_name(token));
                } else {
                    return Err(PlatformError::NotSupported(format!(
                        "unrecognized modifier: {token}"
                    )));
                }
            }
        }
    }

    let key = key.ok_or_else(|| {
        PlatformError::NotSupported("keybind has no key component".into())
    })?;

    Ok(KeyBind { modifiers, key })
}

/// Converts a [`KeyBind`] into a platform-appropriate display string.
///
/// On macOS, modifiers are displayed as symbols. On other platforms, they
/// are displayed as text names separated by `+`.
pub fn keybind_to_display(kb: &KeyBind) -> String {
    let mut parts: Vec<String> = Vec::new();

    for modifier in &kb.modifiers {
        parts.push(display_modifier(*modifier));
    }

    parts.push(display_key(&kb.key));
    join_display_parts(&parts)
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn normalize_modifier(token: &str) -> Option<Modifier> {
    match token.to_lowercase().as_str() {
        "ctrl" | "control" => Some(Modifier::Ctrl),
        "alt" => Some(Modifier::Alt),
        "option" | "opt" => Some(Modifier::Alt),
        "shift" => Some(Modifier::Shift),
        "cmd" | "command" => {
            if cfg!(target_os = "macos") {
                Some(Modifier::Super)
            } else {
                Some(Modifier::Ctrl)
            }
        }
        "super" | "win" | "meta" => Some(Modifier::Super),
        _ => None,
    }
}

fn normalize_key_name(token: &str) -> String {
    // Capitalize first letter for consistent display
    let lower = token.to_lowercase();
    match lower.as_str() {
        "period" => ".".into(),
        "comma" => ",".into(),
        "slash" => "/".into(),
        "backslash" => "\\".into(),
        "space" => "Space".into(),
        "enter" | "return" => "Enter".into(),
        "escape" | "esc" => "Escape".into(),
        "tab" => "Tab".into(),
        "backspace" => "Backspace".into(),
        "delete" | "del" => "Delete".into(),
        "up" => "Up".into(),
        "down" => "Down".into(),
        "left" => "Left".into(),
        "right" => "Right".into(),
        "home" => "Home".into(),
        "end" => "End".into(),
        "pageup" => "PageUp".into(),
        "pagedown" => "PageDown".into(),
        _ => {
            // Single character keys are uppercased; multi-char are title-cased
            if token.len() == 1 {
                token.to_uppercase()
            } else {
                let mut chars = lower.chars();
                match chars.next() {
                    Some(c) => {
                        let upper: String = c.to_uppercase().collect();
                        format!("{upper}{}", chars.as_str())
                    }
                    None => lower,
                }
            }
        }
    }
}

fn display_modifier(modifier: Modifier) -> String {
    if cfg!(target_os = "macos") {
        match modifier {
            Modifier::Ctrl => "\u{2303}".into(),  // ⌃
            Modifier::Alt => "\u{2325}".into(),    // ⌥
            Modifier::Shift => "\u{21E7}".into(),  // ⇧
            Modifier::Super => "\u{2318}".into(),  // ⌘
        }
    } else {
        match modifier {
            Modifier::Ctrl => "Ctrl".into(),
            Modifier::Alt => "Alt".into(),
            Modifier::Shift => "Shift".into(),
            Modifier::Super => {
                if cfg!(target_os = "windows") {
                    "Win".into()
                } else {
                    "Super".into()
                }
            }
        }
    }
}

fn display_key(key: &str) -> String {
    if cfg!(target_os = "macos") {
        // On macOS, some keys have symbol representations
        match key {
            "Enter" => "\u{21A9}".into(),      // ↩
            "Backspace" => "\u{232B}".into(),   // ⌫
            "Delete" => "\u{2326}".into(),      // ⌦
            "Escape" => "\u{238B}".into(),      // ⎋
            "Tab" => "\u{21E5}".into(),         // ⇥
            "Space" => "\u{2423}".into(),       // ␣
            "Up" => "\u{2191}".into(),          // ↑
            "Down" => "\u{2193}".into(),        // ↓
            "Left" => "\u{2190}".into(),        // ←
            "Right" => "\u{2192}".into(),       // →
            other => other.to_string(),
        }
    } else {
        key.to_string()
    }
}

fn join_display_parts(parts: &[String]) -> String {
    if cfg!(target_os = "macos") {
        // macOS convention: no separator between symbols
        parts.join("")
    } else {
        parts.join("+")
    }
}

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
