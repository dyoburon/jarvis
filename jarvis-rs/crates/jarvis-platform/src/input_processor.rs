//! Input processing — decides whether a key press becomes a keybind action
//! or terminal bytes.
//!
//! The [`InputProcessor`] sits between winit events and the rest of the app.
//! It first checks the [`KeybindRegistry`] for matching keybinds, then falls
//! back to encoding keys for the terminal PTY.

use jarvis_common::actions::Action;

use crate::input::{KeyCombo, KeybindRegistry};

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
}

/// Modifier key state bundled for passing to input processing.
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
    pub super_key: bool,
}

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
        let combo = KeyCombo::from_winit(mods.ctrl, mods.alt, mods.shift, mods.super_key, key_name.to_string());

        if !is_press {
            // Handle push-to-talk release
            if let Some(Action::PushToTalk) = registry.lookup(&combo) {
                return InputResult::Action(Action::ReleasePushToTalk);
            }
            return InputResult::Consumed;
        }

        // Check keybinds first (works in all modes)
        if let Some(action) = registry.lookup(&combo) {
            return InputResult::Action(action.clone());
        }

        // In non-terminal modes, keys are consumed (handled by the overlay)
        if self.mode != InputMode::Terminal {
            return InputResult::Consumed;
        }

        // Encode for terminal
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

/// Encode a key press into terminal escape sequences / bytes.
pub fn encode_key_for_terminal(key: &str, ctrl: bool, alt: bool, _shift: bool) -> Vec<u8> {
    let alt_prefix: &[u8] = if alt { b"\x1b" } else { b"" };

    match key {
        // Editing keys
        "Enter" => [alt_prefix, b"\r"].concat(),
        "Backspace" => [alt_prefix, b"\x7f"].concat(),
        "Tab" => b"\t".to_vec(),
        "Escape" => b"\x1b".to_vec(),
        "Space" => [alt_prefix, b" "].concat(),
        "Delete" => b"\x1b[3~".to_vec(),
        "Insert" => b"\x1b[2~".to_vec(),

        // Arrow keys
        "Up" => b"\x1b[A".to_vec(),
        "Down" => b"\x1b[B".to_vec(),
        "Right" => b"\x1b[C".to_vec(),
        "Left" => b"\x1b[D".to_vec(),

        // Navigation
        "Home" => b"\x1b[H".to_vec(),
        "End" => b"\x1b[F".to_vec(),
        "PageUp" => b"\x1b[5~".to_vec(),
        "PageDown" => b"\x1b[6~".to_vec(),

        // Function keys
        "F1" => b"\x1bOP".to_vec(),
        "F2" => b"\x1bOQ".to_vec(),
        "F3" => b"\x1bOR".to_vec(),
        "F4" => b"\x1bOS".to_vec(),
        "F5" => b"\x1b[15~".to_vec(),
        "F6" => b"\x1b[17~".to_vec(),
        "F7" => b"\x1b[18~".to_vec(),
        "F8" => b"\x1b[19~".to_vec(),
        "F9" => b"\x1b[20~".to_vec(),
        "F10" => b"\x1b[21~".to_vec(),
        "F11" => b"\x1b[23~".to_vec(),
        "F12" => b"\x1b[24~".to_vec(),

        _ => {
            if key.len() == 1 {
                let ch = key.chars().next().unwrap();
                if ctrl && ch.is_ascii_alphabetic() {
                    // Ctrl+A = 0x01, Ctrl+B = 0x02, ..., Ctrl+Z = 0x1A
                    let ctrl_byte = (ch.to_ascii_lowercase() as u8) - b'a' + 1;
                    [alt_prefix, &[ctrl_byte]].concat()
                } else if ctrl && ch == '[' {
                    // Ctrl+[ = ESC
                    b"\x1b".to_vec()
                } else if ctrl && ch == '\\' {
                    // Ctrl+\ = 0x1C
                    vec![0x1c]
                } else if ctrl && ch == ']' {
                    // Ctrl+] = 0x1D
                    vec![0x1d]
                } else {
                    // Regular character
                    [alt_prefix, key.as_bytes()].concat()
                }
            } else {
                // Unknown multi-char key name — consume
                Vec::new()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jarvis_config::schema::KeybindConfig;

    fn make_registry() -> KeybindRegistry {
        KeybindRegistry::from_config(&KeybindConfig::default())
    }

    fn mods(ctrl: bool, alt: bool, shift: bool, super_key: bool) -> Modifiers {
        Modifiers { ctrl, alt, shift, super_key }
    }

    #[test]
    fn terminal_input_regular_key() {
        let proc = InputProcessor::new();
        let reg = make_registry();

        let result = proc.process_key(&reg, "A", Modifiers::default(), true);
        assert_eq!(result, InputResult::TerminalInput(b"A".to_vec()));
    }

    #[test]
    fn terminal_input_enter() {
        let proc = InputProcessor::new();
        let reg = make_registry();

        let result = proc.process_key(&reg, "Enter", Modifiers::default(), true);
        assert_eq!(result, InputResult::TerminalInput(b"\r".to_vec()));
    }

    #[test]
    fn keybind_match() {
        let proc = InputProcessor::new();
        let reg = make_registry();

        // Cmd+T => NewPane (Super on macOS, Ctrl on Linux)
        let m = if cfg!(target_os = "macos") {
            mods(false, false, false, true)
        } else {
            mods(true, false, false, false)
        };
        let result = proc.process_key(&reg, "T", m, true);
        assert_eq!(result, InputResult::Action(Action::NewPane));
    }

    #[test]
    fn key_release_consumed() {
        let proc = InputProcessor::new();
        let reg = make_registry();

        let result = proc.process_key(&reg, "A", Modifiers::default(), false);
        assert_eq!(result, InputResult::Consumed);
    }

    #[test]
    fn command_palette_mode_consumes() {
        let mut proc = InputProcessor::new();
        proc.set_mode(InputMode::CommandPalette);
        let reg = make_registry();

        let result = proc.process_key(&reg, "A", Modifiers::default(), true);
        assert_eq!(result, InputResult::Consumed);
    }

    #[test]
    fn ctrl_c_encoding() {
        let bytes = encode_key_for_terminal("C", true, false, false);
        assert_eq!(bytes, vec![0x03]); // ETX
    }

    #[test]
    fn ctrl_d_encoding() {
        let bytes = encode_key_for_terminal("D", true, false, false);
        assert_eq!(bytes, vec![0x04]); // EOT
    }

    #[test]
    fn ctrl_z_encoding() {
        let bytes = encode_key_for_terminal("Z", true, false, false);
        assert_eq!(bytes, vec![0x1a]); // SUB
    }

    #[test]
    fn alt_prefix() {
        let bytes = encode_key_for_terminal("D", false, true, false);
        assert_eq!(bytes, vec![0x1b, b'D']);
    }

    #[test]
    fn arrow_keys() {
        assert_eq!(encode_key_for_terminal("Up", false, false, false), b"\x1b[A");
        assert_eq!(encode_key_for_terminal("Down", false, false, false), b"\x1b[B");
        assert_eq!(encode_key_for_terminal("Right", false, false, false), b"\x1b[C");
        assert_eq!(encode_key_for_terminal("Left", false, false, false), b"\x1b[D");
    }

    #[test]
    fn function_keys() {
        assert_eq!(encode_key_for_terminal("F1", false, false, false), b"\x1bOP");
        assert_eq!(encode_key_for_terminal("F5", false, false, false), b"\x1b[15~");
        assert_eq!(encode_key_for_terminal("F12", false, false, false), b"\x1b[24~");
    }

    #[test]
    fn backspace_and_delete() {
        assert_eq!(encode_key_for_terminal("Backspace", false, false, false), b"\x7f");
        assert_eq!(encode_key_for_terminal("Delete", false, false, false), b"\x1b[3~");
    }

    #[test]
    fn bracketed_paste() {
        let mut proc = InputProcessor::new();
        proc.set_bracketed_paste(true);

        let bytes = proc.encode_paste("hello world");
        assert_eq!(
            bytes,
            b"\x1b[200~hello world\x1b[201~"
        );
    }

    #[test]
    fn unbracketed_paste() {
        let proc = InputProcessor::new();
        let bytes = proc.encode_paste("hello");
        assert_eq!(bytes, b"hello");
    }

    #[test]
    fn escape_key() {
        assert_eq!(encode_key_for_terminal("Escape", false, false, false), b"\x1b");
    }

    #[test]
    fn tab_key() {
        assert_eq!(encode_key_for_terminal("Tab", false, false, false), b"\t");
    }

    #[test]
    fn unknown_key_empty() {
        let bytes = encode_key_for_terminal("UnknownKey", false, false, false);
        assert!(bytes.is_empty());
    }
}
