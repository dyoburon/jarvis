//! Core types for the split tree: Direction and SplitNode.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SplitNode {
    Leaf {
        pane_id: u32,
    },
    Split {
        direction: Direction,
        ratio: f64,
        first: Box<SplitNode>,
        second: Box<SplitNode>,
    },
}

impl SplitNode {
    pub fn leaf(pane_id: u32) -> Self {
        SplitNode::Leaf { pane_id }
    }

    pub fn split_h(first: SplitNode, second: SplitNode) -> Self {
        SplitNode::Split {
            direction: Direction::Horizontal,
            ratio: 0.5,
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    pub fn split_v(first: SplitNode, second: SplitNode) -> Self {
        SplitNode::Split {
            direction: Direction::Vertical,
            ratio: 0.5,
            first: Box::new(first),
            second: Box::new(second),
        }
    }

    pub fn pane_count(&self) -> usize {
        match self {
            SplitNode::Leaf { .. } => 1,
            SplitNode::Split { first, second, .. } => first.pane_count() + second.pane_count(),
        }
    }

    pub fn contains_pane(&self, id: u32) -> bool {
        match self {
            SplitNode::Leaf { pane_id } => *pane_id == id,
            SplitNode::Split { first, second, .. } => {
                first.contains_pane(id) || second.contains_pane(id)
            }
        }
    }

    /// Collect all pane IDs in left-to-right (depth-first) order.
    pub fn collect_pane_ids(&self) -> Vec<u32> {
        let mut ids = Vec::new();
        self.collect_ids_into(&mut ids);
        ids
    }

    fn collect_ids_into(&self, out: &mut Vec<u32>) {
        match self {
            SplitNode::Leaf { pane_id } => out.push(*pane_id),
            SplitNode::Split { first, second, .. } => {
                first.collect_ids_into(out);
                second.collect_ids_into(out);
            }
        }
    }
}
