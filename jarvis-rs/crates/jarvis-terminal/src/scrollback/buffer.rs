//! ScrollbackBuffer: ring buffer for terminal history lines.

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
