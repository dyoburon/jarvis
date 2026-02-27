//! Winit key name normalization.
//!
//! Converts winit's `Key` debug representations to the normalized key names
//! used by [`KeyCombo`](crate::input::KeyCombo) and [`parse_keybind`](crate::keymap::parse_keybind).

/// Convert a winit key name to the normalized string used by `KeyCombo`.
///
/// Winit uses names like `"ArrowUp"`, `"Backspace"`, `" "` for space, etc.
/// Our keybind system uses `"Up"`, `"Backspace"`, `"Space"`.
pub fn normalize_winit_key(key: &str) -> String {
    match key {
        // Arrow keys
        "ArrowUp" => "Up".to_string(),
        "ArrowDown" => "Down".to_string(),
        "ArrowLeft" => "Left".to_string(),
        "ArrowRight" => "Right".to_string(),

        // Navigation
        "Home" => "Home".to_string(),
        "End" => "End".to_string(),
        "PageUp" => "PageUp".to_string(),
        "PageDown" => "PageDown".to_string(),
        "Insert" => "Insert".to_string(),

        // Editing
        "Backspace" => "Backspace".to_string(),
        "Delete" => "Delete".to_string(),
        "Enter" => "Enter".to_string(),
        "Tab" => "Tab".to_string(),
        "Escape" => "Escape".to_string(),

        // Whitespace
        " " => "Space".to_string(),

        // Punctuation (keep as-is, matching normalize_key_name output)
        "." => ".".to_string(),
        "," => ",".to_string(),
        "/" => "/".to_string(),
        "\\" => "\\".to_string(),
        ";" => ";".to_string(),
        "'" => "'".to_string(),
        "[" => "[".to_string(),
        "]" => "]".to_string(),
        "-" => "-".to_string(),
        "=" => "=".to_string(),
        "`" => "`".to_string(),

        _ => {
            // Single character keys: uppercase for consistency
            if key.len() == 1 {
                key.to_uppercase()
            } else {
                // F1, F2, ..., F12 and other named keys pass through
                key.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arrow_keys() {
        assert_eq!(normalize_winit_key("ArrowUp"), "Up");
        assert_eq!(normalize_winit_key("ArrowDown"), "Down");
        assert_eq!(normalize_winit_key("ArrowLeft"), "Left");
        assert_eq!(normalize_winit_key("ArrowRight"), "Right");
    }

    #[test]
    fn special_keys() {
        assert_eq!(normalize_winit_key("Backspace"), "Backspace");
        assert_eq!(normalize_winit_key("Delete"), "Delete");
        assert_eq!(normalize_winit_key("Enter"), "Enter");
        assert_eq!(normalize_winit_key("Tab"), "Tab");
        assert_eq!(normalize_winit_key("Escape"), "Escape");
    }

    #[test]
    fn space() {
        assert_eq!(normalize_winit_key(" "), "Space");
    }

    #[test]
    fn single_chars_uppercased() {
        assert_eq!(normalize_winit_key("a"), "A");
        assert_eq!(normalize_winit_key("z"), "Z");
        assert_eq!(normalize_winit_key("g"), "G");
    }

    #[test]
    fn function_keys_passthrough() {
        assert_eq!(normalize_winit_key("F1"), "F1");
        assert_eq!(normalize_winit_key("F12"), "F12");
    }

    #[test]
    fn punctuation() {
        assert_eq!(normalize_winit_key("."), ".");
        assert_eq!(normalize_winit_key(","), ",");
        assert_eq!(normalize_winit_key("/"), "/");
    }

    #[test]
    fn navigation_keys() {
        assert_eq!(normalize_winit_key("Home"), "Home");
        assert_eq!(normalize_winit_key("End"), "End");
        assert_eq!(normalize_winit_key("PageUp"), "PageUp");
        assert_eq!(normalize_winit_key("PageDown"), "PageDown");
    }
}
