//! Terminal size adapter implementing alacritty_terminal's Dimensions trait.
//!
//! @module terminal/size

use alacritty_terminal::grid::Dimensions;

// =============================================================================
// SIZE INFO
// =============================================================================

/// Terminal dimensions in both pixel and cell coordinates.
///
/// Implements `Dimensions` for use with `Term::new()` and `Term::resize()`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizeInfo {
    /// Total width of the terminal area in pixels.
    pub width: f32,
    /// Total height of the terminal area in pixels.
    pub height: f32,
    /// Width of a single cell in pixels.
    pub cell_width: f32,
    /// Height of a single cell in pixels.
    pub cell_height: f32,
    /// Horizontal padding in pixels.
    pub padding_x: f32,
    /// Vertical padding in pixels.
    pub padding_y: f32,
    /// Number of columns (computed from width, cell_width, padding).
    columns: usize,
    /// Number of visible lines (computed from height, cell_height, padding).
    screen_lines: usize,
}

impl SizeInfo {
    /// Create a new SizeInfo from cell dimensions and counts.
    ///
    /// This is the primary constructor for tests and terminal creation where
    /// you know the desired column/line counts.
    pub fn new(columns: usize, screen_lines: usize, cell_width: f32, cell_height: f32) -> Self {
        let padding_x = 0.0;
        let padding_y = 0.0;
        let width = (columns as f32) * cell_width + padding_x * 2.0;
        let height = (screen_lines as f32) * cell_height + padding_y * 2.0;

        Self {
            width,
            height,
            cell_width,
            cell_height,
            padding_x,
            padding_y,
            columns,
            screen_lines,
        }
    }

    /// Create a SizeInfo from pixel dimensions, computing cell counts.
    ///
    /// Used when the window resizes and we need to recalculate how many
    /// columns/lines fit in the new pixel area.
    pub fn from_pixels(
        width: f32,
        height: f32,
        cell_width: f32,
        cell_height: f32,
        padding_x: f32,
        padding_y: f32,
    ) -> Self {
        let usable_width = (width - padding_x * 2.0).max(0.0);
        let usable_height = (height - padding_y * 2.0).max(0.0);
        let columns = (usable_width / cell_width).floor() as usize;
        let screen_lines = (usable_height / cell_height).floor() as usize;

        // Ensure at least 1x1 terminal.
        let columns = columns.max(1);
        let screen_lines = screen_lines.max(1);

        Self {
            width,
            height,
            cell_width,
            cell_height,
            padding_x,
            padding_y,
            columns,
            screen_lines,
        }
    }
}

impl Dimensions for SizeInfo {
    fn columns(&self) -> usize {
        self.columns
    }

    fn screen_lines(&self) -> usize {
        self.screen_lines
    }

    fn total_lines(&self) -> usize {
        // For initial creation, total_lines equals screen_lines.
        // The Term itself tracks scrollback history internally.
        self.screen_lines
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_info_new_80x24() {
        let size = SizeInfo::new(80, 24, 10.0, 20.0);

        assert_eq!(size.columns(), 80);
        assert_eq!(size.screen_lines(), 24);
        assert_eq!(size.total_lines(), 24);
        assert_eq!(size.cell_width, 10.0);
        assert_eq!(size.cell_height, 20.0);
        assert_eq!(size.width, 800.0);
        assert_eq!(size.height, 480.0);
    }

    #[test]
    fn size_info_from_pixels() {
        let size = SizeInfo::from_pixels(810.0, 490.0, 10.0, 20.0, 5.0, 5.0);

        // Usable: 800x480 → 80 cols, 24 lines
        assert_eq!(size.columns(), 80);
        assert_eq!(size.screen_lines(), 24);
    }

    #[test]
    fn size_info_from_pixels_minimum_1x1() {
        let size = SizeInfo::from_pixels(5.0, 5.0, 10.0, 20.0, 0.0, 0.0);

        // Too small for even 1 cell, but clamped to 1x1
        assert_eq!(size.columns(), 1);
        assert_eq!(size.screen_lines(), 1);
    }

    #[test]
    fn size_info_from_pixels_with_large_padding() {
        let size = SizeInfo::from_pixels(100.0, 100.0, 10.0, 20.0, 50.0, 50.0);

        // Usable: 0x0 → clamped to 1x1
        assert_eq!(size.columns(), 1);
        assert_eq!(size.screen_lines(), 1);
    }

    #[test]
    fn size_info_clone_eq() {
        let a = SizeInfo::new(80, 24, 10.0, 20.0);
        let b = a;

        assert_eq!(a, b);
    }

    /// Phase 1 verification: Term creation with our SizeInfo works.
    #[test]
    fn term_creation_with_size_info() {
        use alacritty_terminal::event::VoidListener;
        use alacritty_terminal::term::{Config, Term};

        let size = SizeInfo::new(80, 24, 10.0, 20.0);
        let term = Term::new(Config::default(), &size, VoidListener);

        assert_eq!(term.columns(), 80);
        assert_eq!(term.screen_lines(), 24);
    }

    /// Phase 1 verification: Term creation with JarvisEventProxy works.
    #[test]
    fn term_creation_with_event_proxy() {
        use crate::event::JarvisEventProxy;
        use alacritty_terminal::term::{Config, Term};

        let size = SizeInfo::new(120, 40, 8.0, 16.0);
        let (proxy, _rx) = JarvisEventProxy::new();
        let term = Term::new(Config::default(), &size, proxy);

        assert_eq!(term.columns(), 120);
        assert_eq!(term.screen_lines(), 40);
    }
}
