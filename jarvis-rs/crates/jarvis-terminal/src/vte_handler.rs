use tracing::trace;
use vte::{Params, Perform};

use crate::grid::{Grid, TerminalColor};

// ---------------------------------------------------------------------------
// VteHandler
// ---------------------------------------------------------------------------

/// Wraps a terminal [`Grid`] and a VTE [`vte::Parser`], driving the grid in
/// response to incoming byte streams.
///
/// Because `vte::Parser::advance` borrows the `Perform` implementor mutably,
/// we split the parser out so that `Grid` can serve as the performer directly
/// through the `GridPerformer` new-type wrapper.
pub struct VteHandler {
    grid: Grid,
    parser: vte::Parser,
}

impl VteHandler {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            grid: Grid::new(cols, rows),
            parser: vte::Parser::new(),
        }
    }

    /// Feed raw bytes from the PTY into the parser, updating the grid.
    pub fn process(&mut self, bytes: &[u8]) {
        // We need to hand the parser a &mut Perform, but the parser itself is
        // also &mut.  Because Grid is a *separate* field we can safely split
        // the borrows via a temporary wrapper.
        let grid = &mut self.grid as *mut Grid;
        // SAFETY: `parser.advance` will only call methods on the performer
        // (which accesses `grid`).  `parser` and `grid` are disjoint fields.
        let performer = unsafe { &mut *grid };
        self.parser.advance(performer, bytes);
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    /// Returns a snapshot of which rows are dirty, then clears all dirty flags.
    pub fn take_dirty(&mut self) -> Vec<bool> {
        self.grid.take_dirty()
    }
}

// ---------------------------------------------------------------------------
// Perform implementation for Grid
// ---------------------------------------------------------------------------

impl Perform for Grid {
    fn print(&mut self, c: char) {
        self.put_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x08 => self.backspace(), // BS
            0x09 => self.tab(),       // HT
            0x0A..=0x0C => {
                // LF, VT, FF
                self.newline();
            }
            0x0D => self.carriage_return(), // CR
            0x07 => {
                // BEL
                trace!("BEL received");
            }
            _ => {
                trace!("unhandled execute byte: 0x{byte:02X}");
            }
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, action: char) {
        // Collect params into a flat fixed-size array for convenience.  Most
        // sequences only care about the first sub-parameter of each parameter
        // group.  A stack buffer avoids heap allocation on every CSI dispatch.
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
            'A' => {
                // CUU - cursor up
                self.move_cursor_relative(-(p1_one() as i32), 0);
            }
            'B' => {
                // CUD - cursor down
                self.move_cursor_relative(p1_one() as i32, 0);
            }
            'C' => {
                // CUF - cursor forward
                self.move_cursor_relative(0, p1_one() as i32);
            }
            'D' => {
                // CUB - cursor back
                self.move_cursor_relative(0, -(p1_one() as i32));
            }
            'E' => {
                // CNL - cursor next line
                self.move_cursor_relative(p1_one() as i32, 0);
                self.cursor.col = 0;
            }
            'F' => {
                // CPL - cursor previous line
                self.move_cursor_relative(-(p1_one() as i32), 0);
                self.cursor.col = 0;
            }
            'G' => {
                // CHA - cursor horizontal absolute (1-based)
                let col = (p1().max(1) as usize).saturating_sub(1);
                self.move_cursor(self.cursor.row, col);
            }
            'H' | 'f' => {
                // CUP / HVP - cursor position (1-based)
                let row = (p1().max(1) as usize).saturating_sub(1);
                let col = (p2().max(1) as usize).saturating_sub(1);
                self.move_cursor(row, col);
            }
            'd' => {
                // VPA - line position absolute (1-based)
                let row = (p1().max(1) as usize).saturating_sub(1);
                self.move_cursor(row, self.cursor.col);
            }

            // -- erasing --------------------------------------------------------
            'J' => {
                // ED - erase in display
                self.erase_in_display(p1());
            }
            'K' => {
                // EL - erase in line
                self.erase_in_line(p1());
            }
            'X' => {
                // ECH - erase characters
                self.erase_chars(p1_one());
            }

            // -- line / char insert-delete --------------------------------------
            'L' => {
                // IL - insert lines
                self.insert_lines(p1_one());
            }
            'M' => {
                // DL - delete lines
                self.delete_lines(p1_one());
            }
            'P' => {
                // DCH - delete chars
                self.delete_chars(p1_one());
            }
            '@' => {
                // ICH - insert blank chars
                self.insert_blank_chars(p1_one());
            }

            // -- scrolling ------------------------------------------------------
            'S' => {
                // SU - scroll up
                self.scroll_up(p1_one());
            }
            'T' => {
                // SD - scroll down
                self.scroll_down(p1_one());
            }

            // -- SGR (select graphic rendition) ---------------------------------
            'm' => {
                self.handle_sgr(params);
            }

            // -- scroll region --------------------------------------------------
            'r' => {
                // DECSTBM - set scrolling region (1-based)
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

            // -- device status report (ignored) ---------------------------------
            'n' => {}

            // -- window manipulation (ignored for now) --------------------------
            't' => {}

            _ => {
                trace!("unhandled CSI action: '{action}' params={flat:?}");
            }
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
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

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
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

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {
        // DCS hook - no-op for now.
    }

    fn put(&mut self, _byte: u8) {
        // DCS put - no-op for now.
    }

    fn unhook(&mut self) {
        // DCS unhook - no-op for now.
    }
}

// ---------------------------------------------------------------------------
// SGR parsing helper
// ---------------------------------------------------------------------------

impl Grid {
    /// Parse SGR (Select Graphic Rendition) parameters and apply them to
    /// `self.attrs`.
    fn handle_sgr(&mut self, params: &Params) {
        let mut iter = params.iter();

        // If there are no params at all, treat as SGR 0 (reset).
        let first = match iter.next() {
            Some(sub) => sub,
            None => {
                self.attrs = Default::default();
                return;
            }
        };

        // We process the first sub-param group, then continue with the rest.
        // A fixed-size stack buffer avoids heap allocation on every SGR call.
        let mut groups_buf: [&[u16]; 32] = [&[]; 32];
        groups_buf[0] = first;
        let mut groups_len = 1;
        for sub in iter {
            if groups_len < groups_buf.len() {
                groups_buf[groups_len] = sub;
                groups_len += 1;
            }
        }
        let groups = &groups_buf[..groups_len];

        let mut i = 0;
        while i < groups.len() {
            let sub = groups[i];
            let code = sub[0];
            match code {
                0 => self.attrs = Default::default(),
                1 => self.attrs.bold = true,
                2 => self.attrs.dim = true,
                3 => self.attrs.italic = true,
                4 => self.attrs.underline = true,
                5 => self.attrs.blink = true,
                7 => self.attrs.inverse = true,
                8 => self.attrs.hidden = true,
                9 => self.attrs.strikethrough = true,
                22 => {
                    self.attrs.bold = false;
                    self.attrs.dim = false;
                }
                23 => self.attrs.italic = false,
                24 => self.attrs.underline = false,
                25 => self.attrs.blink = false,
                27 => self.attrs.inverse = false,
                28 => self.attrs.hidden = false,
                29 => self.attrs.strikethrough = false,
                // Standard foreground colors (30-37).
                30..=37 => {
                    self.attrs.fg = TerminalColor::Indexed((code - 30) as u8);
                }
                // Extended foreground.
                38 => {
                    i += 1;
                    self.parse_extended_color(&groups, &mut i, true);
                    continue; // i already advanced
                }
                39 => self.attrs.fg = TerminalColor::Default,
                // Standard background colors (40-47).
                40..=47 => {
                    self.attrs.bg = TerminalColor::Indexed((code - 40) as u8);
                }
                // Extended background.
                48 => {
                    i += 1;
                    self.parse_extended_color(&groups, &mut i, false);
                    continue;
                }
                49 => self.attrs.bg = TerminalColor::Default,
                // Bright foreground (90-97).
                90..=97 => {
                    self.attrs.fg = TerminalColor::Indexed((code - 90 + 8) as u8);
                }
                // Bright background (100-107).
                100..=107 => {
                    self.attrs.bg = TerminalColor::Indexed((code - 100 + 8) as u8);
                }
                _ => {
                    trace!("unhandled SGR code: {code}");
                }
            }
            i += 1;
        }
    }

    /// Parse an extended color specification (used after SGR 38 or 48).
    /// `is_fg` controls whether the result is applied to foreground or
    /// background.
    ///
    /// Expected forms:
    ///   38;5;N        — 256-color palette
    ///   38;2;R;G;B    — 24-bit true-color
    fn parse_extended_color(&mut self, groups: &[&[u16]], i: &mut usize, is_fg: bool) {
        if *i >= groups.len() {
            return;
        }
        let mode = groups[*i][0];
        match mode {
            5 => {
                // 256-color: next param is the color index.
                *i += 1;
                if *i < groups.len() {
                    let idx = groups[*i][0] as u8;
                    let color = TerminalColor::Indexed(idx);
                    if is_fg {
                        self.attrs.fg = color;
                    } else {
                        self.attrs.bg = color;
                    }
                    *i += 1;
                }
            }
            2 => {
                // True-color: next three params are R, G, B.
                if *i + 3 <= groups.len() {
                    let r = groups[*i + 1][0] as u8;
                    let g = groups[*i + 2][0] as u8;
                    let b = groups[*i + 3][0] as u8;
                    let color = TerminalColor::Rgb(r, g, b);
                    if is_fg {
                        self.attrs.fg = color;
                    } else {
                        self.attrs.bg = color;
                    }
                    *i += 4;
                } else {
                    *i = groups.len();
                }
            }
            _ => {
                *i += 1;
            }
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn handler(cols: usize, rows: usize) -> VteHandler {
        VteHandler::new(cols, rows)
    }

    #[test]
    fn process_plain_text_populates_grid() {
        let mut h = handler(80, 24);
        h.process(b"Hello");
        assert_eq!(h.grid().row_to_string(0), "Hello");
        assert_eq!(h.grid().cursor.col, 5);
    }

    #[test]
    fn process_csi_cursor_movement() {
        let mut h = handler(80, 24);
        // CUP(3,5) = ESC[3;5H  (1-based => row 2, col 4)
        h.process(b"\x1b[3;5H");
        assert_eq!(h.grid().cursor.row, 2);
        assert_eq!(h.grid().cursor.col, 4);

        // CUF(2) = ESC[2C
        h.process(b"\x1b[2C");
        assert_eq!(h.grid().cursor.col, 6);

        // CUU(1) = ESC[1A
        h.process(b"\x1b[1A");
        assert_eq!(h.grid().cursor.row, 1);

        // CUB(3) = ESC[3D
        h.process(b"\x1b[3D");
        assert_eq!(h.grid().cursor.col, 3);
    }

    #[test]
    fn process_sgr_bold_color() {
        let mut h = handler(80, 24);
        // SGR 1 (bold) + 31 (red fg)
        h.process(b"\x1b[1;31m");
        assert!(h.grid().attrs.bold);
        assert_eq!(h.grid().attrs.fg, TerminalColor::Indexed(1));

        // Write a character and verify its attributes.
        h.process(b"X");
        let cell = h.grid().cell(0, 0);
        assert!(cell.attrs.bold);
        assert_eq!(cell.attrs.fg, TerminalColor::Indexed(1));

        // SGR 0 resets.
        h.process(b"\x1b[0m");
        assert!(!h.grid().attrs.bold);
        assert_eq!(h.grid().attrs.fg, TerminalColor::Default);
    }

    #[test]
    fn process_cr_lf_sequence() {
        let mut h = handler(80, 24);
        h.process(b"AB\r\nCD");
        assert_eq!(h.grid().row_to_string(0), "AB");
        assert_eq!(h.grid().row_to_string(1), "CD");
    }

    #[test]
    fn process_erase_in_display() {
        let mut h = handler(10, 3);
        h.process(b"AAAAAAAAAA"); // fill row 0
        h.process(b"\r\n");
        h.process(b"BBBBBBBBBB"); // fill row 1
                                  // Move cursor to row 0, col 5.
        h.process(b"\x1b[1;6H");
        // ED 0: clear from cursor to end.
        h.process(b"\x1b[0J");
        assert_eq!(h.grid().row_to_string(0), "AAAAA");
        assert_eq!(h.grid().row_to_string(1), "");
    }

    #[test]
    fn process_256_color_sgr() {
        let mut h = handler(80, 24);
        // SGR 38;5;196 = 256-color fg index 196
        h.process(b"\x1b[38;5;196m");
        assert_eq!(h.grid().attrs.fg, TerminalColor::Indexed(196));

        // SGR 48;5;42 = 256-color bg index 42
        h.process(b"\x1b[48;5;42m");
        assert_eq!(h.grid().attrs.bg, TerminalColor::Indexed(42));
    }

    #[test]
    fn process_truecolor_sgr() {
        let mut h = handler(80, 24);
        // SGR 38;2;255;128;0 = truecolor fg
        h.process(b"\x1b[38;2;255;128;0m");
        assert_eq!(h.grid().attrs.fg, TerminalColor::Rgb(255, 128, 0));

        // SGR 48;2;10;20;30 = truecolor bg
        h.process(b"\x1b[48;2;10;20;30m");
        assert_eq!(h.grid().attrs.bg, TerminalColor::Rgb(10, 20, 30));
    }

    #[test]
    fn process_alternate_screen_enter_exit() {
        let mut h = handler(10, 3);
        h.process(b"Hello");
        assert_eq!(h.grid().row_to_string(0), "Hello");

        // Enter alternate screen: CSI ? 1049 h
        h.process(b"\x1b[?1049h");
        assert_eq!(h.grid().row_to_string(0), ""); // blank screen

        h.process(b"Alt");
        assert_eq!(h.grid().row_to_string(0), "Alt");

        // Exit alternate screen: CSI ? 1049 l
        h.process(b"\x1b[?1049l");
        assert_eq!(h.grid().row_to_string(0), "Hello"); // restored
    }

    #[test]
    fn process_scroll_region() {
        let mut h = handler(10, 5);
        // Fill rows.
        h.process(b"Row0\r\nRow1\r\nRow2\r\nRow3\r\nRow4");
        // Set scroll region to rows 2-4 (1-based).
        h.process(b"\x1b[2;4r");
        assert_eq!(h.grid().scroll_top, 1);
        assert_eq!(h.grid().scroll_bottom, 3);
    }

    #[test]
    fn process_tab_stops() {
        let mut h = handler(80, 24);
        h.process(b"\t");
        assert_eq!(h.grid().cursor.col, 8);
        h.process(b"X");
        assert_eq!(h.grid().cursor.col, 9);
        h.process(b"\t");
        assert_eq!(h.grid().cursor.col, 16);
    }

    #[test]
    fn process_cursor_save_restore_via_esc() {
        let mut h = handler(80, 24);
        h.process(b"\x1b[5;10H"); // move to row 4, col 9
        h.process(b"\x1b7"); // DECSC
        h.process(b"\x1b[1;1H"); // move to 0,0
        h.process(b"\x1b8"); // DECRC
        assert_eq!(h.grid().cursor.row, 4);
        assert_eq!(h.grid().cursor.col, 9);
    }

    #[test]
    fn process_reverse_index() {
        let mut h = handler(10, 3);
        h.process(b"Row0\r\nRow1\r\nRow2");
        // Move to top row.
        h.process(b"\x1b[1;1H");
        // Reverse index at top should scroll down.
        h.process(b"\x1bM");
        assert_eq!(h.grid().cells[0][0].c, ' '); // new blank row
        assert_eq!(h.grid().row_to_string(1), "Row0");
    }

    #[test]
    fn process_full_reset() {
        let mut h = handler(80, 24);
        h.process(b"Hello");
        h.process(b"\x1bc"); // RIS
        assert_eq!(h.grid().row_to_string(0), "");
        assert_eq!(h.grid().cursor.row, 0);
        assert_eq!(h.grid().cursor.col, 0);
    }

    #[test]
    fn process_osc_title() {
        let mut h = handler(80, 24);
        h.process(b"\x1b]2;My Terminal\x07");
        assert_eq!(h.grid().title, "My Terminal");
    }

    #[test]
    fn process_cursor_visibility() {
        let mut h = handler(80, 24);
        assert!(h.grid().cursor.visible);
        h.process(b"\x1b[?25l"); // hide cursor
        assert!(!h.grid().cursor.visible);
        h.process(b"\x1b[?25h"); // show cursor
        assert!(h.grid().cursor.visible);
    }

    #[test]
    fn process_insert_delete_lines() {
        let mut h = handler(10, 4);
        h.process(b"AAA\r\nBBB\r\nCCC\r\nDDD");
        h.process(b"\x1b[2;1H"); // row 1 (0-indexed)
        h.process(b"\x1b[1L"); // insert 1 line
        assert_eq!(h.grid().row_to_string(0), "AAA");
        assert_eq!(h.grid().row_to_string(1), ""); // inserted blank
        assert_eq!(h.grid().row_to_string(2), "BBB");
        assert_eq!(h.grid().row_to_string(3), "CCC");
    }

    #[test]
    fn process_insert_delete_chars() {
        let mut h = handler(10, 1);
        h.process(b"ABCDEF");
        h.process(b"\x1b[1;2H"); // col 1
        h.process(b"\x1b[2P"); // delete 2 chars
        assert_eq!(h.grid().row_to_string(0), "ADEF");
    }

    #[test]
    fn process_bright_colors() {
        let mut h = handler(80, 24);
        // SGR 90 = bright black fg (index 8)
        h.process(b"\x1b[90m");
        assert_eq!(h.grid().attrs.fg, TerminalColor::Indexed(8));
        // SGR 107 = bright white bg (index 15)
        h.process(b"\x1b[107m");
        assert_eq!(h.grid().attrs.bg, TerminalColor::Indexed(15));
    }

    #[test]
    fn process_sgr_multiple_attrs() {
        let mut h = handler(80, 24);
        // bold + italic + underline + strikethrough
        h.process(b"\x1b[1;3;4;9m");
        let a = &h.grid().attrs;
        assert!(a.bold);
        assert!(a.italic);
        assert!(a.underline);
        assert!(a.strikethrough);

        // Reset individual attrs.
        h.process(b"\x1b[22;23;24;29m");
        let a = &h.grid().attrs;
        assert!(!a.bold);
        assert!(!a.italic);
        assert!(!a.underline);
        assert!(!a.strikethrough);
    }

    #[test]
    fn process_erase_line_modes() {
        let mut h = handler(10, 1);
        h.process(b"ABCDEFGHIJ");
        h.process(b"\x1b[1;6H"); // col 5 (0-indexed)
        h.process(b"\x1b[1K"); // erase to left (inclusive)
        assert_eq!(h.grid().cell(0, 0).c, ' ');
        assert_eq!(h.grid().cell(0, 4).c, ' ');
        assert_eq!(h.grid().cell(0, 5).c, ' ');
        assert_eq!(h.grid().cell(0, 6).c, 'G');
    }

    #[test]
    fn process_vpa() {
        let mut h = handler(80, 24);
        h.process(b"\x1b[10d"); // VPA 10 (1-based => row 9)
        assert_eq!(h.grid().cursor.row, 9);
    }

    #[test]
    fn process_cnl_cpl() {
        let mut h = handler(80, 24);
        h.process(b"\x1b[5;10H"); // row 4, col 9
        h.process(b"\x1b[2E"); // CNL 2: down 2, col 0
        assert_eq!(h.grid().cursor.row, 6);
        assert_eq!(h.grid().cursor.col, 0);
        h.process(b"\x1b[5;10H");
        h.process(b"\x1b[1F"); // CPL 1: up 1, col 0
        assert_eq!(h.grid().cursor.row, 3);
        assert_eq!(h.grid().cursor.col, 0);
    }

    #[test]
    fn process_cha() {
        let mut h = handler(80, 24);
        h.process(b"\x1b[20G"); // CHA 20 (1-based => col 19)
        assert_eq!(h.grid().cursor.col, 19);
    }
}
