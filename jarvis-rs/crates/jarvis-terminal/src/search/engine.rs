//! Search engine: pattern matching, match navigation, and state management.

use regex::Regex;

use crate::grid::Grid;
use crate::scrollback::ScrollbackBuffer;

use super::types::SearchMatch;

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
                Err(_) => return, // invalid regex -- no matches
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
