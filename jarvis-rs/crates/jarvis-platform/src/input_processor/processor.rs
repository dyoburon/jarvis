use jarvis_common::actions::Action;

use crate::input::{KeyCombo, KeybindRegistry};

use super::encoding::encode_key_for_terminal;
use super::types::{InputMode, InputResult, Modifiers};

/// Processes keyboard input from winit into actions or terminal bytes.
pub struct InputProcessor {
    mode: InputMode,
    bracketed_paste: bool,
}

impl InputProcessor {
    pub fn new() -> Self {
        Self {
            mode: InputMode::Terminal,
            bracketed_paste: false,
        }
    }

    pub fn set_mode(&mut self, mode: InputMode) {
        self.mode = mode;
    }

    pub fn mode(&self) -> InputMode {
        self.mode
    }

    pub fn set_bracketed_paste(&mut self, enabled: bool) {
        self.bracketed_paste = enabled;
    }

    /// Process a key event.
    ///
    /// For key presses: checks keybinds first, then encodes for terminal.
    /// For key releases: only checks for push-to-talk release.
    pub fn process_key(
        &self,
        registry: &KeybindRegistry,
        key_name: &str,
        mods: Modifiers,
        is_press: bool,
    ) -> InputResult {
        let combo = KeyCombo::from_winit(
            mods.ctrl,
            mods.alt,
            mods.shift,
            mods.super_key,
            key_name.to_string(),
        );

        if !is_press {
            if let Some(Action::PushToTalk) = registry.lookup(&combo) {
                return InputResult::Action(Action::ReleasePushToTalk);
            }
            return InputResult::Consumed;
        }

        if let Some(action) = registry.lookup(&combo) {
            return InputResult::Action(action.clone());
        }

        if self.mode != InputMode::Terminal {
            return InputResult::Consumed;
        }

        let bytes = encode_key_for_terminal(key_name, mods.ctrl, mods.alt, mods.shift);
        if bytes.is_empty() {
            InputResult::Consumed
        } else {
            InputResult::TerminalInput(bytes)
        }
    }

    /// Encode pasted text, optionally with bracketed paste markers.
    pub fn encode_paste(&self, text: &str) -> Vec<u8> {
        if self.bracketed_paste {
            let mut bytes = b"\x1b[200~".to_vec();
            bytes.extend_from_slice(text.as_bytes());
            bytes.extend_from_slice(b"\x1b[201~");
            bytes
        } else {
            text.as_bytes().to_vec()
        }
    }
}

impl Default for InputProcessor {
    fn default() -> Self {
        Self::new()
    }
}
