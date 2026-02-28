//! CSI dispatch: cursor movement, erasing, insert/delete, scrolling,
//! SGR, scroll region, DEC private modes.

use tracing::trace;
use vte::Params;

use crate::grid::Grid;

impl Grid {
    /// Handle a CSI (Control Sequence Introducer) dispatch.
    pub(crate) fn dispatch_csi(&mut self, params: &Params, intermediates: &[u8], action: char) {
        // Collect params into a flat fixed-size array for convenience.
        let mut flat_buf = [0u16; 32];
        let mut flat_len = 0;
        for sub in params.iter() {
            if flat_len < flat_buf.len() {
                flat_buf[flat_len] = sub[0];
                flat_len += 1;
            }
        }
        let flat = &flat_buf[..flat_len];

        let p1 = || flat.first().copied().unwrap_or(0);
        let p1_one = || {
            let v = p1();
            if v == 0 {
                1
            } else {
                v as usize
            }
        };
        let p2 = || flat.get(1).copied().unwrap_or(0);

        let has_private = intermediates.first() == Some(&b'?');

        match action {
            // -- cursor movement ------------------------------------------------
            'A' => self.move_cursor_relative(-(p1_one() as i32), 0),
            'B' => self.move_cursor_relative(p1_one() as i32, 0),
            'C' => self.move_cursor_relative(0, p1_one() as i32),
            'D' => self.move_cursor_relative(0, -(p1_one() as i32)),
            'E' => {
                self.move_cursor_relative(p1_one() as i32, 0);
                self.cursor.col = 0;
            }
            'F' => {
                self.move_cursor_relative(-(p1_one() as i32), 0);
                self.cursor.col = 0;
            }
            'G' => {
                let col = (p1().max(1) as usize).saturating_sub(1);
                self.move_cursor(self.cursor.row, col);
            }
            'H' | 'f' => {
                let row = (p1().max(1) as usize).saturating_sub(1);
                let col = (p2().max(1) as usize).saturating_sub(1);
                self.move_cursor(row, col);
            }
            'd' => {
                let row = (p1().max(1) as usize).saturating_sub(1);
                self.move_cursor(row, self.cursor.col);
            }

            // -- erasing --------------------------------------------------------
            'J' => self.erase_in_display(p1()),
            'K' => self.erase_in_line(p1()),
            'X' => self.erase_chars(p1_one()),

            // -- line / char insert-delete --------------------------------------
            'L' => self.insert_lines(p1_one()),
            'M' => self.delete_lines(p1_one()),
            'P' => self.delete_chars(p1_one()),
            '@' => self.insert_blank_chars(p1_one()),

            // -- scrolling ------------------------------------------------------
            'S' => {
                self.scroll_up(p1_one());
            }
            'T' => self.scroll_down(p1_one()),

            // -- SGR (select graphic rendition) ---------------------------------
            'm' => self.handle_sgr(params),

            // -- scroll region --------------------------------------------------
            'r' => {
                if !has_private {
                    let top = (p1().max(1) as usize).saturating_sub(1);
                    let bot = if p2() == 0 {
                        self.rows.saturating_sub(1)
                    } else {
                        (p2() as usize).saturating_sub(1)
                    };
                    self.set_scroll_region(top, bot);
                    self.move_cursor(0, 0);
                }
            }

            // -- cursor save / restore ------------------------------------------
            's' => {
                if !has_private {
                    self.save_cursor();
                }
            }
            'u' => {
                if !has_private {
                    self.restore_cursor();
                }
            }

            // -- DEC private modes (CSI ? ... h / l) ----------------------------
            'h' => {
                if has_private {
                    for p in flat {
                        match p {
                            1049 => self.enter_alternate_screen(),
                            25 => self.cursor.visible = true,
                            7 => self.auto_wrap = true,
                            6 => {
                                self.origin_mode = true;
                                self.move_cursor(0, 0);
                            }
                            _ => {
                                trace!("unhandled DEC set mode: {p}");
                            }
                        }
                    }
                }
            }
            'l' => {
                if has_private {
                    for p in flat {
                        match p {
                            1049 => self.exit_alternate_screen(),
                            25 => self.cursor.visible = false,
                            7 => self.auto_wrap = false,
                            6 => {
                                self.origin_mode = false;
                                self.move_cursor(0, 0);
                            }
                            _ => {
                                trace!("unhandled DEC reset mode: {p}");
                            }
                        }
                    }
                }
            }

            // -- ignored --------------------------------------------------------
            'n' | 't' => {}

            _ => {
                trace!("unhandled CSI action: '{action}' params={flat:?}");
            }
        }
    }
}
