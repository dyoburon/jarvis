//! Keybind validation utilities.

use crate::schema::KeybindConfig;
use jarvis_common::ConfigError;
use std::collections::HashMap;

/// Returns all keybinds as `(name, binding)` pairs.
pub fn all_keybinds(config: &KeybindConfig) -> Vec<(&str, &str)> {
    vec![
        ("push_to_talk", &config.push_to_talk),
        ("open_assistant", &config.open_assistant),
        ("new_panel", &config.new_panel),
        ("close_panel", &config.close_panel),
        ("toggle_fullscreen", &config.toggle_fullscreen),
        ("open_settings", &config.open_settings),
        ("focus_panel_1", &config.focus_panel_1),
        ("focus_panel_2", &config.focus_panel_2),
        ("focus_panel_3", &config.focus_panel_3),
        ("focus_panel_4", &config.focus_panel_4),
        ("focus_panel_5", &config.focus_panel_5),
        ("cycle_panels", &config.cycle_panels),
        ("cycle_panels_reverse", &config.cycle_panels_reverse),
        ("split_vertical", &config.split_vertical),
        ("split_horizontal", &config.split_horizontal),
        ("close_pane", &config.close_pane),
        ("command_palette", &config.command_palette),
    ]
}

/// Validate that no two keybinds are mapped to the same key combination.
pub fn validate_no_duplicates(config: &KeybindConfig) -> Result<(), ConfigError> {
    let binds = all_keybinds(config);
    let mut seen: HashMap<&str, &str> = HashMap::new();

    for (name, binding) in &binds {
        if let Some(existing_name) = seen.get(binding) {
            return Err(ConfigError::ValidationError(format!(
                "duplicate keybind '{binding}': assigned to both '{existing_name}' and '{name}'"
            )));
        }
        seen.insert(binding, name);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_keybinds_have_no_duplicates() {
        let config = KeybindConfig::default();
        assert!(validate_no_duplicates(&config).is_ok());
    }

    #[test]
    fn all_keybinds_returns_17_entries() {
        let config = KeybindConfig::default();
        let binds = all_keybinds(&config);
        assert_eq!(binds.len(), 17);
    }

    #[test]
    fn detects_duplicate_keybinds() {
        let config = KeybindConfig {
            push_to_talk: "Cmd+G".into(), // same as open_assistant
            open_assistant: "Cmd+G".into(),
            ..Default::default()
        };
        let result = validate_no_duplicates(&config);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("duplicate keybind"));
        assert!(err.contains("Cmd+G"));
    }

    #[test]
    fn all_keybinds_has_correct_names() {
        let config = KeybindConfig::default();
        let binds = all_keybinds(&config);
        let names: Vec<&str> = binds.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"push_to_talk"));
        assert!(names.contains(&"open_assistant"));
        assert!(names.contains(&"new_panel"));
        assert!(names.contains(&"close_panel"));
        assert!(names.contains(&"toggle_fullscreen"));
        assert!(names.contains(&"open_settings"));
        assert!(names.contains(&"focus_panel_1"));
        assert!(names.contains(&"focus_panel_5"));
        assert!(names.contains(&"cycle_panels"));
        assert!(names.contains(&"cycle_panels_reverse"));
    }

    #[test]
    fn custom_keybinds_no_duplicates() {
        let config = KeybindConfig {
            push_to_talk: "Cmd+Space".into(),
            open_assistant: "Cmd+G".into(),
            ..Default::default()
        };
        assert!(validate_no_duplicates(&config).is_ok());
    }
}
