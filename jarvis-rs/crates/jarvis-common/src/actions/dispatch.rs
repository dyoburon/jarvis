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
            Action::LaunchGame(ref name) => match name.as_str() {
                "tetris" => "Play Tetris",
                "asteroids" => "Play Asteroids",
                "minesweeper" => "Play Minesweeper",
                "pinball" => "Play Pinball",
                "doodlejump" => "Play Doodle Jump",
                "draw" => "Open Draw",
                "subway" => "Play Subway Surfers",
                "videoplayer" => "Open Video Player",
                _ => "Launch Game",
            },
            Action::OpenURL(ref url) => {
                if url.contains("kartbros") {
                    "Play KartBros"
                } else {
                    "Open Website"
                }
            }
            Action::PairMobile => "Pair Mobile Device",
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
            Action::LaunchGame("tetris".into()),
            Action::LaunchGame("asteroids".into()),
            Action::LaunchGame("minesweeper".into()),
            Action::LaunchGame("pinball".into()),
            Action::LaunchGame("doodlejump".into()),
            Action::LaunchGame("draw".into()),
            Action::LaunchGame("subway".into()),
            Action::OpenURL("https://kartbros.io".into()),
            Action::PairMobile,
            Action::ReloadConfig,
            Action::Quit,
        ]
    }
}
