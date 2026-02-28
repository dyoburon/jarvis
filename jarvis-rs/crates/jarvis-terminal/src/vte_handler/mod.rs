mod csi_dispatch;
mod esc_osc;
mod handler;
mod perform;
mod sgr;

pub use handler::*;

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::TerminalColor;

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
