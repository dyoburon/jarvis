//! Mouse coordinate translation utilities.
//!
//! Translates pixel coordinates from windowing events to terminal grid
//! positions and scroll amounts.

/// Translate pixel coordinates to terminal grid cell (row, col).
///
/// `content_offset_x` and `content_offset_y` account for UI chrome
/// (tab bar, status bar, padding) that shifts the terminal content area.
pub fn pixel_to_grid(
    pixel_x: f64,
    pixel_y: f64,
    cell_width: f32,
    cell_height: f32,
    content_offset_x: f64,
    content_offset_y: f64,
) -> (usize, usize) {
    let col = ((pixel_x - content_offset_x) / cell_width as f64)
        .floor()
        .max(0.0) as usize;
    let row = ((pixel_y - content_offset_y) / cell_height as f64)
        .floor()
        .max(0.0) as usize;
    (row, col)
}

/// Convert a scroll wheel delta to a number of terminal lines.
///
/// Returns positive for scroll up, negative for scroll down.
pub fn scroll_delta_to_lines(delta_y: f64, cell_height: f32) -> i32 {
    if cell_height <= 0.0 {
        return 0;
    }
    // Each "click" of the scroll wheel typically sends ~3 lines worth of delta.
    // We normalize by cell height for consistent behavior across font sizes.
    (delta_y / cell_height as f64).round() as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_to_grid_origin() {
        let (row, col) = pixel_to_grid(0.0, 0.0, 10.0, 20.0, 0.0, 0.0);
        assert_eq!((row, col), (0, 0));
    }

    #[test]
    fn pixel_to_grid_with_offset() {
        // Tab bar is 32px, so content starts at y=32
        let (row, col) = pixel_to_grid(50.0, 52.0, 10.0, 20.0, 0.0, 32.0);
        assert_eq!(col, 5); // 50/10 = 5
        assert_eq!(row, 1); // (52-32)/20 = 1
    }

    #[test]
    fn pixel_to_grid_negative_clamps_to_zero() {
        let (row, col) = pixel_to_grid(-10.0, -10.0, 10.0, 20.0, 0.0, 0.0);
        assert_eq!((row, col), (0, 0));
    }

    #[test]
    fn scroll_delta_positive() {
        let lines = scroll_delta_to_lines(60.0, 20.0);
        assert_eq!(lines, 3);
    }

    #[test]
    fn scroll_delta_negative() {
        let lines = scroll_delta_to_lines(-40.0, 20.0);
        assert_eq!(lines, -2);
    }

    #[test]
    fn scroll_delta_zero_cell_height() {
        let lines = scroll_delta_to_lines(100.0, 0.0);
        assert_eq!(lines, 0);
    }

    #[test]
    fn scroll_delta_fractional() {
        let lines = scroll_delta_to_lines(25.0, 20.0);
        assert_eq!(lines, 1); // 1.25 rounds to 1
    }
}
