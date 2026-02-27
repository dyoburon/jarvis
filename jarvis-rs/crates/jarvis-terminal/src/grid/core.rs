//! Grid struct definition and construction helpers.

use super::types::{Cell, CellAttributes, CursorState};

// ---------------------------------------------------------------------------
// Grid
// ---------------------------------------------------------------------------

pub struct Grid {
    pub cols: usize,
    pub rows: usize,
    pub cells: Vec<Vec<Cell>>,
    pub cursor: CursorState,
    pub saved_cursor: Option<CursorState>,
    /// Current drawing attributes applied to newly written characters.
    pub attrs: CellAttributes,
    pub scroll_top: usize,
    pub scroll_bottom: usize,
    /// Per-column tab stops (true = stop present).
    pub tab_stops: Vec<bool>,
    pub origin_mode: bool,
    pub auto_wrap: bool,
    /// Delayed-wrap flag (wrap on *next* printable character).
    pub wrap_pending: bool,
    /// Saved primary screen when in alternate screen mode.
    pub alternate_screen: Option<Vec<Vec<Cell>>>,
    pub title: String,
    /// Per-row dirty flags for incremental rendering.
    pub(crate) dirty_rows: Vec<bool>,
}

impl Grid {
    pub fn new(cols: usize, rows: usize) -> Self {
        let cells = Self::blank_cells(cols, rows);
        let tab_stops = Self::default_tab_stops(cols);
        Self {
            cols,
            rows,
            cells,
            cursor: CursorState::default(),
            saved_cursor: None,
            attrs: CellAttributes::default(),
            scroll_top: 0,
            scroll_bottom: rows.saturating_sub(1),
            tab_stops,
            origin_mode: false,
            auto_wrap: true,
            wrap_pending: false,
            alternate_screen: None,
            title: String::new(),
            dirty_rows: vec![true; rows],
        }
    }

    pub(crate) fn blank_cells(cols: usize, rows: usize) -> Vec<Vec<Cell>> {
        (0..rows)
            .map(|_| (0..cols).map(|_| Cell::default()).collect())
            .collect()
    }

    pub(crate) fn blank_row(cols: usize) -> Vec<Cell> {
        (0..cols).map(|_| Cell::default()).collect()
    }

    pub(crate) fn default_tab_stops(cols: usize) -> Vec<bool> {
        (0..cols).map(|c| c % 8 == 0).collect()
    }

    // -- cell access --------------------------------------------------------

    pub fn cell(&self, row: usize, col: usize) -> &Cell {
        &self.cells[row][col]
    }

    pub fn cell_mut(&mut self, row: usize, col: usize) -> &mut Cell {
        &mut self.cells[row][col]
    }

    // -- reset --------------------------------------------------------------

    pub fn reset(&mut self) {
        *self = Self::new(self.cols, self.rows);
    }
}
