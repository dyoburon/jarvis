use serde::{Deserialize, Serialize};

use super::ResizeDirection;

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
    OpenChat,
    CloseOverlay,

    // -- Games --
    LaunchGame(String),

    // -- Web --
    OpenURL(String),

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

    // -- Mobile --
    PairMobile,
    RevokeMobilePairing,

    // -- Config --
    ReloadConfig,

    // -- Noop --
    None,
}
