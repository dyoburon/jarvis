//! VteHandler struct: wraps Grid + vte::Parser.

use crate::grid::Grid;

/// Wraps a terminal [`Grid`] and a VTE [`vte::Parser`], driving the grid in
/// response to incoming byte streams.
///
/// Because `vte::Parser::advance` borrows the `Perform` implementor mutably,
/// we split the parser out so that `Grid` can serve as the performer directly
/// through the `GridPerformer` new-type wrapper.
pub struct VteHandler {
    grid: Grid,
    parser: vte::Parser,
}

impl VteHandler {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            grid: Grid::new(cols, rows),
            parser: vte::Parser::new(),
        }
    }

    /// Feed raw bytes from the PTY into the parser, updating the grid.
    pub fn process(&mut self, bytes: &[u8]) {
        // We need to hand the parser a &mut Perform, but the parser itself is
        // also &mut.  Because Grid is a *separate* field we can safely split
        // the borrows via a temporary wrapper.
        let grid = &mut self.grid as *mut Grid;
        // SAFETY: `parser.advance` will only call methods on the performer
        // (which accesses `grid`).  `parser` and `grid` are disjoint fields.
        let performer = unsafe { &mut *grid };
        self.parser.advance(performer, bytes);
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    /// Returns a snapshot of which rows are dirty, then clears all dirty flags.
    pub fn take_dirty(&mut self) -> Vec<bool> {
        self.grid.take_dirty()
    }
}
