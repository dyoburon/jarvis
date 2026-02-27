//! Input processing — decides whether a key press becomes a keybind action
//! or terminal bytes.
//!
//! The [`InputProcessor`] sits between winit events and the rest of the app.
//! It first checks the [`KeybindRegistry`] for matching keybinds, then falls
//! back to encoding keys for the terminal PTY.

mod encoding;
mod processor;
mod types;

pub use encoding::encode_key_for_terminal;
pub use processor::InputProcessor;
pub use types::{InputMode, InputResult, Modifiers};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::KeybindRegistry;
    use jarvis_common::actions::Action;
    use jarvis_config::schema::KeybindConfig;

    fn make_registry() -> KeybindRegistry {
        KeybindRegistry::from_config(&KeybindConfig::default())
    }

    fn mods(ctrl: bool, alt: bool, shift: bool, super_key: bool) -> Modifiers {
        Modifiers {
            ctrl,
            alt,
            shift,
            super_key,
        }
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
        assert_eq!(
            encode_key_for_terminal("Up", false, false, false),
            b"\x1b[A"
        );
        assert_eq!(
            encode_key_for_terminal("Down", false, false, false),
            b"\x1b[B"
        );
        assert_eq!(
            encode_key_for_terminal("Right", false, false, false),
            b"\x1b[C"
        );
        assert_eq!(
            encode_key_for_terminal("Left", false, false, false),
            b"\x1b[D"
        );
    }

    #[test]
    fn function_keys() {
        assert_eq!(
            encode_key_for_terminal("F1", false, false, false),
            b"\x1bOP"
        );
        assert_eq!(
            encode_key_for_terminal("F5", false, false, false),
            b"\x1b[15~"
        );
        assert_eq!(
            encode_key_for_terminal("F12", false, false, false),
            b"\x1b[24~"
        );
    }

    #[test]
    fn backspace_and_delete() {
        assert_eq!(
            encode_key_for_terminal("Backspace", false, false, false),
            b"\x7f"
        );
        assert_eq!(
            encode_key_for_terminal("Delete", false, false, false),
            b"\x1b[3~"
        );
    }

    #[test]
    fn bracketed_paste() {
        let mut proc = InputProcessor::new();
        proc.set_bracketed_paste(true);

        let bytes = proc.encode_paste("hello world");
        assert_eq!(bytes, b"\x1b[200~hello world\x1b[201~");
    }

    #[test]
    fn unbracketed_paste() {
        let proc = InputProcessor::new();
        let bytes = proc.encode_paste("hello");
        assert_eq!(bytes, b"hello");
    }

    #[test]
    fn escape_key() {
        assert_eq!(
            encode_key_for_terminal("Escape", false, false, false),
            b"\x1b"
        );
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
