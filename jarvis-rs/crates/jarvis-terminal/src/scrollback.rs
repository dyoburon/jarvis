//! Scrollback ring buffer for terminal history.
//!
//! Stores lines that have scrolled off the top of the visible grid, up to a
//! configurable maximum. Oldest lines are dropped when the capacity is exceeded.

use std::collections::VecDeque;

use crate::grid::Cell;

/// Default maximum number of scrollback lines.
const DEFAULT_MAX_LINES: usize = 10_000;

/// A buffer that stores lines that have scrolled off the visible terminal grid.
pub struct ScrollbackBuffer {
    lines: VecDeque<Vec<Cell>>,
    max_lines: usize,
}

impl ScrollbackBuffer {
    /// Create an empty scrollback buffer with the given maximum capacity.
    pub fn new(max_lines: usize) -> Self {
        ScrollbackBuffer {
            lines: VecDeque::new(),
            max_lines,
        }
    }

    /// Push a single line into the buffer, dropping the oldest line if the
    /// buffer is at capacity.
    pub fn push(&mut self, line: Vec<Cell>) {
        if self.lines.len() >= self.max_lines {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    /// Push multiple lines into the buffer, respecting capacity.
    pub fn push_many(&mut self, lines: Vec<Vec<Cell>>) {
        for line in lines {
            self.push(line);
        }
    }

    /// Number of lines currently stored.
    pub fn len(&self) -> usize {
        self.lines.len()
    }

    /// Returns `true` if the buffer contains no lines.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Retrieve a line by index (`0` = oldest).
    pub fn get(&self, index: usize) -> Option<&Vec<Cell>> {
        self.lines.get(index)
    }

    /// Convert the line at `index` to a `String` (trimming trailing spaces).
    pub fn line_to_string(&self, index: usize) -> Option<String> {
        self.lines.get(index).map(|cells| {
            let s: String = cells.iter().map(|c| c.c).collect();
            s.trim_end().to_string()
        })
    }

    /// Remove all stored lines.
    pub fn clear(&mut self) {
        self.lines.clear();
    }

    /// Iterate over all stored lines (oldest first).
    pub fn iter(&self) -> impl Iterator<Item = &Vec<Cell>> {
        self.lines.iter()
    }

    /// Search for a plain-text pattern across all stored lines.
    ///
    /// Returns a list of `(line_index, column)` pairs for every occurrence.
    pub fn search(&self, pattern: &str) -> Vec<(usize, usize)> {
        if pattern.is_empty() {
            return Vec::new();
        }

        let mut results = Vec::new();
        for (line_idx, cells) in self.lines.iter().enumerate() {
            let text: String = cells.iter().map(|c| c.c).collect();
            for (col, _) in text.match_indices(pattern) {
                results.push((line_idx, col));
            }
        }
        results
    }
}

impl Default for ScrollbackBuffer {
    fn default() -> Self {
        ScrollbackBuffer::new(DEFAULT_MAX_LINES)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::grid::CellAttributes;

    /// Helper: build a line of `Cell`s from a `&str`.
    fn make_line(s: &str) -> Vec<Cell> {
        s.chars()
            .map(|c| Cell {
                c,
                attrs: CellAttributes::default(),
                width: 1,
            })
            .collect()
    }

    #[test]
    fn push_and_retrieve() {
        let mut buf = ScrollbackBuffer::new(100);
        buf.push(make_line("hello"));
        buf.push(make_line("world"));

        assert_eq!(buf.len(), 2);
        assert_eq!(buf.line_to_string(0), Some("hello".to_string()));
        assert_eq!(buf.line_to_string(1), Some("world".to_string()));
    }

    #[test]
    fn capacity_limit_drops_oldest() {
        let mut buf = ScrollbackBuffer::new(2);
        buf.push(make_line("aaa"));
        buf.push(make_line("bbb"));
        buf.push(make_line("ccc"));

        assert_eq!(buf.len(), 2);
        // "aaa" should have been evicted.
        assert_eq!(buf.line_to_string(0), Some("bbb".to_string()));
        assert_eq!(buf.line_to_string(1), Some("ccc".to_string()));
    }

    #[test]
    fn push_many_respects_capacity() {
        let mut buf = ScrollbackBuffer::new(3);
        buf.push_many(vec![
            make_line("1"),
            make_line("2"),
            make_line("3"),
            make_line("4"),
            make_line("5"),
        ]);

        assert_eq!(buf.len(), 3);
        assert_eq!(buf.line_to_string(0), Some("3".to_string()));
        assert_eq!(buf.line_to_string(1), Some("4".to_string()));
        assert_eq!(buf.line_to_string(2), Some("5".to_string()));
    }

    #[test]
    fn line_to_string_trims_trailing_spaces() {
        let mut buf = ScrollbackBuffer::new(10);
        buf.push(make_line("hello   "));

        assert_eq!(buf.line_to_string(0), Some("hello".to_string()));
    }

    #[test]
    fn clear_removes_all() {
        let mut buf = ScrollbackBuffer::new(10);
        buf.push(make_line("one"));
        buf.push(make_line("two"));
        buf.clear();

        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn search_finds_matches() {
        let mut buf = ScrollbackBuffer::new(100);
        buf.push(make_line("hello world"));
        buf.push(make_line("foo bar"));
        buf.push(make_line("hello again"));

        let matches = buf.search("hello");
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0], (0, 0));
        assert_eq!(matches[1], (2, 0));
    }

    #[test]
    fn search_returns_empty_for_no_match() {
        let mut buf = ScrollbackBuffer::new(100);
        buf.push(make_line("hello world"));

        let matches = buf.search("xyz");
        assert!(matches.is_empty());
    }

    #[test]
    fn empty_buffer() {
        let buf = ScrollbackBuffer::new(100);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.get(0), None);
        assert_eq!(buf.line_to_string(0), None);
        assert!(buf.search("anything").is_empty());
    }
}
