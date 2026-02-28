use super::types::{KeyBind, Modifier};

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

fn display_modifier(modifier: Modifier) -> String {
    if cfg!(target_os = "macos") {
        match modifier {
            Modifier::Ctrl => "\u{2303}".into(),  // ⌃
            Modifier::Alt => "\u{2325}".into(),   // ⌥
            Modifier::Shift => "\u{21E7}".into(), // ⇧
            Modifier::Super => "\u{2318}".into(), // ⌘
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
        match key {
            "Enter" => "\u{21A9}".into(),     // ↩
            "Backspace" => "\u{232B}".into(), // ⌫
            "Delete" => "\u{2326}".into(),    // ⌦
            "Escape" => "\u{238B}".into(),    // ⎋
            "Tab" => "\u{21E5}".into(),       // ⇥
            "Space" => "\u{2423}".into(),     // ␣
            "Up" => "\u{2191}".into(),        // ↑
            "Down" => "\u{2193}".into(),      // ↓
            "Left" => "\u{2190}".into(),      // ←
            "Right" => "\u{2192}".into(),     // →
            other => other.to_string(),
        }
    } else {
        key.to_string()
    }
}

fn join_display_parts(parts: &[String]) -> String {
    if cfg!(target_os = "macos") {
        parts.join("")
    } else {
        parts.join("+")
    }
}
