//! Dirty-row tracking for incremental rendering.

use super::core::Grid;

impl Grid {
    #[inline]
    pub(crate) fn mark_dirty(&mut self, row: usize) {
        if row < self.dirty_rows.len() {
            self.dirty_rows[row] = true;
        }
    }

    #[inline]
    pub(crate) fn mark_range_dirty(&mut self, start: usize, end: usize) {
        for r in start..end.min(self.dirty_rows.len()) {
            self.dirty_rows[r] = true;
        }
    }

    pub(crate) fn mark_all_dirty(&mut self) {
        for d in &mut self.dirty_rows {
            *d = true;
        }
    }

    /// Returns a snapshot of which rows are dirty, then clears all dirty flags.
    pub fn take_dirty(&mut self) -> Vec<bool> {
        let snapshot = self.dirty_rows.clone();
        for d in &mut self.dirty_rows {
            *d = false;
        }
        snapshot
    }

    /// Check whether any row is dirty.
    pub fn any_dirty(&self) -> bool {
        self.dirty_rows.iter().any(|&d| d)
    }
}
