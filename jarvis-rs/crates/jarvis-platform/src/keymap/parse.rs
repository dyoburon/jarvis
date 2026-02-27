use jarvis_common::PlatformError;

use super::types::{KeyBind, Modifier};

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
        return Err(PlatformError::NotSupported("empty keybind string".into()));
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
                key = Some(normalize_key_name(token));
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

    let key =
        key.ok_or_else(|| PlatformError::NotSupported("keybind has no key component".into()))?;

    Ok(KeyBind { modifiers, key })
}

pub(super) fn normalize_modifier(token: &str) -> Option<Modifier> {
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

pub(super) fn normalize_key_name(token: &str) -> String {
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
