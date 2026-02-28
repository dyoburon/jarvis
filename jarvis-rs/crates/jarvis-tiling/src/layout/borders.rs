//! Split border computation for drag-resize hit testing.
//!
//! Walks the split tree and produces `SplitBorder` entries — one per
//! split node — that describe where the divider line sits and which
//! pane IDs are on each side. The event handler uses these to detect
//! when the cursor is near a border and to update the correct ratio.

use jarvis_common::types::Rect;

use crate::tree::{Direction, SplitNode};

// =============================================================================
// TYPES
// =============================================================================

/// A split border between two tiling regions.
#[derive(Debug, Clone, PartialEq)]
pub struct SplitBorder {
    /// The direction of the split (Horizontal = vertical divider line).
    pub direction: Direction,
    /// Position of the divider in pixels (x for horizontal, y for vertical).
    pub position: f64,
    /// Start of the divider line (y for horizontal, x for vertical).
    pub start: f64,
    /// End of the divider line.
    pub end: f64,
    /// The first pane ID in the first subtree (used to adjust ratio).
    pub first_pane: u32,
    /// The bounding rect of the entire split region.
    pub bounds: Rect,
}

impl SplitBorder {
    /// Half-width of the hit zone on each side of the border.
    const HIT_HALF_WIDTH: f64 = 6.0;

    /// Test whether a point (x, y) is within the drag zone of this border.
    pub fn hit_test(&self, x: f64, y: f64) -> bool {
        match self.direction {
            Direction::Horizontal => {
                // Vertical divider line: check x within range, y within span
                (x - self.position).abs() <= Self::HIT_HALF_WIDTH
                    && y >= self.start
                    && y <= self.end
            }
            Direction::Vertical => {
                // Horizontal divider line: check y within range, x within span
                (y - self.position).abs() <= Self::HIT_HALF_WIDTH
                    && x >= self.start
                    && x <= self.end
            }
        }
    }

    /// Convert a pixel delta to a ratio delta for this border.
    pub fn pixel_to_ratio(&self, pixel_delta: f64) -> f64 {
        let span = match self.direction {
            Direction::Horizontal => self.bounds.width,
            Direction::Vertical => self.bounds.height,
        };
        if span <= 0.0 {
            return 0.0;
        }
        pixel_delta / span
    }
}

// =============================================================================
// COMPUTATION
// =============================================================================

/// Compute all split borders from the tree within the given viewport.
pub fn compute_borders(root: &SplitNode, bounds: Rect, gap: f64) -> Vec<SplitBorder> {
    let mut borders = Vec::new();
    walk_borders(root, bounds, gap, &mut borders);
    borders
}

fn walk_borders(node: &SplitNode, bounds: Rect, gap: f64, out: &mut Vec<SplitBorder>) {
    match node {
        SplitNode::Leaf { .. } => {}
        SplitNode::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            let first_pane = first.collect_pane_ids().into_iter().next().unwrap_or(0);

            match direction {
                Direction::Horizontal => {
                    let avail = (bounds.width - gap).max(0.0);
                    let w1 = avail * ratio;
                    let border_x = bounds.x + w1 + gap / 2.0;

                    out.push(SplitBorder {
                        direction: *direction,
                        position: border_x,
                        start: bounds.y,
                        end: bounds.y + bounds.height,
                        first_pane,
                        bounds,
                    });

                    let first_bounds = Rect {
                        x: bounds.x,
                        y: bounds.y,
                        width: w1,
                        height: bounds.height,
                    };
                    let second_bounds = Rect {
                        x: bounds.x + w1 + gap,
                        y: bounds.y,
                        width: (avail - w1).max(0.0),
                        height: bounds.height,
                    };
                    walk_borders(first, first_bounds, gap, out);
                    walk_borders(second, second_bounds, gap, out);
                }
                Direction::Vertical => {
                    let avail = (bounds.height - gap).max(0.0);
                    let h1 = avail * ratio;
                    let border_y = bounds.y + h1 + gap / 2.0;

                    out.push(SplitBorder {
                        direction: *direction,
                        position: border_y,
                        start: bounds.x,
                        end: bounds.x + bounds.width,
                        first_pane,
                        bounds,
                    });

                    let first_bounds = Rect {
                        x: bounds.x,
                        y: bounds.y,
                        width: bounds.width,
                        height: h1,
                    };
                    let second_bounds = Rect {
                        x: bounds.x,
                        y: bounds.y + h1 + gap,
                        width: bounds.width,
                        height: (avail - h1).max(0.0),
                    };
                    walk_borders(first, first_bounds, gap, out);
                    walk_borders(second, second_bounds, gap, out);
                }
            }
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn viewport() -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        }
    }

    #[test]
    fn single_pane_no_borders() {
        let tree = SplitNode::leaf(1);
        let borders = compute_borders(&tree, viewport(), 2.0);
        assert!(borders.is_empty());
    }

    #[test]
    fn horizontal_split_one_border() {
        let tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        let borders = compute_borders(&tree, viewport(), 2.0);
        assert_eq!(borders.len(), 1);
        assert_eq!(borders[0].direction, Direction::Horizontal);
        // At 50% of (800 - 2) = 399, border at 399 + 1 = 400
        assert!((borders[0].position - 400.0).abs() < 1.0);
        assert_eq!(borders[0].first_pane, 1);
    }

    #[test]
    fn vertical_split_one_border() {
        let tree = SplitNode::split_v(SplitNode::leaf(1), SplitNode::leaf(2));
        let borders = compute_borders(&tree, viewport(), 2.0);
        assert_eq!(borders.len(), 1);
        assert_eq!(borders[0].direction, Direction::Vertical);
        // At 50% of (600 - 2) = 299, border at 299 + 1 = 300
        assert!((borders[0].position - 300.0).abs() < 1.0);
    }

    #[test]
    fn nested_split_two_borders() {
        // [1 | 2 / 3]
        let tree = SplitNode::split_h(
            SplitNode::leaf(1),
            SplitNode::split_v(SplitNode::leaf(2), SplitNode::leaf(3)),
        );
        let borders = compute_borders(&tree, viewport(), 2.0);
        assert_eq!(borders.len(), 2);
        // First border is the horizontal split between pane 1 and (2/3)
        assert_eq!(borders[0].direction, Direction::Horizontal);
        // Second border is the vertical split between 2 and 3
        assert_eq!(borders[1].direction, Direction::Vertical);
    }

    #[test]
    fn hit_test_horizontal_border() {
        let border = SplitBorder {
            direction: Direction::Horizontal,
            position: 400.0,
            start: 0.0,
            end: 600.0,
            first_pane: 1,
            bounds: viewport(),
        };
        // On the border
        assert!(border.hit_test(400.0, 300.0));
        // Just within hit zone (6px)
        assert!(border.hit_test(405.0, 300.0));
        assert!(border.hit_test(395.0, 300.0));
        // Outside hit zone
        assert!(!border.hit_test(410.0, 300.0));
        // Outside vertical span
        assert!(!border.hit_test(400.0, -1.0));
        assert!(!border.hit_test(400.0, 601.0));
    }

    #[test]
    fn hit_test_vertical_border() {
        let border = SplitBorder {
            direction: Direction::Vertical,
            position: 300.0,
            start: 0.0,
            end: 800.0,
            first_pane: 1,
            bounds: viewport(),
        };
        assert!(border.hit_test(400.0, 300.0));
        assert!(border.hit_test(400.0, 305.0));
        assert!(!border.hit_test(400.0, 310.0));
    }

    #[test]
    fn pixel_to_ratio_horizontal() {
        let border = SplitBorder {
            direction: Direction::Horizontal,
            position: 400.0,
            start: 0.0,
            end: 600.0,
            first_pane: 1,
            bounds: viewport(), // width=800
        };
        // 80px = 10% of 800
        assert!((border.pixel_to_ratio(80.0) - 0.1).abs() < 0.001);
    }

    #[test]
    fn pixel_to_ratio_vertical() {
        let border = SplitBorder {
            direction: Direction::Vertical,
            position: 300.0,
            start: 0.0,
            end: 800.0,
            first_pane: 1,
            bounds: viewport(), // height=600
        };
        // 60px = 10% of 600
        assert!((border.pixel_to_ratio(60.0) - 0.1).abs() < 0.001);
    }

    #[test]
    fn pixel_to_ratio_zero_span() {
        let border = SplitBorder {
            direction: Direction::Horizontal,
            position: 0.0,
            start: 0.0,
            end: 0.0,
            first_pane: 1,
            bounds: Rect {
                x: 0.0,
                y: 0.0,
                width: 0.0,
                height: 0.0,
            },
        };
        assert_eq!(border.pixel_to_ratio(100.0), 0.0);
    }
}
