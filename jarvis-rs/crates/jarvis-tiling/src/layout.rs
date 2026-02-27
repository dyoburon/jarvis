use jarvis_common::types::Rect;
use crate::tree::{Direction, SplitNode};

pub struct LayoutEngine {
    pub gap: u32,
    pub min_pane_size: f64,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self {
            gap: 2,
            min_pane_size: 50.0,
        }
    }
}

impl LayoutEngine {
    pub fn compute(&self, root: &SplitNode, bounds: Rect) -> Vec<(u32, Rect)> {
        let mut results = Vec::new();
        self.layout_node(root, bounds, &mut results);
        results
    }

    fn layout_node(&self, node: &SplitNode, bounds: Rect, out: &mut Vec<(u32, Rect)>) {
        match node {
            SplitNode::Leaf { pane_id } => {
                out.push((*pane_id, bounds));
            }
            SplitNode::Split { direction, ratio, first, second } => {
                let gap = self.gap as f64;
                let (a, b) = match direction {
                    Direction::Horizontal => {
                        let w1 = (bounds.width - gap) * ratio;
                        let w2 = bounds.width - gap - w1;
                        (
                            Rect { x: bounds.x, y: bounds.y, width: w1, height: bounds.height },
                            Rect { x: bounds.x + w1 + gap, y: bounds.y, width: w2, height: bounds.height },
                        )
                    }
                    Direction::Vertical => {
                        let h1 = (bounds.height - gap) * ratio;
                        let h2 = bounds.height - gap - h1;
                        (
                            Rect { x: bounds.x, y: bounds.y, width: bounds.width, height: h1 },
                            Rect { x: bounds.x, y: bounds.y + h1 + gap, width: bounds.width, height: h2 },
                        )
                    }
                };
                self.layout_node(first, a, out);
                self.layout_node(second, b, out);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_pane_fills_bounds() {
        let engine = LayoutEngine { gap: 0, min_pane_size: 10.0 };
        let root = SplitNode::Leaf { pane_id: 1 };
        let bounds = Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 };
        let result = engine.compute(&root, bounds);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], (1, bounds));
    }

    #[test]
    fn horizontal_split_divides_width() {
        let engine = LayoutEngine { gap: 0, min_pane_size: 10.0 };
        let root = SplitNode::Split {
            direction: Direction::Horizontal,
            ratio: 0.5,
            first: Box::new(SplitNode::Leaf { pane_id: 1 }),
            second: Box::new(SplitNode::Leaf { pane_id: 2 }),
        };
        let bounds = Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 };
        let result = engine.compute(&root, bounds);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 1);
        assert_eq!(result[1].0, 2);
        assert!((result[0].1.width - 400.0).abs() < 0.01);
        assert!((result[1].1.width - 400.0).abs() < 0.01);
    }

    #[test]
    fn gap_reduces_available_space() {
        let engine = LayoutEngine { gap: 10, min_pane_size: 10.0 };
        let root = SplitNode::Split {
            direction: Direction::Horizontal,
            ratio: 0.5,
            first: Box::new(SplitNode::Leaf { pane_id: 1 }),
            second: Box::new(SplitNode::Leaf { pane_id: 2 }),
        };
        let bounds = Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 };
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
        let bounds = Rect { x: 0.0, y: 0.0, width: 800.0, height: 600.0 };
        let result = engine.compute(&root, bounds);
        assert_eq!(result.len(), 3);
    }
}
