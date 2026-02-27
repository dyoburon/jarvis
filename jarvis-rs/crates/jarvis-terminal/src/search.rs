//! Terminal search — find text in the scrollback buffer and visible grid.
//!
//! Supports both literal string matching and regex-based search.

use regex::Regex;

use crate::grid::Grid;
use crate::scrollback::ScrollbackBuffer;

// ---------------------------------------------------------------------------
// SearchMatch
// ---------------------------------------------------------------------------

/// A single search hit within the combined scrollback+grid content.
///
/// `line` uses the same absolute indexing as the selection system:
/// `0..scrollback.len()` = scrollback, `scrollback.len()..` = grid rows.
#[derive(Debug, Clone)]
pub struct SearchMatch {
    pub line: usize,
    pub col: usize,
    pub len: usize,
}

// ---------------------------------------------------------------------------
// SearchState
// ---------------------------------------------------------------------------

/// Manages the active search pattern, found matches, and cursor.
pub struct SearchState {
    pattern: Option<String>,
    matches: Vec<SearchMatch>,
    current_match: Option<usize>,
    use_regex: bool,
}

impl SearchState {
    /// Create a new, empty search state.
    pub fn new() -> Self {
        SearchState {
            pattern: None,
            matches: Vec::new(),
            current_match: None,
            use_regex: false,
        }
    }

    /// Run a search across the scrollback buffer and visible grid.
    ///
    /// In literal mode the pattern is matched with plain `str::match_indices`.
    /// In regex mode the `regex` crate is used (invalid patterns are silently
    /// ignored, producing zero matches).
    pub fn search(
        &mut self,
        pattern: &str,
        grid: &Grid,
        scrollback: &ScrollbackBuffer,
        use_regex: bool,
    ) {
        self.pattern = Some(pattern.to_string());
        self.use_regex = use_regex;
        self.matches.clear();
        self.current_match = None;

        if pattern.is_empty() {
            return;
        }

        let compiled_regex = if use_regex {
            match Regex::new(pattern) {
                Ok(re) => Some(re),
                Err(_) => return, // invalid regex — no matches
            }
        } else {
            None
        };

        // Search scrollback lines first (absolute line 0..scrollback.len()).
        for line_idx in 0..scrollback.len() {
            let text = scrollback.line_to_string(line_idx).unwrap_or_default();
            self.find_in_line(line_idx, &text, &compiled_regex, pattern);
        }

        // Then search visible grid rows.
        let sb_len = scrollback.len();
        for row in 0..grid.rows {
            let text = grid.row_to_string(row);
            self.find_in_line(sb_len + row, &text, &compiled_regex, pattern);
        }

        if !self.matches.is_empty() {
            self.current_match = Some(0);
        }
    }

    /// Advance to the next match, wrapping around at the end.
    pub fn next_match(&mut self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }
        let idx = match self.current_match {
            Some(i) => (i + 1) % self.matches.len(),
            None => 0,
        };
        self.current_match = Some(idx);
        Some(&self.matches[idx])
    }

    /// Go to the previous match, wrapping around at the beginning.
    pub fn prev_match(&mut self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }
        let idx = match self.current_match {
            Some(0) => self.matches.len() - 1,
            Some(i) => i - 1,
            None => self.matches.len() - 1,
        };
        self.current_match = Some(idx);
        Some(&self.matches[idx])
    }

    /// Return the current match (if any).
    pub fn current(&self) -> Option<&SearchMatch> {
        self.current_match.map(|i| &self.matches[i])
    }

    /// Total number of matches found.
    pub fn match_count(&self) -> usize {
        self.matches.len()
    }

    /// Reset the search state entirely.
    pub fn clear(&mut self) {
        self.pattern = None;
        self.matches.clear();
        self.current_match = None;
    }

    /// Returns `true` if the character at (`line`, `col`) falls within any
    /// search match. Useful for highlighting.
    pub fn is_match_at(&self, line: usize, col: usize) -> bool {
        self.matches
            .iter()
            .any(|m| m.line == line && col >= m.col && col < m.col + m.len)
    }

    // -----------------------------------------------------------------------
    // Internal
    // -----------------------------------------------------------------------

    fn find_in_line(
        &mut self,
        line_idx: usize,
        text: &str,
        compiled_regex: &Option<Regex>,
        pattern: &str,
    ) {
        if let Some(re) = compiled_regex {
            for mat in re.find_iter(text) {
                if !mat.is_empty() {
                    self.matches.push(SearchMatch {
                        line: line_idx,
                        col: mat.start(),
                        len: mat.len(),
                    });
                }
            }
        } else {
            for (col, _) in text.match_indices(pattern) {
                self.matches.push(SearchMatch {
                    line: line_idx,
                    col,
                    len: pattern.len(),
                });
            }
        }
    }
}

impl Default for SearchState {
    fn default() -> Self {
        SearchState::new()
    }
}

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
