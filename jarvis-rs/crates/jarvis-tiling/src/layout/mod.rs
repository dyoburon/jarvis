pub mod borders;
mod calculation;
mod types;

pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::{Direction, SplitNode};
    use jarvis_common::types::Rect;

    #[test]
    fn single_pane_fills_bounds() {
        let engine = LayoutEngine {
            gap: 0,
            min_pane_size: 10.0,
        };
        let root = SplitNode::Leaf { pane_id: 1 };
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        };
        let result = engine.compute(&root, bounds);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], (1, bounds));
    }

    #[test]
    fn horizontal_split_divides_width() {
        let engine = LayoutEngine {
            gap: 0,
            min_pane_size: 10.0,
        };
        let root = SplitNode::Split {
            direction: Direction::Horizontal,
            ratio: 0.5,
            first: Box::new(SplitNode::Leaf { pane_id: 1 }),
            second: Box::new(SplitNode::Leaf { pane_id: 2 }),
        };
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        };
        let result = engine.compute(&root, bounds);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 1);
        assert_eq!(result[1].0, 2);
        assert!((result[0].1.width - 400.0).abs() < 0.01);
        assert!((result[1].1.width - 400.0).abs() < 0.01);
    }

    #[test]
    fn gap_reduces_available_space() {
        let engine = LayoutEngine {
            gap: 10,
            min_pane_size: 10.0,
        };
        let root = SplitNode::Split {
            direction: Direction::Horizontal,
            ratio: 0.5,
            first: Box::new(SplitNode::Leaf { pane_id: 1 }),
            second: Box::new(SplitNode::Leaf { pane_id: 2 }),
        };
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        };
        let result = engine.compute(&root, bounds);
        let total = result[0].1.width + result[1].1.width;
        assert!((total - 790.0).abs() < 0.01);
    }

    #[test]
    fn nested_splits() {
        let engine = LayoutEngine::default();
        let root = SplitNode::Split {
            direction: Direction::Horizontal,
            ratio: 0.5,
            first: Box::new(SplitNode::Leaf { pane_id: 1 }),
            second: Box::new(SplitNode::Split {
                direction: Direction::Vertical,
                ratio: 0.5,
                first: Box::new(SplitNode::Leaf { pane_id: 2 }),
                second: Box::new(SplitNode::Leaf { pane_id: 3 }),
            }),
        };
        let bounds = Rect {
            x: 0.0,
            y: 0.0,
            width: 800.0,
            height: 600.0,
        };
        let result = engine.compute(&root, bounds);
        assert_eq!(result.len(), 3);
    }
}
