use super::Action;

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
