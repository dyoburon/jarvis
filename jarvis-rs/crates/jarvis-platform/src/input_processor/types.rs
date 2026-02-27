use jarvis_common::actions::Action;

/// The result of processing a key event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputResult {
    /// The key matched a keybind — dispatch this action.
    Action(Action),
    /// The key should be sent to the terminal as raw bytes.
    TerminalInput(Vec<u8>),
    /// The key was consumed (modifier-only press, or non-terminal mode input).
    Consumed,
}

/// The current input context — affects how keys are routed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    /// Normal terminal mode: keybinds intercept, rest goes to PTY.
    Terminal,
    /// Command palette is open: keys go to palette filter.
    CommandPalette,
    /// Settings UI is open.
    Settings,
    /// AI assistant panel is open: keys go to assistant input.
    Assistant,
}

/// Modifier key state bundled for passing to input processing.
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
}
