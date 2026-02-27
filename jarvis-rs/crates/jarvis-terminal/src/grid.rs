use serde::{Deserialize, Serialize};
use unicode_width::UnicodeWidthChar;

// ---------------------------------------------------------------------------
// TerminalColor
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, Default)]
pub enum TerminalColor {
    #[default]
    Default,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

// ---------------------------------------------------------------------------
// CellAttributes
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize, Deserialize)]
pub struct CellAttributes {
    pub fg: TerminalColor,
    pub bg: TerminalColor,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub dim: bool,
    pub inverse: bool,
    pub hidden: bool,
    pub blink: bool,
}

// ---------------------------------------------------------------------------
// Cell
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub c: char,
    pub attrs: CellAttributes,
    /// 1 = normal, 2 = wide CJK, 0 = continuation of a wide char.
    pub width: u8,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            c: ' ',
            attrs: CellAttributes::default(),
            width: 1,
        }
    }
}

// ---------------------------------------------------------------------------
// Cursor
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum CursorShape {
    #[default]
    Block,
    Underline,
    Bar,
}

#[derive(Clone, Debug)]
pub struct CursorState {
    pub row: usize,
    pub col: usize,
    pub visible: bool,
    pub shape: CursorShape,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            row: 0,
            col: 0,
            visible: true,
            shape: CursorShape::default(),
        }
    }
}

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
    dirty_rows: Vec<bool>,
}

impl Grid {
    // -- construction -------------------------------------------------------

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

    fn blank_cells(cols: usize, rows: usize) -> Vec<Vec<Cell>> {
        (0..rows)
            .map(|_| (0..cols).map(|_| Cell::default()).collect())
            .collect()
    }

    fn blank_row(cols: usize) -> Vec<Cell> {
        (0..cols).map(|_| Cell::default()).collect()
    }

    fn default_tab_stops(cols: usize) -> Vec<bool> {
        (0..cols).map(|c| c % 8 == 0).collect()
    }

    // -- dirty tracking -----------------------------------------------------

    #[inline]
    fn mark_dirty(&mut self, row: usize) {
        if row < self.dirty_rows.len() {
            self.dirty_rows[row] = true;
        }
    }

    #[inline]
    fn mark_range_dirty(&mut self, start: usize, end: usize) {
        for r in start..end.min(self.dirty_rows.len()) {
            self.dirty_rows[r] = true;
        }
    }

    fn mark_all_dirty(&mut self) {
        for d in &mut self.dirty_rows {
            *d = true;
        }
    }

    /// Returns a snapshot of which rows are dirty, then clears all dirty flags.
    pub fn take_dirty(&mut self) -> Vec<bool> {
        let snapshot = self.dirty_rows.clone();
        for d in &mut self.dirty_rows {
            *d = false;
        }
        snapshot
    }

    /// Check whether any row is dirty.
    pub fn any_dirty(&self) -> bool {
        self.dirty_rows.iter().any(|&d| d)
    }

    // -- resize -------------------------------------------------------------

    /// Resize the grid, preserving content where possible.
    /// Returns lines that scrolled off the top (for scrollback).
    pub fn resize(&mut self, new_cols: usize, new_rows: usize) -> Vec<Vec<Cell>> {
        let mut scrolled_off = Vec::new();

        // Adjust columns in every existing row.
        for row in &mut self.cells {
            row.resize(new_cols, Cell::default());
        }

        if new_rows < self.rows {
            // Shrink: if cursor is below new bottom, scroll lines off the top.
            let excess = self.cells.len().saturating_sub(new_rows);
            if excess > 0 {
                scrolled_off = self.cells.drain(..excess).collect();
                // Adjust cursor row.
                self.cursor.row = self.cursor.row.saturating_sub(excess);
            }
        } else if new_rows > self.rows {
            // Grow: add blank lines at the bottom.
            let extra = new_rows - self.cells.len();
            for _ in 0..extra {
                self.cells.push(Self::blank_row(new_cols));
            }
        }

        self.cols = new_cols;
        self.rows = new_rows;
        self.scroll_top = 0;
        self.scroll_bottom = new_rows.saturating_sub(1);
        self.tab_stops = Self::default_tab_stops(new_cols);

        // Clamp cursor.
        self.cursor.row = self.cursor.row.min(new_rows.saturating_sub(1));
        self.cursor.col = self.cursor.col.min(new_cols.saturating_sub(1));

        self.dirty_rows = vec![true; new_rows];

        scrolled_off
    }

    // -- cell access --------------------------------------------------------

    pub fn cell(&self, row: usize, col: usize) -> &Cell {
        &self.cells[row][col]
    }

    pub fn cell_mut(&mut self, row: usize, col: usize) -> &mut Cell {
        &mut self.cells[row][col]
    }

    // -- character output ---------------------------------------------------

    /// Write a character at the cursor position using current attrs, then
    /// advance the cursor.  Handles wide characters and auto-wrap.
    pub fn put_char(&mut self, c: char) {
        let char_width = c.width().unwrap_or(0) as u8;
        let display_width = if char_width == 0 { 1 } else { char_width };

        // Handle delayed wrap.
        if self.wrap_pending {
            self.wrap_pending = false;
            self.cursor.col = 0;
            self.newline();
        }

        // If the character is wide and we are at the last column, wrap first.
        if display_width == 2 && self.cursor.col + 1 >= self.cols && self.auto_wrap {
            // Fill current position with space, then wrap.
            let row = self.cursor.row;
            let col = self.cursor.col;
            self.cells[row][col] = Cell {
                c: ' ',
                attrs: self.attrs,
                width: 1,
            };
            self.mark_dirty(row);
            self.cursor.col = 0;
            self.newline();
        }

        let row = self.cursor.row;
        let col = self.cursor.col;

        if row < self.rows && col < self.cols {
            self.cells[row][col] = Cell {
                c,
                attrs: self.attrs,
                width: display_width,
            };

            // For wide chars, place a zero-width continuation cell.
            if display_width == 2 && col + 1 < self.cols {
                self.cells[row][col + 1] = Cell {
                    c: ' ',
                    attrs: self.attrs,
                    width: 0,
                };
            }

            self.mark_dirty(row);
        }

        // Advance cursor.
        let new_col = col + display_width as usize;
        if new_col >= self.cols {
            if self.auto_wrap {
                // Stay at last column and set wrap_pending.
                self.cursor.col = self.cols - 1;
                self.wrap_pending = true;
            }
            // If no auto_wrap, cursor stays at last column.
        } else {
            self.cursor.col = new_col;
        }
    }

    // -- control characters -------------------------------------------------

    /// Line feed: move cursor down one line, scrolling if at the bottom of
    /// the scroll region.
    pub fn newline(&mut self) {
        if self.cursor.row == self.scroll_bottom {
            self.scroll_up(1);
        } else if self.cursor.row + 1 < self.rows {
            self.cursor.row += 1;
        }
    }

    pub fn carriage_return(&mut self) {
        self.cursor.col = 0;
        self.wrap_pending = false;
    }

    pub fn backspace(&mut self) {
        if self.cursor.col > 0 {
            self.cursor.col -= 1;
        }
        self.wrap_pending = false;
    }

    /// Advance to the next tab stop.
    pub fn tab(&mut self) {
        let mut col = self.cursor.col + 1;
        while col < self.cols {
            if self.tab_stops.get(col).copied().unwrap_or(false) {
                break;
            }
            col += 1;
        }
        self.cursor.col = col.min(self.cols - 1);
        self.wrap_pending = false;
    }

    // -- scrolling ----------------------------------------------------------

    /// Scroll the scroll-region up by `count` lines. Returns lines scrolled
    /// off the top of the region.
    pub fn scroll_up(&mut self, count: usize) -> Vec<Vec<Cell>> {
        let top = self.scroll_top;
        let bot = self.scroll_bottom;
        if top > bot || count == 0 {
            return Vec::new();
        }
        let count = count.min(bot - top + 1);
        let mut scrolled: Vec<Vec<Cell>> = Vec::with_capacity(count);
        for _ in 0..count {
            let row = self.cells.remove(top);
            scrolled.push(row);
        }
        for _ in 0..count {
            self.cells
                .insert(bot - count + 1, Self::blank_row(self.cols));
        }
        self.mark_range_dirty(top, bot + 1);
        scrolled
    }

    /// Scroll the scroll-region down by `count` lines.
    pub fn scroll_down(&mut self, count: usize) {
        let top = self.scroll_top;
        let bot = self.scroll_bottom;
        if top > bot || count == 0 {
            return;
        }
        let count = count.min(bot - top + 1);
        for _ in 0..count {
            self.cells.remove(bot);
        }
        for _ in 0..count {
            self.cells.insert(top, Self::blank_row(self.cols));
        }
        self.mark_range_dirty(top, bot + 1);
    }

    // -- erasing ------------------------------------------------------------

    /// Erase in display.
    ///   0 = cursor to end, 1 = start to cursor, 2 = entire screen,
    ///   3 = scrollback (no-op here).
    pub fn erase_in_display(&mut self, mode: u16) {
        let (row, col) = (self.cursor.row, self.cursor.col);
        match mode {
            0 => {
                for c in col..self.cols {
                    self.cells[row][c] = Cell::default();
                }
                for r in (row + 1)..self.rows {
                    for c in 0..self.cols {
                        self.cells[r][c] = Cell::default();
                    }
                }
                self.mark_range_dirty(row, self.rows);
            }
            1 => {
                for r in 0..row {
                    for c in 0..self.cols {
                        self.cells[r][c] = Cell::default();
                    }
                }
                for c in 0..=col.min(self.cols.saturating_sub(1)) {
                    self.cells[row][c] = Cell::default();
                }
                self.mark_range_dirty(0, row + 1);
            }
            2 => {
                for r in 0..self.rows {
                    for c in 0..self.cols {
                        self.cells[r][c] = Cell::default();
                    }
                }
                self.mark_all_dirty();
            }
            3 => {
                // Clear scrollback -- handled externally.
            }
            _ => {}
        }
    }

    /// Erase in line.
    ///   0 = cursor to end, 1 = start to cursor, 2 = entire line.
    pub fn erase_in_line(&mut self, mode: u16) {
        let (row, col) = (self.cursor.row, self.cursor.col);
        if row >= self.rows {
            return;
        }
        match mode {
            0 => {
                for c in col..self.cols {
                    self.cells[row][c] = Cell::default();
                }
            }
            1 => {
                for c in 0..=col.min(self.cols.saturating_sub(1)) {
                    self.cells[row][c] = Cell::default();
                }
            }
            2 => {
                for c in 0..self.cols {
                    self.cells[row][c] = Cell::default();
                }
            }
            _ => {}
        }
        self.mark_dirty(row);
    }

    // -- line insertion / deletion ------------------------------------------

    /// Insert `count` blank lines at the cursor row within the scroll region.
    pub fn insert_lines(&mut self, count: usize) {
        let row = self.cursor.row;
        if row < self.scroll_top || row > self.scroll_bottom {
            return;
        }
        let count = count.min(self.scroll_bottom - row + 1);
        for _ in 0..count {
            self.cells.remove(self.scroll_bottom);
        }
        for _ in 0..count {
            self.cells.insert(row, Self::blank_row(self.cols));
        }
        self.mark_range_dirty(row, self.scroll_bottom + 1);
    }

    /// Delete `count` lines at the cursor row within the scroll region.
    pub fn delete_lines(&mut self, count: usize) {
        let row = self.cursor.row;
        if row < self.scroll_top || row > self.scroll_bottom {
            return;
        }
        let count = count.min(self.scroll_bottom - row + 1);
        for _ in 0..count {
            self.cells.remove(row);
        }
        for _ in 0..count {
            self.cells
                .insert(self.scroll_bottom - count + 1, Self::blank_row(self.cols));
        }
        self.mark_range_dirty(row, self.scroll_bottom + 1);
    }

    // -- char insertion / deletion ------------------------------------------

    /// Insert `count` blank characters at the cursor position, shifting
    /// existing chars to the right.
    pub fn insert_blank_chars(&mut self, count: usize) {
        let row = self.cursor.row;
        let col = self.cursor.col;
        if row >= self.rows || col >= self.cols {
            return;
        }
        let count = count.min(self.cols - col);
        for _ in 0..count {
            self.cells[row].pop();
        }
        for _ in 0..count {
            self.cells[row].insert(col, Cell::default());
        }
        self.cells[row].resize(self.cols, Cell::default());
        self.mark_dirty(row);
    }

    /// Delete `count` characters at the cursor position, shifting remaining
    /// chars to the left.
    pub fn delete_chars(&mut self, count: usize) {
        let row = self.cursor.row;
        let col = self.cursor.col;
        if row >= self.rows || col >= self.cols {
            return;
        }
        let count = count.min(self.cols - col);
        for _ in 0..count {
            if col < self.cells[row].len() {
                self.cells[row].remove(col);
            }
        }
        self.cells[row].resize(self.cols, Cell::default());
        self.mark_dirty(row);
    }

    /// Erase `count` characters starting at the cursor (replace with blanks).
    pub fn erase_chars(&mut self, count: usize) {
        let row = self.cursor.row;
        let col = self.cursor.col;
        if row >= self.rows {
            return;
        }
        let end = (col + count).min(self.cols);
        for c in col..end {
            self.cells[row][c] = Cell::default();
        }
        self.mark_dirty(row);
    }

    // -- cursor movement ----------------------------------------------------

    /// Move cursor to an absolute position, clamped to grid bounds.
    pub fn move_cursor(&mut self, row: usize, col: usize) {
        let old_row = self.cursor.row;
        self.cursor.row = row.min(self.rows.saturating_sub(1));
        self.cursor.col = col.min(self.cols.saturating_sub(1));
        self.wrap_pending = false;
        self.mark_dirty(old_row);
        self.mark_dirty(self.cursor.row);
    }

    /// Move cursor relative to current position.
    pub fn move_cursor_relative(&mut self, d_row: i32, d_col: i32) {
        let old_row = self.cursor.row;
        let new_row = (self.cursor.row as i32 + d_row)
            .max(0)
            .min(self.rows.saturating_sub(1) as i32) as usize;
        let new_col = (self.cursor.col as i32 + d_col)
            .max(0)
            .min(self.cols.saturating_sub(1) as i32) as usize;
        self.cursor.row = new_row;
        self.cursor.col = new_col;
        self.wrap_pending = false;
        self.mark_dirty(old_row);
        self.mark_dirty(new_row);
    }

    // -- cursor save / restore (DECSC / DECRC) ------------------------------

    pub fn save_cursor(&mut self) {
        self.saved_cursor = Some(self.cursor.clone());
    }

    pub fn restore_cursor(&mut self) {
        if let Some(saved) = self.saved_cursor.take() {
            self.cursor = saved;
            // Clamp to current dimensions.
            self.cursor.row = self.cursor.row.min(self.rows.saturating_sub(1));
            self.cursor.col = self.cursor.col.min(self.cols.saturating_sub(1));
        }
        self.wrap_pending = false;
    }

    // -- scroll region ------------------------------------------------------

    pub fn set_scroll_region(&mut self, top: usize, bottom: usize) {
        if top < bottom && bottom < self.rows {
            self.scroll_top = top;
            self.scroll_bottom = bottom;
        }
    }

    // -- alternate screen (smcup / rmcup) -----------------------------------

    pub fn enter_alternate_screen(&mut self) {
        if self.alternate_screen.is_some() {
            return; // already in alternate
        }
        let saved = self.cells.clone();
        self.alternate_screen = Some(saved);
        self.cells = Self::blank_cells(self.cols, self.rows);
        self.cursor = CursorState::default();
        self.mark_all_dirty();
    }

    pub fn exit_alternate_screen(&mut self) {
        if let Some(saved) = self.alternate_screen.take() {
            self.cells = saved;
            self.cursor.row = self.cursor.row.min(self.rows.saturating_sub(1));
            self.cursor.col = self.cursor.col.min(self.cols.saturating_sub(1));
            self.mark_all_dirty();
        }
    }

    // -- reset --------------------------------------------------------------

    pub fn reset(&mut self) {
        *self = Self::new(self.cols, self.rows);
    }

    // -- text extraction ----------------------------------------------------

    /// Extract text content from a single row.
    pub fn row_to_string(&self, row: usize) -> String {
        if row >= self.rows {
            return String::new();
        }
        self.cells[row]
            .iter()
            .filter(|cell| cell.width != 0) // skip continuation cells
            .map(|cell| cell.c)
            .collect::<String>()
            .trim_end()
            .to_string()
    }

    /// All visible content as a string (rows separated by newlines).
    pub fn content_to_string(&self) -> String {
        (0..self.rows)
            .map(|r| self.row_to_string(r))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_grid_has_correct_dimensions() {
        let g = Grid::new(80, 24);
        assert_eq!(g.cols, 80);
        assert_eq!(g.rows, 24);
        assert_eq!(g.cells.len(), 24);
        assert_eq!(g.cells[0].len(), 80);
    }

    #[test]
    fn put_char_writes_at_cursor_and_advances() {
        let mut g = Grid::new(80, 24);
        g.put_char('A');
        assert_eq!(g.cells[0][0].c, 'A');
        assert_eq!(g.cursor.col, 1);
        g.put_char('B');
        assert_eq!(g.cells[0][1].c, 'B');
        assert_eq!(g.cursor.col, 2);
    }

    #[test]
    fn put_char_wraps_at_end_of_line() {
        let mut g = Grid::new(5, 3);
        for ch in "ABCDE".chars() {
            g.put_char(ch);
        }
        // After writing 5 chars in a 5-col grid, wrap_pending should be true.
        assert!(g.wrap_pending);
        assert_eq!(g.cursor.row, 0);
        // Writing one more should trigger the wrap.
        g.put_char('F');
        assert_eq!(g.cursor.row, 1);
        assert_eq!(g.cells[1][0].c, 'F');
    }

    #[test]
    fn newline_scrolls_when_at_bottom() {
        let mut g = Grid::new(5, 3);
        g.cursor.row = 2; // bottom row
        g.cells[0][0].c = 'X';
        g.newline();
        // Should have scrolled: row 0 content should now be gone.
        assert_eq!(g.cursor.row, 2);
        assert_eq!(g.cells[0][0].c, ' ');
    }

    #[test]
    fn carriage_return_moves_to_column_0() {
        let mut g = Grid::new(80, 24);
        g.cursor.col = 40;
        g.carriage_return();
        assert_eq!(g.cursor.col, 0);
    }

    #[test]
    fn tab_advances_to_next_tab_stop() {
        let mut g = Grid::new(80, 24);
        assert_eq!(g.cursor.col, 0);
        g.tab();
        assert_eq!(g.cursor.col, 8);
        g.tab();
        assert_eq!(g.cursor.col, 16);
    }

    #[test]
    fn erase_in_display_mode_2_clears_all() {
        let mut g = Grid::new(5, 3);
        g.put_char('A');
        g.put_char('B');
        g.erase_in_display(2);
        for r in 0..3 {
            for c in 0..5 {
                assert_eq!(g.cells[r][c].c, ' ');
            }
        }
    }

    #[test]
    fn erase_in_line_mode_0_clears_to_right() {
        let mut g = Grid::new(5, 3);
        for ch in "HELLO".chars() {
            g.put_char(ch);
        }
        g.cursor.col = 2;
        g.erase_in_line(0);
        assert_eq!(g.cells[0][0].c, 'H');
        assert_eq!(g.cells[0][1].c, 'E');
        assert_eq!(g.cells[0][2].c, ' ');
        assert_eq!(g.cells[0][3].c, ' ');
        assert_eq!(g.cells[0][4].c, ' ');
    }

    #[test]
    fn cursor_save_restore() {
        let mut g = Grid::new(80, 24);
        g.cursor.row = 5;
        g.cursor.col = 10;
        g.save_cursor();
        g.cursor.row = 0;
        g.cursor.col = 0;
        g.restore_cursor();
        assert_eq!(g.cursor.row, 5);
        assert_eq!(g.cursor.col, 10);
    }

    #[test]
    fn wide_char_takes_2_cells() {
        let mut g = Grid::new(80, 24);
        // CJK character (U+4E16 = world) has width 2.
        g.put_char('\u{4E16}');
        assert_eq!(g.cells[0][0].c, '\u{4E16}');
        assert_eq!(g.cells[0][0].width, 2);
        assert_eq!(g.cells[0][1].width, 0); // continuation
        assert_eq!(g.cursor.col, 2);
    }

    #[test]
    fn scroll_up_moves_lines_up() {
        let mut g = Grid::new(5, 3);
        g.cells[0][0].c = 'A';
        g.cells[1][0].c = 'B';
        g.cells[2][0].c = 'C';
        let scrolled = g.scroll_up(1);
        assert_eq!(scrolled.len(), 1);
        assert_eq!(scrolled[0][0].c, 'A');
        assert_eq!(g.cells[0][0].c, 'B');
        assert_eq!(g.cells[1][0].c, 'C');
        assert_eq!(g.cells[2][0].c, ' '); // new blank row
    }

    #[test]
    fn resize_preserves_content() {
        let mut g = Grid::new(5, 3);
        g.cells[0][0].c = 'X';
        g.cells[1][0].c = 'Y';
        g.resize(10, 5);
        assert_eq!(g.cols, 10);
        assert_eq!(g.rows, 5);
        assert_eq!(g.cells[0][0].c, 'X');
        assert_eq!(g.cells[1][0].c, 'Y');
    }

    #[test]
    fn row_to_string_returns_correct_text() {
        let mut g = Grid::new(80, 24);
        for ch in "Hello".chars() {
            g.put_char(ch);
        }
        assert_eq!(g.row_to_string(0), "Hello");
    }

    #[test]
    fn alternate_screen_saves_and_restores() {
        let mut g = Grid::new(5, 3);
        g.cells[0][0].c = 'A';
        g.enter_alternate_screen();
        // Screen should be blank now.
        assert_eq!(g.cells[0][0].c, ' ');
        g.cells[0][0].c = 'Z';
        g.exit_alternate_screen();
        // Should be back to the original content.
        assert_eq!(g.cells[0][0].c, 'A');
    }

    #[test]
    fn scroll_down_moves_lines_down() {
        let mut g = Grid::new(5, 3);
        g.cells[0][0].c = 'A';
        g.cells[1][0].c = 'B';
        g.cells[2][0].c = 'C';
        g.scroll_down(1);
        assert_eq!(g.cells[0][0].c, ' '); // new blank row
        assert_eq!(g.cells[1][0].c, 'A');
        assert_eq!(g.cells[2][0].c, 'B');
    }

    #[test]
    fn backspace_moves_cursor_left() {
        let mut g = Grid::new(80, 24);
        g.cursor.col = 5;
        g.backspace();
        assert_eq!(g.cursor.col, 4);
        g.cursor.col = 0;
        g.backspace();
        assert_eq!(g.cursor.col, 0); // does not go below 0
    }

    #[test]
    fn delete_chars_shifts_left() {
        let mut g = Grid::new(5, 1);
        for ch in "ABCDE".chars() {
            g.put_char(ch);
        }
        g.cursor.col = 1;
        g.delete_chars(2);
        assert_eq!(g.cells[0][0].c, 'A');
        assert_eq!(g.cells[0][1].c, 'D');
        assert_eq!(g.cells[0][2].c, 'E');
        assert_eq!(g.cells[0][3].c, ' ');
    }

    #[test]
    fn insert_blank_chars_shifts_right() {
        let mut g = Grid::new(5, 1);
        for ch in "ABCDE".chars() {
            g.put_char(ch);
        }
        g.cursor.col = 1;
        g.insert_blank_chars(2);
        assert_eq!(g.cells[0][0].c, 'A');
        assert_eq!(g.cells[0][1].c, ' ');
        assert_eq!(g.cells[0][2].c, ' ');
        assert_eq!(g.cells[0][3].c, 'B');
        assert_eq!(g.cells[0][4].c, 'C');
    }

    #[test]
    fn content_to_string_all_rows() {
        let mut g = Grid::new(10, 2);
        for ch in "Hello".chars() {
            g.put_char(ch);
        }
        g.cursor.row = 1;
        g.cursor.col = 0;
        for ch in "World".chars() {
            g.put_char(ch);
        }
        let s = g.content_to_string();
        assert_eq!(s, "Hello\nWorld");
    }

    #[test]
    fn insert_lines_within_scroll_region() {
        let mut g = Grid::new(5, 4);
        g.cells[0][0].c = 'A';
        g.cells[1][0].c = 'B';
        g.cells[2][0].c = 'C';
        g.cells[3][0].c = 'D';
        g.cursor.row = 1;
        g.insert_lines(1);
        assert_eq!(g.cells[0][0].c, 'A');
        assert_eq!(g.cells[1][0].c, ' '); // inserted blank
        assert_eq!(g.cells[2][0].c, 'B');
        assert_eq!(g.cells[3][0].c, 'C');
    }

    #[test]
    fn delete_lines_within_scroll_region() {
        let mut g = Grid::new(5, 4);
        g.cells[0][0].c = 'A';
        g.cells[1][0].c = 'B';
        g.cells[2][0].c = 'C';
        g.cells[3][0].c = 'D';
        g.cursor.row = 1;
        g.delete_lines(1);
        assert_eq!(g.cells[0][0].c, 'A');
        assert_eq!(g.cells[1][0].c, 'C');
        assert_eq!(g.cells[2][0].c, 'D');
        assert_eq!(g.cells[3][0].c, ' '); // blank at bottom
    }

    #[test]
    fn set_scroll_region_and_scroll() {
        let mut g = Grid::new(5, 5);
        for (i, ch) in "ABCDE".chars().enumerate() {
            g.cells[i][0].c = ch;
        }
        g.set_scroll_region(1, 3);
        g.scroll_up(1);
        assert_eq!(g.cells[0][0].c, 'A'); // outside region
        assert_eq!(g.cells[1][0].c, 'C'); // was row 2
        assert_eq!(g.cells[2][0].c, 'D'); // was row 3
        assert_eq!(g.cells[3][0].c, ' '); // new blank in region
        assert_eq!(g.cells[4][0].c, 'E'); // outside region
    }

    #[test]
    fn move_cursor_clamps_to_bounds() {
        let mut g = Grid::new(10, 5);
        g.move_cursor(100, 100);
        assert_eq!(g.cursor.row, 4);
        assert_eq!(g.cursor.col, 9);
    }

    #[test]
    fn move_cursor_relative_clamps() {
        let mut g = Grid::new(10, 5);
        g.cursor.row = 2;
        g.cursor.col = 5;
        g.move_cursor_relative(-10, -10);
        assert_eq!(g.cursor.row, 0);
        assert_eq!(g.cursor.col, 0);
        g.move_cursor_relative(100, 100);
        assert_eq!(g.cursor.row, 4);
        assert_eq!(g.cursor.col, 9);
    }

    #[test]
    fn erase_in_display_mode_0_clears_below() {
        let mut g = Grid::new(5, 3);
        for r in 0..3 {
            for c in 0..5 {
                g.cells[r][c].c = 'X';
            }
        }
        g.cursor.row = 1;
        g.cursor.col = 2;
        g.erase_in_display(0);
        // Row 0 untouched.
        for c in 0..5 {
            assert_eq!(g.cells[0][c].c, 'X');
        }
        // Row 1 cols 0-1 untouched, 2-4 cleared.
        assert_eq!(g.cells[1][0].c, 'X');
        assert_eq!(g.cells[1][1].c, 'X');
        assert_eq!(g.cells[1][2].c, ' ');
        assert_eq!(g.cells[1][3].c, ' ');
        // Row 2 all cleared.
        for c in 0..5 {
            assert_eq!(g.cells[2][c].c, ' ');
        }
    }

    #[test]
    fn erase_in_display_mode_1_clears_above() {
        let mut g = Grid::new(5, 3);
        for r in 0..3 {
            for c in 0..5 {
                g.cells[r][c].c = 'X';
            }
        }
        g.cursor.row = 1;
        g.cursor.col = 2;
        g.erase_in_display(1);
        // Row 0 all cleared.
        for c in 0..5 {
            assert_eq!(g.cells[0][c].c, ' ');
        }
        // Row 1 cols 0-2 cleared, 3-4 untouched.
        assert_eq!(g.cells[1][0].c, ' ');
        assert_eq!(g.cells[1][1].c, ' ');
        assert_eq!(g.cells[1][2].c, ' ');
        assert_eq!(g.cells[1][3].c, 'X');
        assert_eq!(g.cells[1][4].c, 'X');
        // Row 2 untouched.
        for c in 0..5 {
            assert_eq!(g.cells[2][c].c, 'X');
        }
    }

    #[test]
    fn erase_in_line_mode_1_clears_to_left() {
        let mut g = Grid::new(5, 1);
        for ch in "ABCDE".chars() {
            g.put_char(ch);
        }
        g.cursor.col = 2;
        g.erase_in_line(1);
        assert_eq!(g.cells[0][0].c, ' ');
        assert_eq!(g.cells[0][1].c, ' ');
        assert_eq!(g.cells[0][2].c, ' ');
        assert_eq!(g.cells[0][3].c, 'D');
        assert_eq!(g.cells[0][4].c, 'E');
    }

    #[test]
    fn erase_in_line_mode_2_clears_whole_line() {
        let mut g = Grid::new(5, 1);
        for ch in "ABCDE".chars() {
            g.put_char(ch);
        }
        g.erase_in_line(2);
        for c in 0..5 {
            assert_eq!(g.cells[0][c].c, ' ');
        }
    }

    #[test]
    fn reset_restores_defaults() {
        let mut g = Grid::new(80, 24);
        g.put_char('Z');
        g.cursor.row = 10;
        g.attrs.bold = true;
        g.reset();
        assert_eq!(g.cells[0][0].c, ' ');
        assert_eq!(g.cursor.row, 0);
        assert_eq!(g.cursor.col, 0);
        assert!(!g.attrs.bold);
    }

    #[test]
    fn erase_chars_replaces_with_blanks() {
        let mut g = Grid::new(5, 1);
        for ch in "ABCDE".chars() {
            g.put_char(ch);
        }
        g.cursor.col = 1;
        g.erase_chars(2);
        assert_eq!(g.cells[0][0].c, 'A');
        assert_eq!(g.cells[0][1].c, ' ');
        assert_eq!(g.cells[0][2].c, ' ');
        assert_eq!(g.cells[0][3].c, 'D');
        assert_eq!(g.cells[0][4].c, 'E');
    }

    #[test]
    fn wide_char_at_end_of_line_wraps() {
        let mut g = Grid::new(5, 2);
        g.cursor.col = 4; // last column
        g.put_char('\u{4E16}'); // wide char width=2
                                // Should wrap to next line.
        assert_eq!(g.cursor.row, 1);
        assert_eq!(g.cells[1][0].c, '\u{4E16}');
        assert_eq!(g.cells[1][0].width, 2);
    }

    // -- dirty tracking tests -----------------------------------------------

    #[test]
    fn new_grid_starts_all_dirty() {
        let mut g = Grid::new(80, 24);
        let dirty = g.take_dirty();
        assert!(dirty.iter().all(|&d| d));
        assert_eq!(dirty.len(), 24);
    }

    #[test]
    fn take_dirty_clears_flags() {
        let mut g = Grid::new(80, 24);
        let _ = g.take_dirty();
        let dirty = g.take_dirty();
        assert!(dirty.iter().all(|&d| !d));
    }

    #[test]
    fn put_char_marks_cursor_row_dirty() {
        let mut g = Grid::new(80, 24);
        let _ = g.take_dirty();
        g.put_char('A');
        let dirty = g.take_dirty();
        assert!(dirty[0]);
        assert!(!dirty[1]);
    }

    #[test]
    fn scroll_up_marks_scroll_region_dirty() {
        let mut g = Grid::new(5, 5);
        let _ = g.take_dirty();
        g.scroll_up(1);
        let dirty = g.take_dirty();
        // Default scroll region is 0..4
        assert!(dirty.iter().all(|&d| d));
    }

    #[test]
    fn erase_in_display_mode2_marks_all_dirty() {
        let mut g = Grid::new(5, 3);
        let _ = g.take_dirty();
        g.erase_in_display(2);
        let dirty = g.take_dirty();
        assert!(dirty.iter().all(|&d| d));
    }

    #[test]
    fn erase_in_line_marks_only_that_row() {
        let mut g = Grid::new(10, 3);
        g.cursor.row = 1;
        let _ = g.take_dirty();
        g.erase_in_line(2);
        let dirty = g.take_dirty();
        assert!(!dirty[0]);
        assert!(dirty[1]);
        assert!(!dirty[2]);
    }

    #[test]
    fn move_cursor_marks_old_and_new_rows() {
        let mut g = Grid::new(80, 24);
        let _ = g.take_dirty();
        g.move_cursor(5, 0);
        let dirty = g.take_dirty();
        assert!(dirty[0]); // old row
        assert!(dirty[5]); // new row
        assert!(!dirty[3]); // untouched
    }

    #[test]
    fn resize_marks_all_dirty() {
        let mut g = Grid::new(80, 24);
        let _ = g.take_dirty();
        g.resize(40, 12);
        let dirty = g.take_dirty();
        assert_eq!(dirty.len(), 12);
        assert!(dirty.iter().all(|&d| d));
    }

    #[test]
    fn any_dirty_returns_false_after_take() {
        let mut g = Grid::new(80, 24);
        assert!(g.any_dirty());
        let _ = g.take_dirty();
        assert!(!g.any_dirty());
        g.put_char('X');
        assert!(g.any_dirty());
    }
}
