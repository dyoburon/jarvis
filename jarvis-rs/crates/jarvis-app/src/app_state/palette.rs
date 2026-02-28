//! Command palette key handling.

use jarvis_common::actions::Action;
use jarvis_platform::input_processor::InputMode;

use super::core::JarvisApp;

impl JarvisApp {
    /// Handle key events for the command palette.
    pub(super) fn handle_palette_key(&mut self, key_name: &str, is_press: bool) -> bool {
        if !is_press || !self.command_palette_open {
            return false;
        }

        let palette = match self.command_palette.as_mut() {
            Some(p) => p,
            None => return false,
        };

        match key_name {
            "Escape" => {
                self.dispatch(Action::CloseOverlay);
                true
            }
            "Enter" => {
                if let Some(action) = palette.confirm() {
                    self.command_palette_open = false;
                    self.command_palette = None;
                    self.input.set_mode(InputMode::Terminal);
                    self.dispatch(action);
                }
                true
            }
            "Up" => {
                palette.select_prev();
                true
            }
            "Down" => {
                palette.select_next();
                true
            }
            "Backspace" => {
                palette.backspace();
                true
            }
            "Tab" => {
                palette.select_next();
                true
            }
            _ => {
                if key_name.len() == 1 {
                    let ch = key_name.chars().next().unwrap();
                    if ch.is_ascii_graphic() || ch == ' ' {
                        palette.append_char(ch.to_ascii_lowercase());
                        return true;
                    }
                }
                false
            }
        }
    }
}
