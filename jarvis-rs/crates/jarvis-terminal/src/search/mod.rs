//! Terminal search -- find text in the scrollback buffer and visible grid.
//!
//! Supports both literal string matching and regex-based search.

mod engine;
mod types;

pub use engine::*;
pub use types::*;

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::{Cell, CellAttributes, Grid};
    use crate::scrollback::ScrollbackBuffer;

    fn make_line(s: &str) -> Vec<Cell> {
        s.chars()
            .map(|c| Cell {
                c,
                attrs: CellAttributes::default(),
                width: 1,
            })
            .collect()
    }

    fn setup_grid_and_scrollback() -> (Grid, ScrollbackBuffer) {
        let mut grid = Grid::new(20, 3);
        // Row 0: "hello world"
        for (i, ch) in "hello world".chars().enumerate() {
            grid.cell_mut(0, i).c = ch;
        }
        // Row 1: "foo bar baz"
        for (i, ch) in "foo bar baz".chars().enumerate() {
            grid.cell_mut(1, i).c = ch;
        }
        // Row 2: "hello again"
        for (i, ch) in "hello again".chars().enumerate() {
            grid.cell_mut(2, i).c = ch;
        }

        let mut sb = ScrollbackBuffer::new(100);
        sb.push(make_line("scrollback hello line"));
        sb.push(make_line("another line"));

        (grid, sb)
    }

    #[test]
    fn search_finds_literal_matches() {
        let (grid, sb) = setup_grid_and_scrollback();
        let mut state = SearchState::new();
        state.search("hello", &grid, &sb, false);

        // Scrollback line 0 has "hello", grid row 0 has "hello", grid row 2 has "hello".
        assert_eq!(state.match_count(), 3);
    }

    #[test]
    fn search_finds_multiple_matches() {
        let (grid, sb) = setup_grid_and_scrollback();
        let mut state = SearchState::new();
        state.search("line", &grid, &sb, false);

        // "scrollback hello line" (sb 0) and "another line" (sb 1)
        assert_eq!(state.match_count(), 2);
    }

    #[test]
    fn next_match_cycles_through() {
        let (grid, sb) = setup_grid_and_scrollback();
        let mut state = SearchState::new();
        state.search("hello", &grid, &sb, false);

        // Initial current is at index 0.
        let first = state.current().unwrap().line;

        let _second = state.next_match().unwrap().line;
        let _third = state.next_match().unwrap().line;
        // Wrap around back to the first.
        let fourth = state.next_match().unwrap().line;
        assert_eq!(fourth, first);
    }

    #[test]
    fn prev_match_cycles_backwards() {
        let (grid, sb) = setup_grid_and_scrollback();
        let mut state = SearchState::new();
        state.search("hello", &grid, &sb, false);

        // Going previous from index 0 should wrap to the last match.
        let last = state.prev_match().unwrap().clone();
        assert_eq!(last.line, sb.len() + 2); // grid row 2
    }

    #[test]
    fn clear_resets() {
        let (grid, sb) = setup_grid_and_scrollback();
        let mut state = SearchState::new();
        state.search("hello", &grid, &sb, false);
        assert!(state.match_count() > 0);

        state.clear();
        assert_eq!(state.match_count(), 0);
        assert!(state.current().is_none());
    }

    #[test]
    fn regex_search_works() {
        let (grid, sb) = setup_grid_and_scrollback();
        let mut state = SearchState::new();
        state.search(r"hel+o", &grid, &sb, true);

        // Same matches as literal "hello" since hel+o matches hello.
        assert_eq!(state.match_count(), 3);
    }

    #[test]
    fn no_matches_returns_empty() {
        let (grid, sb) = setup_grid_and_scrollback();
        let mut state = SearchState::new();
        state.search("zzzzz", &grid, &sb, false);

        assert_eq!(state.match_count(), 0);
        assert!(state.current().is_none());
        assert!(state.next_match().is_none());
        assert!(state.prev_match().is_none());
    }

    #[test]
    fn is_match_at_works() {
        let (grid, sb) = setup_grid_and_scrollback();
        let mut state = SearchState::new();
        state.search("foo", &grid, &sb, false);

        // "foo" is at grid row 1, col 0 => absolute line = sb.len() + 1
        let abs_line = sb.len() + 1;
        assert!(state.is_match_at(abs_line, 0));
        assert!(state.is_match_at(abs_line, 1));
        assert!(state.is_match_at(abs_line, 2));
        assert!(!state.is_match_at(abs_line, 3));
    }
}
