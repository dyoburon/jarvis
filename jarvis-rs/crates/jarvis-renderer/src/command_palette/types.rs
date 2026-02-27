use jarvis_common::actions::Action;

/// A single item in the command palette.
#[derive(Debug, Clone)]
pub struct PaletteItem {
    /// The action this item triggers.
    pub action: Action,
    /// Human-readable label.
    pub label: String,
    /// The keybind display string (e.g. "⌘T"), if one is bound.
    pub keybind_display: Option<String>,
}
