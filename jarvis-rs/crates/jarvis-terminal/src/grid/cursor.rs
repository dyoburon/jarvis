//! Cursor movement, save/restore, and control characters.

use super::core::Grid;

impl Grid {
    /// Move cursor to an absolute position, clamped to grid bounds.
    pub fn move_cursor(&mut self, row: usize, col: usize) {
        let old_row = self.cursor.row;
        self.cursor.row = row.min(self.rows.saturating_sub(1));
        self.cursor.col = col.min(self.cols.saturating_sub(1));
        self.wrap_pending = false;
        self.mark_dirty(old_row);
        self.mark_dirty(self.cursor.row);
    }

    /// Move cursor relative to current position.
    pub fn move_cursor_relative(&mut self, d_row: i32, d_col: i32) {
        let old_row = self.cursor.row;
        let new_row = (self.cursor.row as i32 + d_row)
            .max(0)
            .min(self.rows.saturating_sub(1) as i32) as usize;
        let new_col = (self.cursor.col as i32 + d_col)
            .max(0)
            .min(self.cols.saturating_sub(1) as i32) as usize;
        self.cursor.row = new_row;
        self.cursor.col = new_col;
        self.wrap_pending = false;
        self.mark_dirty(old_row);
        self.mark_dirty(new_row);
    }

    // -- cursor save / restore (DECSC / DECRC) ------------------------------

    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some(self.cursor.clone());
    }

    pub fn restore_cursor(&mut self) {
        if let Some(saved) = self.saved_cursor.take() {
            self.cursor = saved;
            // Clamp to current dimensions.
            self.cursor.row = self.cursor.row.min(self.rows.saturating_sub(1));
            self.cursor.col = self.cursor.col.min(self.cols.saturating_sub(1));
        }
        self.wrap_pending = false;
    }

    // -- control characters -------------------------------------------------

    /// Line feed: move cursor down one line, scrolling if at the bottom of
    /// the scroll region.
    pub fn newline(&mut self) {
        if self.cursor.row == self.scroll_bottom {
            self.scroll_up(1);
        } else if self.cursor.row + 1 < self.rows {
            self.cursor.row += 1;
        }
    }

    pub fn carriage_return(&mut self) {
        self.cursor.col = 0;
        self.wrap_pending = false;
    }

    pub fn backspace(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        }
        self.wrap_pending = false;
    }

    /// Advance to the next tab stop.
    pub fn tab(&mut self) {
        let mut col = self.cursor.col + 1;
        while col < self.cols {
            if self.tab_stops.get(col).copied().unwrap_or(false) {
                break;
            }
            col += 1;
        }
        self.cursor.col = col.min(self.cols - 1);
        self.wrap_pending = false;
    }
}
