//! Keyboard shortcuts configuration types.
//!
//! Named `keybind_config` to avoid clash with the crate-level `keybinds` module.

use serde::{Deserialize, Serialize};

/// Keyboard shortcuts configuration.
///
/// Format: "Modifier+Key" where Modifier is one of: Cmd, Option, Control, Shift.
/// Multiple modifiers: "Cmd+Shift+G".
/// Double press: "Escape+Escape".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct KeybindConfig {
    pub push_to_talk: String,
    pub open_assistant: String,
    pub new_panel: String,
    pub close_panel: String,
    pub toggle_fullscreen: String,
    pub open_settings: String,
    pub open_chat: String,
    pub focus_panel_1: String,
    pub focus_panel_2: String,
    pub focus_panel_3: String,
    pub focus_panel_4: String,
    pub focus_panel_5: String,
    pub cycle_panels: String,
    pub cycle_panels_reverse: String,
    pub split_vertical: String,
    pub split_horizontal: String,
    pub close_pane: String,
    pub command_palette: String,
    pub copy: String,
    pub paste: String,
}

impl Default for KeybindConfig {
    fn default() -> Self {
        Self {
            push_to_talk: "Option+Period".into(),
            open_assistant: "Cmd+G".into(),
            new_panel: "Cmd+T".into(),
            close_panel: "Escape+Escape".into(),
            toggle_fullscreen: "Cmd+F".into(),
            open_settings: "Cmd+,".into(),
            open_chat: "Cmd+J".into(),
            focus_panel_1: "Cmd+1".into(),
            focus_panel_2: "Cmd+2".into(),
            focus_panel_3: "Cmd+3".into(),
            focus_panel_4: "Cmd+4".into(),
            focus_panel_5: "Cmd+5".into(),
            cycle_panels: "Tab".into(),
            cycle_panels_reverse: "Shift+Tab".into(),
            split_vertical: "Cmd+D".into(),
            split_horizontal: "Cmd+Shift+D".into(),
            close_pane: "Cmd+W".into(),
            command_palette: "Cmd+Shift+P".into(),
            copy: "Cmd+C".into(),
            paste: "Cmd+V".into(),
        }
    }
}
