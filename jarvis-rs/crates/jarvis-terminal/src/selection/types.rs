//! Selection types: SelectionPoint, SelectionRange, SelectionKind.

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
