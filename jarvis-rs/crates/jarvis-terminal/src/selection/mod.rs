//! Text selection for the terminal.
//!
//! Supports normal (character-range), line, and block (rectangular) selection
//! modes. Works across both the visible grid and the scrollback buffer.

mod logic;
mod types;

pub use logic::*;
pub use types::*;

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
