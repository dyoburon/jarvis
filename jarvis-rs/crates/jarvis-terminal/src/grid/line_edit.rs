//! Line and character insertion/deletion operations.

use super::core::Grid;
use super::types::Cell;

impl Grid {
    /// Insert `count` blank lines at the cursor row within the scroll region.
    ///
    /// Uses `drain` + `splice` for O(n) bulk moves instead of repeated
    /// O(n) `remove`/`insert` calls.
    pub fn insert_lines(&mut self, count: usize) {
        let row = self.cursor.row;
        if row < self.scroll_top || row > self.scroll_bottom {
            return;
        }
        let count = count.min(self.scroll_bottom - row + 1);
        // Remove `count` rows from the bottom of the scroll region.
        let drain_start = self.scroll_bottom + 1 - count;
        self.cells.drain(drain_start..drain_start + count);
        // Insert `count` blank rows at the cursor position.
        let blanks = (0..count).map(|_| Self::blank_row(self.cols));
        self.cells.splice(row..row, blanks);
        self.mark_range_dirty(row, self.scroll_bottom + 1);
    }

    /// Delete `count` lines at the cursor row within the scroll region.
    pub fn delete_lines(&mut self, count: usize) {
        let row = self.cursor.row;
        if row < self.scroll_top || row > self.scroll_bottom {
            return;
        }
        let count = count.min(self.scroll_bottom - row + 1);
        for _ in 0..count {
            self.cells.remove(row);
        }
        for _ in 0..count {
            self.cells
                .insert(self.scroll_bottom - count + 1, Self::blank_row(self.cols));
        }
        self.mark_range_dirty(row, self.scroll_bottom + 1);
    }

    /// Insert `count` blank characters at the cursor position, shifting
    /// existing chars to the right.
    pub fn insert_blank_chars(&mut self, count: usize) {
        let row = self.cursor.row;
        let col = self.cursor.col;
        if row >= self.rows || col >= self.cols {
            return;
        }
        let count = count.min(self.cols - col);
        for _ in 0..count {
            self.cells[row].pop();
        }
        for _ in 0..count {
            self.cells[row].insert(col, Cell::default());
        }
        self.cells[row].resize(self.cols, Cell::default());
        self.mark_dirty(row);
    }

    /// Delete `count` characters at the cursor position, shifting remaining
    /// chars to the left.
    pub fn delete_chars(&mut self, count: usize) {
        let row = self.cursor.row;
        let col = self.cursor.col;
        if row >= self.rows || col >= self.cols {
            return;
        }
        let count = count.min(self.cols - col);
        for _ in 0..count {
            if col < self.cells[row].len() {
                self.cells[row].remove(col);
            }
        }
        self.cells[row].resize(self.cols, Cell::default());
        self.mark_dirty(row);
    }
}
