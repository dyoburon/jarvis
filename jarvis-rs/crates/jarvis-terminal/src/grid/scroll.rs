//! Scrolling, scroll region, and alternate screen management.

use super::core::Grid;
use super::types::Cell;
use super::types::CursorState;

impl Grid {
    /// Scroll the scroll-region up by `count` lines. Returns lines scrolled
    /// off the top of the region.
    ///
    /// Uses `drain` + `splice` for O(n) bulk moves instead of repeated
    /// O(n) `remove`/`insert` calls (which would be O(count * n)).
    pub fn scroll_up(&mut self, count: usize) -> Vec<Vec<Cell>> {
        let top = self.scroll_top;
        let bot = self.scroll_bottom;
        if top > bot || count == 0 {
            return Vec::new();
        }
        let count = count.min(bot - top + 1);
        // Drain the top `count` rows from the scroll region in one shot.
        let scrolled: Vec<Vec<Cell>> = self.cells.drain(top..top + count).collect();
        // Insert `count` blank rows at the bottom of the (now-shorter) region.
        let insert_at = bot - count + 1; // bot shifted down by `count` after drain
        let blanks = (0..count).map(|_| Self::blank_row(self.cols));
        self.cells.splice(insert_at..insert_at, blanks);
        self.mark_range_dirty(top, bot + 1);
        scrolled
    }

    /// Scroll the scroll-region down by `count` lines.
    ///
    /// Uses `drain` + `splice` for O(n) bulk moves instead of repeated
    /// O(n) `remove`/`insert` calls.
    pub fn scroll_down(&mut self, count: usize) {
        let top = self.scroll_top;
        let bot = self.scroll_bottom;
        if top > bot || count == 0 {
            return;
        }
        let count = count.min(bot - top + 1);
        // Remove `count` rows from the bottom of the scroll region.
        let drain_start = bot + 1 - count;
        self.cells.drain(drain_start..drain_start + count);
        // Insert `count` blank rows at the top of the region.
        let blanks = (0..count).map(|_| Self::blank_row(self.cols));
        self.cells.splice(top..top, blanks);
        self.mark_range_dirty(top, bot + 1);
    }

    // -- scroll region ------------------------------------------------------

    pub fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        if top < bottom && bottom < self.rows {
            self.scroll_top = top;
            self.scroll_bottom = bottom;
        }
    }

    // -- alternate screen (smcup / rmcup) -----------------------------------

    pub fn enter_alternate_screen(&mut self) {
        if self.alternate_screen.is_some() {
            return; // already in alternate
        }
        let saved = self.cells.clone();
        self.alternate_screen = Some(saved);
        self.cells = Self::blank_cells(self.cols, self.rows);
        self.cursor = CursorState::default();
        self.mark_all_dirty();
    }

    pub fn exit_alternate_screen(&mut self) {
        if let Some(saved) = self.alternate_screen.take() {
            self.cells = saved;
            self.cursor.row = self.cursor.row.min(self.rows.saturating_sub(1));
            self.cursor.col = self.cursor.col.min(self.cols.saturating_sub(1));
            self.mark_all_dirty();
        }
    }

    // -- resize -------------------------------------------------------------

    /// Resize the grid, preserving content where possible.
    /// Returns lines that scrolled off the top (for scrollback).
    pub fn resize(&mut self, new_cols: usize, new_rows: usize) -> Vec<Vec<Cell>> {
        let mut scrolled_off = Vec::new();

        // Adjust columns in every existing row.
        for row in &mut self.cells {
            row.resize(new_cols, Cell::default());
        }

        if new_rows < self.rows {
            // Shrink: if cursor is below new bottom, scroll lines off the top.
            let excess = self.cells.len().saturating_sub(new_rows);
            if excess > 0 {
                scrolled_off = self.cells.drain(..excess).collect();
                // Adjust cursor row.
                self.cursor.row = self.cursor.row.saturating_sub(excess);
            }
        } else if new_rows > self.rows {
            // Grow: add blank lines at the bottom.
            let extra = new_rows - self.cells.len();
            for _ in 0..extra {
                self.cells.push(Self::blank_row(new_cols));
            }
        }

        self.cols = new_cols;
        self.rows = new_rows;
        self.scroll_top = 0;
        self.scroll_bottom = new_rows.saturating_sub(1);
        self.tab_stops = Self::default_tab_stops(new_cols);

        // Clamp cursor.
        self.cursor.row = self.cursor.row.min(new_rows.saturating_sub(1));
        self.cursor.col = self.cursor.col.min(new_cols.saturating_sub(1));

        self.dirty_rows = vec![true; new_rows];

        scrolled_off
    }
}
