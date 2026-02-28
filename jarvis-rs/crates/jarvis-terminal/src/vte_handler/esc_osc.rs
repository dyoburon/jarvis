//! ESC dispatch and OSC dispatch handlers.

use tracing::trace;

use crate::grid::Grid;

impl Grid {
    /// Handle an ESC (escape) dispatch.
    pub(crate) fn dispatch_esc(&mut self, intermediates: &[u8], byte: u8) {
        match (byte, intermediates) {
            (b'7', _) => self.save_cursor(),    // DECSC
            (b'8', _) => self.restore_cursor(), // DECRC
            (b'M', _) => {
                // RI - reverse index: move cursor up, scroll down at top
                if self.cursor.row == self.scroll_top {
                    self.scroll_down(1);
                } else if self.cursor.row > 0 {
                    self.cursor.row -= 1;
                }
            }
            (b'D', _) => {
                // IND - index: move cursor down, scroll up at bottom
                self.newline();
            }
            (b'E', _) => {
                // NEL - next line
                self.carriage_return();
                self.newline();
            }
            (b'c', _) => {
                // RIS - full reset
                self.reset();
            }
            _ => {
                trace!("unhandled ESC dispatch: byte=0x{byte:02X} intermediates={intermediates:?}");
            }
        }
    }

    /// Handle an OSC (Operating System Command) dispatch.
    pub(crate) fn dispatch_osc(&mut self, params: &[&[u8]]) {
        if params.is_empty() {
            return;
        }
        // First param is the numeric command.
        let cmd = std::str::from_utf8(params[0])
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(u16::MAX);

        match cmd {
            0 | 2 => {
                // Set window title.
                if let Some(title_bytes) = params.get(1) {
                    if let Ok(title) = std::str::from_utf8(title_bytes) {
                        self.title = title.to_string();
                    }
                }
            }
            _ => {
                trace!("unhandled OSC command: {cmd}");
            }
        }
    }
}
