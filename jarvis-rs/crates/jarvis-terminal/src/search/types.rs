//! Search types: SearchMatch.

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
