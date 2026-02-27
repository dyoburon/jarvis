mod content;
mod core;
mod cursor;
mod dirty;
mod erase;
mod line_edit;
mod scroll;
mod types;

pub use self::core::*;
pub use types::*;

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_has_correct_dimensions() {
        let g = Grid::new(80, 24);
        assert_eq!(g.cols, 80);
        assert_eq!(g.rows, 24);
        assert_eq!(g.cells.len(), 24);
        assert_eq!(g.cells[0].len(), 80);
    }

    #[test]
    fn put_char_writes_at_cursor_and_advances() {
        let mut g = Grid::new(80, 24);
        g.put_char('A');
        assert_eq!(g.cells[0][0].c, 'A');
        assert_eq!(g.cursor.col, 1);
        g.put_char('B');
        assert_eq!(g.cells[0][1].c, 'B');
        assert_eq!(g.cursor.col, 2);
    }

    #[test]
    fn put_char_wraps_at_end_of_line() {
        let mut g = Grid::new(5, 3);
        for ch in "ABCDE".chars() {
            g.put_char(ch);
        }
        // After writing 5 chars in a 5-col grid, wrap_pending should be true.
        assert!(g.wrap_pending);
        assert_eq!(g.cursor.row, 0);
        // Writing one more should trigger the wrap.
        g.put_char('F');
        assert_eq!(g.cursor.row, 1);
        assert_eq!(g.cells[1][0].c, 'F');
    }

    #[test]
    fn newline_scrolls_when_at_bottom() {
        let mut g = Grid::new(5, 3);
        g.cursor.row = 2; // bottom row
        g.cells[0][0].c = 'X';
        g.newline();
        // Should have scrolled: row 0 content should now be gone.
        assert_eq!(g.cursor.row, 2);
        assert_eq!(g.cells[0][0].c, ' ');
    }

    #[test]
    fn carriage_return_moves_to_column_0() {
        let mut g = Grid::new(80, 24);
        g.cursor.col = 40;
        g.carriage_return();
        assert_eq!(g.cursor.col, 0);
    }

    #[test]
    fn tab_advances_to_next_tab_stop() {
        let mut g = Grid::new(80, 24);
        assert_eq!(g.cursor.col, 0);
        g.tab();
        assert_eq!(g.cursor.col, 8);
        g.tab();
        assert_eq!(g.cursor.col, 16);
    }

    #[test]
    fn erase_in_display_mode_2_clears_all() {
        let mut g = Grid::new(5, 3);
        g.put_char('A');
        g.put_char('B');
        g.erase_in_display(2);
        for r in 0..3 {
            for c in 0..5 {
                assert_eq!(g.cells[r][c].c, ' ');
            }
        }
    }

    #[test]
    fn erase_in_line_mode_0_clears_to_right() {
        let mut g = Grid::new(5, 3);
        for ch in "HELLO".chars() {
            g.put_char(ch);
        }
        g.cursor.col = 2;
        g.erase_in_line(0);
        assert_eq!(g.cells[0][0].c, 'H');
        assert_eq!(g.cells[0][1].c, 'E');
        assert_eq!(g.cells[0][2].c, ' ');
        assert_eq!(g.cells[0][3].c, ' ');
        assert_eq!(g.cells[0][4].c, ' ');
    }

    #[test]
    fn cursor_save_restore() {
        let mut g = Grid::new(80, 24);
        g.cursor.row = 5;
        g.cursor.col = 10;
        g.save_cursor();
        g.cursor.row = 0;
        g.cursor.col = 0;
        g.restore_cursor();
        assert_eq!(g.cursor.row, 5);
        assert_eq!(g.cursor.col, 10);
    }

    #[test]
    fn wide_char_takes_2_cells() {
        let mut g = Grid::new(80, 24);
        // CJK character (U+4E16 = world) has width 2.
        g.put_char('\u{4E16}');
        assert_eq!(g.cells[0][0].c, '\u{4E16}');
        assert_eq!(g.cells[0][0].width, 2);
        assert_eq!(g.cells[0][1].width, 0); // continuation
        assert_eq!(g.cursor.col, 2);
    }

    #[test]
    fn scroll_up_moves_lines_up() {
        let mut g = Grid::new(5, 3);
        g.cells[0][0].c = 'A';
        g.cells[1][0].c = 'B';
        g.cells[2][0].c = 'C';
        let scrolled = g.scroll_up(1);
        assert_eq!(scrolled.len(), 1);
        assert_eq!(scrolled[0][0].c, 'A');
        assert_eq!(g.cells[0][0].c, 'B');
        assert_eq!(g.cells[1][0].c, 'C');
        assert_eq!(g.cells[2][0].c, ' '); // new blank row
    }

    #[test]
    fn resize_preserves_content() {
        let mut g = Grid::new(5, 3);
        g.cells[0][0].c = 'X';
        g.cells[1][0].c = 'Y';
        g.resize(10, 5);
        assert_eq!(g.cols, 10);
        assert_eq!(g.rows, 5);
        assert_eq!(g.cells[0][0].c, 'X');
        assert_eq!(g.cells[1][0].c, 'Y');
    }

    #[test]
    fn row_to_string_returns_correct_text() {
        let mut g = Grid::new(80, 24);
        for ch in "Hello".chars() {
            g.put_char(ch);
        }
        assert_eq!(g.row_to_string(0), "Hello");
    }

    #[test]
    fn alternate_screen_saves_and_restores() {
        let mut g = Grid::new(5, 3);
        g.cells[0][0].c = 'A';
        g.enter_alternate_screen();
        // Screen should be blank now.
        assert_eq!(g.cells[0][0].c, ' ');
        g.cells[0][0].c = 'Z';
        g.exit_alternate_screen();
        // Should be back to the original content.
        assert_eq!(g.cells[0][0].c, 'A');
    }

    #[test]
    fn scroll_down_moves_lines_down() {
        let mut g = Grid::new(5, 3);
        g.cells[0][0].c = 'A';
        g.cells[1][0].c = 'B';
        g.cells[2][0].c = 'C';
        g.scroll_down(1);
        assert_eq!(g.cells[0][0].c, ' '); // new blank row
        assert_eq!(g.cells[1][0].c, 'A');
        assert_eq!(g.cells[2][0].c, 'B');
    }

    #[test]
    fn backspace_moves_cursor_left() {
        let mut g = Grid::new(80, 24);
        g.cursor.col = 5;
        g.backspace();
        assert_eq!(g.cursor.col, 4);
        g.cursor.col = 0;
        g.backspace();
        assert_eq!(g.cursor.col, 0); // does not go below 0
    }

    #[test]
    fn delete_chars_shifts_left() {
        let mut g = Grid::new(5, 1);
        for ch in "ABCDE".chars() {
            g.put_char(ch);
        }
        g.cursor.col = 1;
        g.delete_chars(2);
        assert_eq!(g.cells[0][0].c, 'A');
        assert_eq!(g.cells[0][1].c, 'D');
        assert_eq!(g.cells[0][2].c, 'E');
        assert_eq!(g.cells[0][3].c, ' ');
    }

    #[test]
    fn insert_blank_chars_shifts_right() {
        let mut g = Grid::new(5, 1);
        for ch in "ABCDE".chars() {
            g.put_char(ch);
        }
        g.cursor.col = 1;
        g.insert_blank_chars(2);
        assert_eq!(g.cells[0][0].c, 'A');
        assert_eq!(g.cells[0][1].c, ' ');
        assert_eq!(g.cells[0][2].c, ' ');
        assert_eq!(g.cells[0][3].c, 'B');
        assert_eq!(g.cells[0][4].c, 'C');
    }

    #[test]
    fn content_to_string_all_rows() {
        let mut g = Grid::new(10, 2);
        for ch in "Hello".chars() {
            g.put_char(ch);
        }
        g.cursor.row = 1;
        g.cursor.col = 0;
        for ch in "World".chars() {
            g.put_char(ch);
        }
        let s = g.content_to_string();
        assert_eq!(s, "Hello\nWorld");
    }

    #[test]
    fn insert_lines_within_scroll_region() {
        let mut g = Grid::new(5, 4);
        g.cells[0][0].c = 'A';
        g.cells[1][0].c = 'B';
        g.cells[2][0].c = 'C';
        g.cells[3][0].c = 'D';
        g.cursor.row = 1;
        g.insert_lines(1);
        assert_eq!(g.cells[0][0].c, 'A');
        assert_eq!(g.cells[1][0].c, ' '); // inserted blank
        assert_eq!(g.cells[2][0].c, 'B');
        assert_eq!(g.cells[3][0].c, 'C');
    }

    #[test]
    fn delete_lines_within_scroll_region() {
        let mut g = Grid::new(5, 4);
        g.cells[0][0].c = 'A';
        g.cells[1][0].c = 'B';
        g.cells[2][0].c = 'C';
        g.cells[3][0].c = 'D';
        g.cursor.row = 1;
        g.delete_lines(1);
        assert_eq!(g.cells[0][0].c, 'A');
        assert_eq!(g.cells[1][0].c, 'C');
        assert_eq!(g.cells[2][0].c, 'D');
        assert_eq!(g.cells[3][0].c, ' '); // blank at bottom
    }

    #[test]
    fn set_scroll_region_and_scroll() {
        let mut g = Grid::new(5, 5);
        for (i, ch) in "ABCDE".chars().enumerate() {
            g.cells[i][0].c = ch;
        }
        g.set_scroll_region(1, 3);
        g.scroll_up(1);
        assert_eq!(g.cells[0][0].c, 'A'); // outside region
        assert_eq!(g.cells[1][0].c, 'C'); // was row 2
        assert_eq!(g.cells[2][0].c, 'D'); // was row 3
        assert_eq!(g.cells[3][0].c, ' '); // new blank in region
        assert_eq!(g.cells[4][0].c, 'E'); // outside region
    }

    #[test]
    fn move_cursor_clamps_to_bounds() {
        let mut g = Grid::new(10, 5);
        g.move_cursor(100, 100);
        assert_eq!(g.cursor.row, 4);
        assert_eq!(g.cursor.col, 9);
    }

    #[test]
    fn move_cursor_relative_clamps() {
        let mut g = Grid::new(10, 5);
        g.cursor.row = 2;
        g.cursor.col = 5;
        g.move_cursor_relative(-10, -10);
        assert_eq!(g.cursor.row, 0);
        assert_eq!(g.cursor.col, 0);
        g.move_cursor_relative(100, 100);
        assert_eq!(g.cursor.row, 4);
        assert_eq!(g.cursor.col, 9);
    }

    #[test]
    fn erase_in_display_mode_0_clears_below() {
        let mut g = Grid::new(5, 3);
        for r in 0..3 {
            for c in 0..5 {
                g.cells[r][c].c = 'X';
            }
        }
        g.cursor.row = 1;
        g.cursor.col = 2;
        g.erase_in_display(0);
        // Row 0 untouched.
        for c in 0..5 {
            assert_eq!(g.cells[0][c].c, 'X');
        }
        // Row 1 cols 0-1 untouched, 2-4 cleared.
        assert_eq!(g.cells[1][0].c, 'X');
        assert_eq!(g.cells[1][1].c, 'X');
        assert_eq!(g.cells[1][2].c, ' ');
        assert_eq!(g.cells[1][3].c, ' ');
        // Row 2 all cleared.
        for c in 0..5 {
            assert_eq!(g.cells[2][c].c, ' ');
        }
    }

    #[test]
    fn erase_in_display_mode_1_clears_above() {
        let mut g = Grid::new(5, 3);
        for r in 0..3 {
            for c in 0..5 {
                g.cells[r][c].c = 'X';
            }
        }
        g.cursor.row = 1;
        g.cursor.col = 2;
        g.erase_in_display(1);
        // Row 0 all cleared.
        for c in 0..5 {
            assert_eq!(g.cells[0][c].c, ' ');
        }
        // Row 1 cols 0-2 cleared, 3-4 untouched.
        assert_eq!(g.cells[1][0].c, ' ');
        assert_eq!(g.cells[1][1].c, ' ');
        assert_eq!(g.cells[1][2].c, ' ');
        assert_eq!(g.cells[1][3].c, 'X');
        assert_eq!(g.cells[1][4].c, 'X');
        // Row 2 untouched.
        for c in 0..5 {
            assert_eq!(g.cells[2][c].c, 'X');
        }
    }

    #[test]
    fn erase_in_line_mode_1_clears_to_left() {
        let mut g = Grid::new(5, 1);
        for ch in "ABCDE".chars() {
            g.put_char(ch);
        }
        g.cursor.col = 2;
        g.erase_in_line(1);
        assert_eq!(g.cells[0][0].c, ' ');
        assert_eq!(g.cells[0][1].c, ' ');
        assert_eq!(g.cells[0][2].c, ' ');
        assert_eq!(g.cells[0][3].c, 'D');
        assert_eq!(g.cells[0][4].c, 'E');
    }

    #[test]
    fn erase_in_line_mode_2_clears_whole_line() {
        let mut g = Grid::new(5, 1);
        for ch in "ABCDE".chars() {
            g.put_char(ch);
        }
        g.erase_in_line(2);
        for c in 0..5 {
            assert_eq!(g.cells[0][c].c, ' ');
        }
    }

    #[test]
    fn reset_restores_defaults() {
        let mut g = Grid::new(80, 24);
        g.put_char('Z');
        g.cursor.row = 10;
        g.attrs.bold = true;
        g.reset();
        assert_eq!(g.cells[0][0].c, ' ');
        assert_eq!(g.cursor.row, 0);
        assert_eq!(g.cursor.col, 0);
        assert!(!g.attrs.bold);
    }

    #[test]
    fn erase_chars_replaces_with_blanks() {
        let mut g = Grid::new(5, 1);
        for ch in "ABCDE".chars() {
            g.put_char(ch);
        }
        g.cursor.col = 1;
        g.erase_chars(2);
        assert_eq!(g.cells[0][0].c, 'A');
        assert_eq!(g.cells[0][1].c, ' ');
        assert_eq!(g.cells[0][2].c, ' ');
        assert_eq!(g.cells[0][3].c, 'D');
        assert_eq!(g.cells[0][4].c, 'E');
    }

    #[test]
    fn wide_char_at_end_of_line_wraps() {
        let mut g = Grid::new(5, 2);
        g.cursor.col = 4; // last column
        g.put_char('\u{4E16}'); // wide char width=2
                                // Should wrap to next line.
        assert_eq!(g.cursor.row, 1);
        assert_eq!(g.cells[1][0].c, '\u{4E16}');
        assert_eq!(g.cells[1][0].width, 2);
    }

    // -- dirty tracking tests -----------------------------------------------

    #[test]
    fn new_grid_starts_all_dirty() {
        let mut g = Grid::new(80, 24);
        let dirty = g.take_dirty();
        assert!(dirty.iter().all(|&d| d));
        assert_eq!(dirty.len(), 24);
    }

    #[test]
    fn take_dirty_clears_flags() {
        let mut g = Grid::new(80, 24);
        let _ = g.take_dirty();
        let dirty = g.take_dirty();
        assert!(dirty.iter().all(|&d| !d));
    }

    #[test]
    fn put_char_marks_cursor_row_dirty() {
        let mut g = Grid::new(80, 24);
        let _ = g.take_dirty();
        g.put_char('A');
        let dirty = g.take_dirty();
        assert!(dirty[0]);
        assert!(!dirty[1]);
    }

    #[test]
    fn scroll_up_marks_scroll_region_dirty() {
        let mut g = Grid::new(5, 5);
        let _ = g.take_dirty();
        g.scroll_up(1);
        let dirty = g.take_dirty();
        // Default scroll region is 0..4
        assert!(dirty.iter().all(|&d| d));
    }

    #[test]
    fn erase_in_display_mode2_marks_all_dirty() {
        let mut g = Grid::new(5, 3);
        let _ = g.take_dirty();
        g.erase_in_display(2);
        let dirty = g.take_dirty();
        assert!(dirty.iter().all(|&d| d));
    }

    #[test]
    fn erase_in_line_marks_only_that_row() {
        let mut g = Grid::new(10, 3);
        g.cursor.row = 1;
        let _ = g.take_dirty();
        g.erase_in_line(2);
        let dirty = g.take_dirty();
        assert!(!dirty[0]);
        assert!(dirty[1]);
        assert!(!dirty[2]);
    }

    #[test]
    fn move_cursor_marks_old_and_new_rows() {
        let mut g = Grid::new(80, 24);
        let _ = g.take_dirty();
        g.move_cursor(5, 0);
        let dirty = g.take_dirty();
        assert!(dirty[0]); // old row
        assert!(dirty[5]); // new row
        assert!(!dirty[3]); // untouched
    }

    #[test]
    fn resize_marks_all_dirty() {
        let mut g = Grid::new(80, 24);
        let _ = g.take_dirty();
        g.resize(40, 12);
        let dirty = g.take_dirty();
        assert_eq!(dirty.len(), 12);
        assert!(dirty.iter().all(|&d| d));
    }

    #[test]
    fn any_dirty_returns_false_after_take() {
        let mut g = Grid::new(80, 24);
        assert!(g.any_dirty());
        let _ = g.take_dirty();
        assert!(!g.any_dirty());
        g.put_char('X');
        assert!(g.any_dirty());
    }
}
