use serde::{Deserialize, Serialize};

/// Direction for pane resizing and swapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResizeDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Every user-triggerable action in the application.
///
/// Keybinds, command palette, and CLI all resolve to an `Action`.
/// The app state dispatcher matches on this enum to route to subsystems.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Action {
    // -- Pane / Tiling --
    NewPane,
    ClosePane,
    SplitHorizontal,
    SplitVertical,
    FocusPane(u32),
    FocusNextPane,
    FocusPrevPane,
    ZoomPane,
    ResizePane {
        direction: ResizeDirection,
        delta: i32,
    },
    SwapPane(ResizeDirection),

    // -- Window --
    ToggleFullscreen,
    Quit,

    // -- UI --
    OpenCommandPalette,
    OpenSettings,
    CloseOverlay,

    // -- AI / Voice --
    OpenAssistant,
    PushToTalk,
    ReleasePushToTalk,

    // -- Terminal --
    ScrollUp(u32),
    ScrollDown(u32),
    ScrollToTop,
    ScrollToBottom,
    Copy,
    Paste,
    SelectAll,
    SearchOpen,
    SearchClose,
    SearchNext,
    SearchPrev,
    ClearTerminal,
    ResetTerminal,

    // -- Config --
    ReloadConfig,

    // -- Noop --
    None,
}

impl Action {
    /// Human-readable label for display in the command palette.
    pub fn label(&self) -> &'static str {
        match self {
            Action::NewPane => "New Pane",
            Action::ClosePane => "Close Pane",
            Action::SplitHorizontal => "Split Horizontal",
            Action::SplitVertical => "Split Vertical",
            Action::FocusPane(1) => "Focus Pane 1",
            Action::FocusPane(2) => "Focus Pane 2",
            Action::FocusPane(3) => "Focus Pane 3",
            Action::FocusPane(4) => "Focus Pane 4",
            Action::FocusPane(5) => "Focus Pane 5",
            Action::FocusPane(_) => "Focus Pane",
            Action::FocusNextPane => "Focus Next Pane",
            Action::FocusPrevPane => "Focus Previous Pane",
            Action::ZoomPane => "Zoom Pane",
            Action::ResizePane { .. } => "Resize Pane",
            Action::SwapPane(_) => "Swap Pane",
            Action::ToggleFullscreen => "Toggle Fullscreen",
            Action::Quit => "Quit",
            Action::OpenCommandPalette => "Command Palette",
            Action::OpenSettings => "Open Settings",
            Action::CloseOverlay => "Close Overlay",
            Action::OpenAssistant => "Open Assistant",
            Action::PushToTalk => "Push to Talk",
            Action::ReleasePushToTalk => "Release Push to Talk",
            Action::ScrollUp(_) => "Scroll Up",
            Action::ScrollDown(_) => "Scroll Down",
            Action::ScrollToTop => "Scroll to Top",
            Action::ScrollToBottom => "Scroll to Bottom",
            Action::Copy => "Copy",
            Action::Paste => "Paste",
            Action::SelectAll => "Select All",
            Action::SearchOpen => "Find",
            Action::SearchClose => "Close Find",
            Action::SearchNext => "Find Next",
            Action::SearchPrev => "Find Previous",
            Action::ClearTerminal => "Clear Terminal",
            Action::ResetTerminal => "Reset Terminal",
            Action::ReloadConfig => "Reload Config",
            Action::None => "None",
        }
    }

    /// All actions that should appear in the command palette.
    pub fn palette_actions() -> Vec<Action> {
        vec![
            Action::NewPane,
            Action::ClosePane,
            Action::SplitHorizontal,
            Action::SplitVertical,
            Action::FocusNextPane,
            Action::FocusPrevPane,
            Action::ZoomPane,
            Action::ToggleFullscreen,
            Action::OpenSettings,
            Action::OpenAssistant,
            Action::Copy,
            Action::Paste,
            Action::SelectAll,
            Action::SearchOpen,
            Action::ScrollToTop,
            Action::ScrollToBottom,
            Action::ClearTerminal,
            Action::ResetTerminal,
            Action::ReloadConfig,
            Action::Quit,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_palette_actions_have_labels() {
        for action in Action::palette_actions() {
            let label = action.label();
            assert!(!label.is_empty(), "action {:?} has empty label", action);
        }
    }

    #[test]
    fn palette_actions_not_empty() {
        assert!(!Action::palette_actions().is_empty());
    }

    #[test]
    fn focus_pane_labels() {
        assert_eq!(Action::FocusPane(1).label(), "Focus Pane 1");
        assert_eq!(Action::FocusPane(5).label(), "Focus Pane 5");
        assert_eq!(Action::FocusPane(99).label(), "Focus Pane");
    }

    #[test]
    fn action_serde_roundtrip() {
        let actions = vec![
            Action::NewPane,
            Action::FocusPane(3),
            Action::ResizePane {
                direction: ResizeDirection::Left,
                delta: 10,
            },
            Action::ScrollUp(5),
        ];

        for action in &actions {
            let json = serde_json::to_string(action).unwrap();
            let deserialized: Action = serde_json::from_str(&json).unwrap();
            assert_eq!(*action, deserialized);
        }
    }

    #[test]
    fn resize_direction_serde() {
        let dir = ResizeDirection::Up;
        let json = serde_json::to_string(&dir).unwrap();
        let back: ResizeDirection = serde_json::from_str(&json).unwrap();
        assert_eq!(dir, back);
    }
}
