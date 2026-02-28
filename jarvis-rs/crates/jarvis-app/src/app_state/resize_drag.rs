//! Mouse-based drag resize state for split borders.
//!
//! Tracks whether the user is dragging a split border and which
//! border they are dragging. The event handler calls into this
//! module on cursor movement and mouse button events.

use jarvis_tiling::layout::borders::SplitBorder;
use jarvis_tiling::tree::Direction;

// =============================================================================
// TYPES
// =============================================================================

/// Active drag state during a resize operation.
#[derive(Debug, Clone)]
pub struct DragState {
    /// The border being dragged.
    pub border: SplitBorder,
    /// Cursor position (in direction axis) when drag started.
    pub start_pos: f64,
}

/// Result of checking cursor position against split borders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorZone {
    /// Not near any border.
    None,
    /// Near a horizontal split border (vertical divider line).
    ColResize,
    /// Near a vertical split border (horizontal divider line).
    RowResize,
}

// =============================================================================
// HIT TESTING
// =============================================================================

/// Find which border (if any) the cursor is near.
pub fn find_hovered_border(
    borders: &[SplitBorder],
    cursor_x: f64,
    cursor_y: f64,
) -> Option<&SplitBorder> {
    borders.iter().find(|b| b.hit_test(cursor_x, cursor_y))
}

/// Determine the cursor zone from the hovered border.
pub fn cursor_zone(border: Option<&SplitBorder>) -> CursorZone {
    match border {
        Some(b) => match b.direction {
            Direction::Horizontal => CursorZone::ColResize,
            Direction::Vertical => CursorZone::RowResize,
        },
        None => CursorZone::None,
    }
}

/// Compute the ratio delta from a pixel delta during a drag.
pub fn drag_ratio_delta(drag: &DragState, current_pos: f64) -> f64 {
    let pixel_delta = current_pos - drag.start_pos;
    drag.border.pixel_to_ratio(pixel_delta)
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use jarvis_common::types::Rect;

    fn sample_border() -> SplitBorder {
        SplitBorder {
            direction: Direction::Horizontal,
            position: 400.0,
            start: 0.0,
            end: 600.0,
            first_pane: 1,
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 800.0,
                height: 600.0,
            },
        }
    }

    #[test]
    fn find_hovered_border_hit() {
        let borders = vec![sample_border()];
        let result = find_hovered_border(&borders, 400.0, 300.0);
        assert!(result.is_some());
        assert_eq!(result.unwrap().first_pane, 1);
    }

    #[test]
    fn find_hovered_border_miss() {
        let borders = vec![sample_border()];
        let result = find_hovered_border(&borders, 100.0, 300.0);
        assert!(result.is_none());
    }

    #[test]
    fn find_hovered_border_empty() {
        let borders: Vec<SplitBorder> = vec![];
        assert!(find_hovered_border(&borders, 0.0, 0.0).is_none());
    }

    #[test]
    fn cursor_zone_col_resize() {
        let border = sample_border();
        assert_eq!(cursor_zone(Some(&border)), CursorZone::ColResize);
    }

    #[test]
    fn cursor_zone_row_resize() {
        let border = SplitBorder {
            direction: Direction::Vertical,
            ..sample_border()
        };
        assert_eq!(cursor_zone(Some(&border)), CursorZone::RowResize);
    }

    #[test]
    fn cursor_zone_none() {
        assert_eq!(cursor_zone(None), CursorZone::None);
    }

    #[test]
    fn drag_ratio_delta_positive() {
        let drag = DragState {
            border: sample_border(), // width=800
            start_pos: 400.0,
        };
        // Move 80px right = 10% of 800
        let delta = drag_ratio_delta(&drag, 480.0);
        assert!((delta - 0.1).abs() < 0.001);
    }

    #[test]
    fn drag_ratio_delta_negative() {
        let drag = DragState {
            border: sample_border(),
            start_pos: 400.0,
        };
        let delta = drag_ratio_delta(&drag, 320.0);
        assert!((delta - (-0.1)).abs() < 0.001);
    }

    #[test]
    fn drag_ratio_delta_no_movement() {
        let drag = DragState {
            border: sample_border(),
            start_pos: 400.0,
        };
        assert_eq!(drag_ratio_delta(&drag, 400.0), 0.0);
    }
}
