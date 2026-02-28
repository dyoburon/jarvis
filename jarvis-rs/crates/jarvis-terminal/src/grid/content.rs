//! Character output and text extraction.

use unicode_width::UnicodeWidthChar;

use super::core::Grid;
use super::types::Cell;

impl Grid {
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
