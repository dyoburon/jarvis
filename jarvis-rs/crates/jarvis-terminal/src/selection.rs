//! Text selection for the terminal.
//!
//! Supports normal (character-range), line, and block (rectangular) selection
//! modes. Works across both the visible grid and the scrollback buffer.

use crate::grid::Grid;
use crate::scrollback::ScrollbackBuffer;

// ---------------------------------------------------------------------------
// SelectionPoint / SelectionRange
// ---------------------------------------------------------------------------

/// A single point in the terminal (row + column).
///
/// Row indices are *absolute*: `0..scrollback.len()` covers scrollback lines,
/// and `scrollback.len()..scrollback.len()+grid.rows` covers the visible grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct SelectionPoint {
    pub row: usize,
    pub col: usize,
}

/// An ordered range with `start <= end`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectionRange {
    pub start: SelectionPoint,
    pub end: SelectionPoint,
}

// ---------------------------------------------------------------------------
// SelectionKind
// ---------------------------------------------------------------------------

/// The kind of text selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionKind {
    /// Character-level selection from start to end.
    #[default]
    Normal,
    /// Whole-line selection.
    Line,
    /// Rectangular / block (column) selection.
    Block,
}

// ---------------------------------------------------------------------------
// Selection
// ---------------------------------------------------------------------------

/// Tracks the current text selection state.
pub struct Selection {
    /// Where the selection was initiated (anchor point).
    anchor: Option<SelectionPoint>,
    /// The current cursor/extension point of the selection.
    active: Option<SelectionPoint>,
    /// What kind of selection this is.
    kind: SelectionKind,
}

impl Selection {
    /// Create a new, empty selection.
    pub fn new() -> Self {
        Selection {
            anchor: None,
            active: None,
            kind: SelectionKind::Normal,
        }
    }

    /// Begin a selection at `point` with the given `kind`.
    pub fn start(&mut self, point: SelectionPoint, kind: SelectionKind) {
        self.anchor = Some(point);
        self.active = Some(point);
        self.kind = kind;
    }

    /// Extend the selection to `point`.
    pub fn update(&mut self, point: SelectionPoint) {
        if self.anchor.is_some() {
            self.active = Some(point);
        }
    }

    /// Finalize the selection (currently a no-op; reserved for future use).
    pub fn finish(&mut self) {
        // intentionally empty
    }

    /// Remove the selection entirely.
    pub fn clear(&mut self) {
        self.anchor = None;
        self.active = None;
    }

    /// Returns `true` if there is an active (non-empty) selection.
    pub fn is_active(&self) -> bool {
        self.anchor.is_some() && self.active.is_some()
    }

    /// Return the normalized range (start <= end), or `None` if no selection.
    pub fn range(&self) -> Option<SelectionRange> {
        match (self.anchor, self.active) {
            (Some(a), Some(b)) => {
                let (start, end) = if a <= b { (a, b) } else { (b, a) };
                Some(SelectionRange { start, end })
            }
            _ => None,
        }
    }

    /// Returns `true` if the cell at (`row`, `col`) falls within the current
    /// selection.
    pub fn contains(&self, row: usize, col: usize) -> bool {
        let range = match self.range() {
            Some(r) => r,
            None => return false,
        };

        match self.kind {
            SelectionKind::Normal => {
                let point = SelectionPoint { row, col };
                point >= range.start && point <= range.end
            }
            SelectionKind::Line => row >= range.start.row && row <= range.end.row,
            SelectionKind::Block => {
                let min_col = range.start.col.min(range.end.col);
                let max_col = range.start.col.max(range.end.col);
                // For block selection, use the original anchor/active columns
                // for proper rectangle semantics.
                let (min_col, max_col) = match (self.anchor, self.active) {
                    (Some(a), Some(b)) => (a.col.min(b.col), a.col.max(b.col)),
                    _ => (min_col, max_col),
                };
                row >= range.start.row && row <= range.end.row && col >= min_col && col <= max_col
            }
        }
    }

    /// Extract the selected text from the grid and scrollback buffer.
    pub fn selected_text(&self, grid: &Grid, scrollback: &ScrollbackBuffer) -> String {
        let range = match self.range() {
            Some(r) => r,
            None => return String::new(),
        };

        let sb_len = scrollback.len();
        let mut result = String::new();

        match self.kind {
            SelectionKind::Normal => {
                for row in range.start.row..=range.end.row {
                    let line_text = self.row_text(row, grid, scrollback, sb_len);
                    let start_col = if row == range.start.row {
                        range.start.col
                    } else {
                        0
                    };
                    let end_col = if row == range.end.row {
                        range.end.col
                    } else {
                        line_text.len().saturating_sub(1)
                    };

                    if start_col < line_text.len() {
                        let end = (end_col + 1).min(line_text.len());
                        let chars: Vec<char> = line_text.chars().collect();
                        let slice_end = end.min(chars.len());
                        let slice_start = start_col.min(slice_end);
                        let segment: String = chars[slice_start..slice_end].iter().collect();
                        result.push_str(&segment);
                    }

                    if row != range.end.row {
                        result.push('\n');
                    }
                }
            }
            SelectionKind::Line => {
                for row in range.start.row..=range.end.row {
                    let line_text = self.row_text(row, grid, scrollback, sb_len);
                    result.push_str(&line_text);
                    if row != range.end.row {
                        result.push('\n');
                    }
                }
            }
            SelectionKind::Block => {
                let (min_col, max_col) = match (self.anchor, self.active) {
                    (Some(a), Some(b)) => (a.col.min(b.col), a.col.max(b.col)),
                    _ => (range.start.col, range.end.col),
                };

                for row in range.start.row..=range.end.row {
                    let line_text = self.row_text(row, grid, scrollback, sb_len);
                    let chars: Vec<char> = line_text.chars().collect();
                    let start = min_col.min(chars.len());
                    let end = (max_col + 1).min(chars.len());
                    let segment: String = chars[start..end].iter().collect();
                    result.push_str(&segment);
                    if row != range.end.row {
                        result.push('\n');
                    }
                }
            }
        }

        result
    }

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Return the full text of a row (from either scrollback or the grid).
    fn row_text(
        &self,
        row: usize,
        grid: &Grid,
        scrollback: &ScrollbackBuffer,
        sb_len: usize,
    ) -> String {
        if row < sb_len {
            // Scrollback line
            scrollback.line_to_string(row).unwrap_or_default()
        } else {
            // Grid line
            let grid_row = row - sb_len;
            if grid_row < grid.rows {
                grid.row_to_string(grid_row)
            } else {
                String::new()
            }
        }
    }
}

impl Default for Selection {
    fn default() -> Self {
        Selection::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::Grid;
    use crate::scrollback::ScrollbackBuffer;

    #[test]
    fn no_selection_initially() {
        let sel = Selection::new();
        assert!(!sel.is_active());
        assert!(sel.range().is_none());
    }

    #[test]
    fn start_and_update_creates_range() {
        let mut sel = Selection::new();
        sel.start(SelectionPoint { row: 0, col: 2 }, SelectionKind::Normal);
        sel.update(SelectionPoint { row: 1, col: 5 });

        assert!(sel.is_active());
        let range = sel.range().unwrap();
        assert_eq!(range.start, SelectionPoint { row: 0, col: 2 });
        assert_eq!(range.end, SelectionPoint { row: 1, col: 5 });
    }

    #[test]
    fn range_is_normalized() {
        let mut sel = Selection::new();
        // Start at a later point, then update to an earlier point.
        sel.start(SelectionPoint { row: 3, col: 10 }, SelectionKind::Normal);
        sel.update(SelectionPoint { row: 1, col: 2 });

        let range = sel.range().unwrap();
        assert!(range.start <= range.end);
        assert_eq!(range.start, SelectionPoint { row: 1, col: 2 });
        assert_eq!(range.end, SelectionPoint { row: 3, col: 10 });
    }

    #[test]
    fn clear_removes_selection() {
        let mut sel = Selection::new();
        sel.start(SelectionPoint { row: 0, col: 0 }, SelectionKind::Normal);
        sel.update(SelectionPoint { row: 1, col: 5 });
        assert!(sel.is_active());

        sel.clear();
        assert!(!sel.is_active());
        assert!(sel.range().is_none());
    }

    #[test]
    fn contains_returns_correct_cells() {
        let mut sel = Selection::new();
        sel.start(SelectionPoint { row: 1, col: 2 }, SelectionKind::Normal);
        sel.update(SelectionPoint { row: 1, col: 5 });

        // Within range
        assert!(sel.contains(1, 2));
        assert!(sel.contains(1, 3));
        assert!(sel.contains(1, 5));

        // Outside range
        assert!(!sel.contains(1, 1));
        assert!(!sel.contains(1, 6));
        assert!(!sel.contains(0, 3));
        assert!(!sel.contains(2, 3));
    }

    #[test]
    fn line_selection_includes_full_rows() {
        let mut sel = Selection::new();
        sel.start(SelectionPoint { row: 1, col: 5 }, SelectionKind::Line);
        sel.update(SelectionPoint { row: 2, col: 3 });

        // Any column in rows 1 and 2 should be selected.
        assert!(sel.contains(1, 0));
        assert!(sel.contains(1, 100));
        assert!(sel.contains(2, 0));
        assert!(sel.contains(2, 100));

        // Row 0 and 3 should not be selected.
        assert!(!sel.contains(0, 5));
        assert!(!sel.contains(3, 0));
    }

    #[test]
    fn selected_text_normal() {
        let mut grid = Grid::new(10, 3);
        // Write "hello" on row 0, "world" on row 1
        for (i, ch) in "hello     ".chars().enumerate() {
            let cell = grid.cell_mut(0, i);
            cell.c = ch;
        }
        for (i, ch) in "world     ".chars().enumerate() {
            let cell = grid.cell_mut(1, i);
            cell.c = ch;
        }

        let scrollback = ScrollbackBuffer::new(100);

        let mut sel = Selection::new();
        sel.start(SelectionPoint { row: 0, col: 0 }, SelectionKind::Normal);
        sel.update(SelectionPoint { row: 0, col: 4 });

        let text = sel.selected_text(&grid, &scrollback);
        assert_eq!(text, "hello");
    }

    #[test]
    fn selected_text_line() {
        let mut grid = Grid::new(10, 3);
        for (i, ch) in "aaaaaaaaaa".chars().enumerate() {
            let cell = grid.cell_mut(0, i);
            cell.c = ch;
        }
        for (i, ch) in "bbbbbbbbbb".chars().enumerate() {
            let cell = grid.cell_mut(1, i);
            cell.c = ch;
        }

        let scrollback = ScrollbackBuffer::new(100);

        let mut sel = Selection::new();
        sel.start(SelectionPoint { row: 0, col: 3 }, SelectionKind::Line);
        sel.update(SelectionPoint { row: 1, col: 1 });

        let text = sel.selected_text(&grid, &scrollback);
        assert_eq!(text, "aaaaaaaaaa\nbbbbbbbbbb");
    }
}
