//! Selection state management and text extraction.

use crate::grid::Grid;
use crate::scrollback::ScrollbackBuffer;

use super::types::{SelectionKind, SelectionPoint, SelectionRange};

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
