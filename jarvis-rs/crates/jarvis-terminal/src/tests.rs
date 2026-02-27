//! Integration tests for alacritty_terminal via jarvis adapters.
//!
//! @module terminal/tests

#[cfg(test)]
mod tests {
    use alacritty_terminal::event::VoidListener;
    use alacritty_terminal::grid::Dimensions;
    use alacritty_terminal::term::{Config, Term};
    use alacritty_terminal::vte::ansi;

    use crate::size::SizeInfo;

    /// Helper: create a Term with VoidListener for testing.
    fn test_term(cols: usize, lines: usize) -> Term<VoidListener> {
        let size = SizeInfo::new(cols, lines, 10.0, 20.0);
        Term::new(Config::default(), &size, VoidListener)
    }

    /// Helper: feed raw bytes through the ansi Processor into the term.
    fn feed_bytes(term: &mut Term<VoidListener>, bytes: &[u8]) {
        let mut processor: ansi::Processor = ansi::Processor::new();
        processor.advance(term, bytes);
    }

    #[test]
    fn vte_hello_writes_cells() {
        let mut term = test_term(80, 24);
        feed_bytes(&mut term, b"Hello");

        let grid = term.grid();
        assert_eq!(
            grid[alacritty_terminal::index::Line(0)][alacritty_terminal::index::Column(0)].c,
            'H'
        );
        assert_eq!(
            grid[alacritty_terminal::index::Line(0)][alacritty_terminal::index::Column(1)].c,
            'e'
        );
        assert_eq!(
            grid[alacritty_terminal::index::Line(0)][alacritty_terminal::index::Column(2)].c,
            'l'
        );
        assert_eq!(
            grid[alacritty_terminal::index::Line(0)][alacritty_terminal::index::Column(3)].c,
            'l'
        );
        assert_eq!(
            grid[alacritty_terminal::index::Line(0)][alacritty_terminal::index::Column(4)].c,
            'o'
        );
    }

    #[test]
    fn vte_newline_moves_cursor() {
        let mut term = test_term(80, 24);
        feed_bytes(&mut term, b"AB\r\nCD");

        let grid = term.grid();
        // Line 0: "AB"
        assert_eq!(
            grid[alacritty_terminal::index::Line(0)][alacritty_terminal::index::Column(0)].c,
            'A'
        );
        assert_eq!(
            grid[alacritty_terminal::index::Line(0)][alacritty_terminal::index::Column(1)].c,
            'B'
        );
        // Line 1: "CD"
        assert_eq!(
            grid[alacritty_terminal::index::Line(1)][alacritty_terminal::index::Column(0)].c,
            'C'
        );
        assert_eq!(
            grid[alacritty_terminal::index::Line(1)][alacritty_terminal::index::Column(1)].c,
            'D'
        );
    }

    #[test]
    fn vte_term_dimensions_match() {
        let term = test_term(132, 50);

        assert_eq!(term.columns(), 132);
        assert_eq!(term.screen_lines(), 50);
    }

    #[test]
    fn vte_cursor_position_after_write() {
        let mut term = test_term(80, 24);
        feed_bytes(&mut term, b"Hello\r\n");

        let cursor = term.grid().cursor.point;
        // After "Hello\r\n": cursor at line 1, column 0
        assert_eq!(cursor.line.0, 1);
        assert_eq!(cursor.column.0, 0);
    }

    #[test]
    fn vte_clear_screen_resets_cells() {
        let mut term = test_term(80, 24);
        feed_bytes(&mut term, b"ABCDEF");
        // ESC[2J = clear entire screen
        feed_bytes(&mut term, b"\x1b[2J");

        let grid = term.grid();
        // After clear, cells should be default (space)
        assert_eq!(
            grid[alacritty_terminal::index::Line(0)][alacritty_terminal::index::Column(0)].c,
            ' '
        );
    }
}
