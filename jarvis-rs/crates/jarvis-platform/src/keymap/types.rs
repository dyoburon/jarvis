use serde::{Deserialize, Serialize};

/// A keyboard modifier key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Modifier {
    /// Ctrl key on all platforms.
    Ctrl,
    /// Alt key (Option on macOS).
    Alt,
    /// Shift key.
    Shift,
    /// Super key: Cmd on macOS, Win on Windows, Super on Linux.
    Super,
}

/// A key binding consisting of zero or more modifiers and a key name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyBind {
    pub modifiers: Vec<Modifier>,
    pub key: String,
}
