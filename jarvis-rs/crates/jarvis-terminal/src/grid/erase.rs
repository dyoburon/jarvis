//! Erase operations: erase in display, erase in line, erase chars.

use super::core::Grid;
use super::types::Cell;

impl Grid {
    /// Erase in display.
    ///   0 = cursor to end, 1 = start to cursor, 2 = entire screen,
    ///   3 = scrollback (no-op here).
    pub fn erase_in_display(&mut self, mode: u16) {
        let (row, col) = (self.cursor.row, self.cursor.col);
        match mode {
            0 => {
                for c in col..self.cols {
                    self.cells[row][c] = Cell::default();
                }
                for r in (row + 1)..self.rows {
                    for c in 0..self.cols {
                        self.cells[r][c] = Cell::default();
                    }
                }
                self.mark_range_dirty(row, self.rows);
            }
            1 => {
                for r in 0..row {
                    for c in 0..self.cols {
                        self.cells[r][c] = Cell::default();
                    }
                }
                for c in 0..=col.min(self.cols.saturating_sub(1)) {
                    self.cells[row][c] = Cell::default();
                }
                self.mark_range_dirty(0, row + 1);
            }
            2 => {
                for r in 0..self.rows {
                    for c in 0..self.cols {
                        self.cells[r][c] = Cell::default();
                    }
                }
                self.mark_all_dirty();
            }
            3 => {
                // Clear scrollback -- handled externally.
            }
            _ => {}
        }
    }

    /// Erase in line.
    ///   0 = cursor to end, 1 = start to cursor, 2 = entire line.
    pub fn erase_in_line(&mut self, mode: u16) {
        let (row, col) = (self.cursor.row, self.cursor.col);
        if row >= self.rows {
            return;
        }
        match mode {
            0 => {
                for c in col..self.cols {
                    self.cells[row][c] = Cell::default();
                }
            }
            1 => {
                for c in 0..=col.min(self.cols.saturating_sub(1)) {
                    self.cells[row][c] = Cell::default();
                }
            }
            2 => {
                for c in 0..self.cols {
                    self.cells[row][c] = Cell::default();
                }
            }
            _ => {}
        }
        self.mark_dirty(row);
    }

    /// Erase `count` characters starting at the cursor (replace with blanks).
    pub fn erase_chars(&mut self, count: usize) {
        let row = self.cursor.row;
        let col = self.cursor.col;
        if row >= self.rows {
            return;
        }
        let end = (col + count).min(self.cols);
        for c in col..end {
            self.cells[row][c] = Cell::default();
        }
        self.mark_dirty(row);
    }
}
