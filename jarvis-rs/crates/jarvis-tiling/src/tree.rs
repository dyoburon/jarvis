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

    /// Split the leaf with `target_id` into two panes. The existing pane stays
    /// in the `first` position and the new pane goes in the `second` position.
    /// Returns `true` if the target was found and split.
    pub fn split_at(&mut self, target_id: u32, new_id: u32, direction: Direction) -> bool {
        match self {
            SplitNode::Leaf { pane_id } if *pane_id == target_id => {
                *self = SplitNode::Split {
                    direction,
                    ratio: 0.5,
                    first: Box::new(SplitNode::leaf(target_id)),
                    second: Box::new(SplitNode::leaf(new_id)),
                };
                true
            }
            SplitNode::Leaf { .. } => false,
            SplitNode::Split { first, second, .. } => {
                first.split_at(target_id, new_id, direction)
                    || second.split_at(target_id, new_id, direction)
            }
        }
    }

    /// Remove a pane from the tree. The sibling of the removed pane replaces
    /// the parent split. Returns `true` if the pane was found and removed.
    /// Cannot remove the last pane (when the root is a leaf).
    pub fn remove_pane(&mut self, target_id: u32) -> bool {
        match self {
            SplitNode::Leaf { .. } => false,
            SplitNode::Split { first, second, .. } => {
                // Check if target is a direct child
                if matches!(first.as_ref(), SplitNode::Leaf { pane_id } if *pane_id == target_id) {
                    *self = *second.clone();
                    return true;
                }
                if matches!(second.as_ref(), SplitNode::Leaf { pane_id } if *pane_id == target_id) {
                    *self = *first.clone();
                    return true;
                }
                // Recurse
                first.remove_pane(target_id) || second.remove_pane(target_id)
            }
        }
    }

    /// Swap two pane IDs in the tree. Both must exist for the swap to take effect.
    pub fn swap_panes(&mut self, id_a: u32, id_b: u32) -> bool {
        if !self.contains_pane(id_a) || !self.contains_pane(id_b) {
            return false;
        }
        // Use a sentinel to avoid conflicts: a -> sentinel, b -> a, sentinel -> b
        let sentinel = u32::MAX;
        self.replace_id(id_a, sentinel);
        self.replace_id(id_b, id_a);
        self.replace_id(sentinel, id_b);
        true
    }

    fn replace_id(&mut self, from: u32, to: u32) {
        match self {
            SplitNode::Leaf { pane_id } if *pane_id == from => {
                *pane_id = to;
            }
            SplitNode::Leaf { .. } => {}
            SplitNode::Split { first, second, .. } => {
                first.replace_id(from, to);
                second.replace_id(from, to);
            }
        }
    }

    /// Adjust the split ratio at the parent of the given pane.
    /// `delta` is added to the current ratio, clamped to [0.1, 0.9].
    /// Returns `true` if the pane was found in a split.
    pub fn adjust_ratio(&mut self, target_id: u32, delta: f64) -> bool {
        match self {
            SplitNode::Leaf { .. } => false,
            SplitNode::Split {
                ratio,
                first,
                second,
                ..
            } => {
                if first.contains_pane(target_id) && !second.contains_pane(target_id) {
                    // Target is in the first child of this split — this is the parent split
                    if matches!(first.as_ref(), SplitNode::Leaf { pane_id } if *pane_id == target_id)
                    {
                        *ratio = (*ratio + delta).clamp(0.1, 0.9);
                        return true;
                    }
                    // Recurse into first
                    return first.adjust_ratio(target_id, delta);
                }
                if second.contains_pane(target_id) && !first.contains_pane(target_id) {
                    if matches!(second.as_ref(), SplitNode::Leaf { pane_id } if *pane_id == target_id)
                    {
                        // Target is second child — growing second means shrinking ratio
                        *ratio = (*ratio - delta).clamp(0.1, 0.9);
                        return true;
                    }
                    return second.adjust_ratio(target_id, delta);
                }
                false
            }
        }
    }

    /// Find the neighbor of `target_id` in the given direction.
    /// For Horizontal direction, finds the pane to the right (or left if at edge).
    /// For Vertical direction, finds the pane below (or above if at edge).
    /// Returns `None` if there is no neighbor in that direction.
    pub fn find_neighbor(&self, target_id: u32, direction: Direction) -> Option<u32> {
        let ids = self.collect_pane_ids();
        let idx = ids.iter().position(|&id| id == target_id)?;

        // For directional focus, we use the ordered list and find next/prev
        // based on the tree structure. This is a simplified version that uses
        // linear ordering — a full spatial approach would need layout rects.
        match direction {
            Direction::Horizontal => {
                // Next in order
                if idx + 1 < ids.len() {
                    Some(ids[idx + 1])
                } else {
                    None
                }
            }
            Direction::Vertical => {
                // Prev in order (conceptually "up" or "down" depends on layout)
                if idx > 0 {
                    Some(ids[idx - 1])
                } else {
                    None
                }
            }
        }
    }

    /// Get the next pane ID in order after `current_id`, wrapping around.
    pub fn next_pane(&self, current_id: u32) -> Option<u32> {
        let ids = self.collect_pane_ids();
        if ids.len() <= 1 {
            return None;
        }
        let idx = ids.iter().position(|&id| id == current_id)?;
        Some(ids[(idx + 1) % ids.len()])
    }

    /// Get the previous pane ID in order before `current_id`, wrapping around.
    pub fn prev_pane(&self, current_id: u32) -> Option<u32> {
        let ids = self.collect_pane_ids();
        if ids.len() <= 1 {
            return None;
        }
        let idx = ids.iter().position(|&id| id == current_id)?;
        Some(ids[(idx + ids.len() - 1) % ids.len()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaf_count() {
        assert_eq!(SplitNode::leaf(1).pane_count(), 1);
    }

    #[test]
    fn split_count() {
        let tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert_eq!(tree.pane_count(), 2);
    }

    #[test]
    fn contains() {
        let tree = SplitNode::split_h(
            SplitNode::leaf(1),
            SplitNode::split_v(SplitNode::leaf(2), SplitNode::leaf(3)),
        );
        assert!(tree.contains_pane(1));
        assert!(tree.contains_pane(3));
        assert!(!tree.contains_pane(99));
    }

    #[test]
    fn collect_pane_ids_single() {
        let tree = SplitNode::leaf(42);
        assert_eq!(tree.collect_pane_ids(), vec![42]);
    }

    #[test]
    fn collect_pane_ids_nested() {
        let tree = SplitNode::split_h(
            SplitNode::leaf(1),
            SplitNode::split_v(SplitNode::leaf(2), SplitNode::leaf(3)),
        );
        assert_eq!(tree.collect_pane_ids(), vec![1, 2, 3]);
    }

    #[test]
    fn split_at_leaf() {
        let mut tree = SplitNode::leaf(1);
        assert!(tree.split_at(1, 2, Direction::Horizontal));
        assert_eq!(tree.pane_count(), 2);
        assert!(tree.contains_pane(1));
        assert!(tree.contains_pane(2));
    }

    #[test]
    fn split_at_nested() {
        let mut tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert!(tree.split_at(2, 3, Direction::Vertical));
        assert_eq!(tree.pane_count(), 3);
        assert_eq!(tree.collect_pane_ids(), vec![1, 2, 3]);
    }

    #[test]
    fn split_at_nonexistent() {
        let mut tree = SplitNode::leaf(1);
        assert!(!tree.split_at(99, 2, Direction::Horizontal));
        assert_eq!(tree.pane_count(), 1);
    }

    #[test]
    fn remove_pane_from_split() {
        let mut tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert!(tree.remove_pane(1));
        assert_eq!(tree.pane_count(), 1);
        assert!(tree.contains_pane(2));
        assert!(!tree.contains_pane(1));
    }

    #[test]
    fn remove_pane_nested() {
        let mut tree = SplitNode::split_h(
            SplitNode::leaf(1),
            SplitNode::split_v(SplitNode::leaf(2), SplitNode::leaf(3)),
        );
        assert!(tree.remove_pane(2));
        assert_eq!(tree.pane_count(), 2);
        assert!(tree.contains_pane(1));
        assert!(tree.contains_pane(3));
    }

    #[test]
    fn remove_last_pane_fails() {
        let mut tree = SplitNode::leaf(1);
        assert!(!tree.remove_pane(1));
    }

    #[test]
    fn swap_panes_basic() {
        let mut tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert!(tree.swap_panes(1, 2));
        assert_eq!(tree.collect_pane_ids(), vec![2, 1]);
    }

    #[test]
    fn swap_panes_nested() {
        let mut tree = SplitNode::split_h(
            SplitNode::leaf(1),
            SplitNode::split_v(SplitNode::leaf(2), SplitNode::leaf(3)),
        );
        assert!(tree.swap_panes(1, 3));
        assert_eq!(tree.collect_pane_ids(), vec![3, 2, 1]);
    }

    #[test]
    fn swap_nonexistent_fails() {
        let mut tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert!(!tree.swap_panes(1, 99));
    }

    #[test]
    fn adjust_ratio_grow_first() {
        let mut tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert!(tree.adjust_ratio(1, 0.1));
        if let SplitNode::Split { ratio, .. } = &tree {
            assert!((*ratio - 0.6).abs() < 0.001);
        } else {
            panic!("expected split");
        }
    }

    #[test]
    fn adjust_ratio_grow_second() {
        let mut tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert!(tree.adjust_ratio(2, 0.1));
        if let SplitNode::Split { ratio, .. } = &tree {
            assert!((*ratio - 0.4).abs() < 0.001);
        } else {
            panic!("expected split");
        }
    }

    #[test]
    fn adjust_ratio_clamps() {
        let mut tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        // Try to grow way beyond limit
        assert!(tree.adjust_ratio(1, 0.9));
        if let SplitNode::Split { ratio, .. } = &tree {
            assert!((*ratio - 0.9).abs() < 0.001);
        } else {
            panic!("expected split");
        }
    }

    #[test]
    fn next_pane_wraps() {
        let tree = SplitNode::split_h(
            SplitNode::leaf(1),
            SplitNode::split_v(SplitNode::leaf(2), SplitNode::leaf(3)),
        );
        assert_eq!(tree.next_pane(1), Some(2));
        assert_eq!(tree.next_pane(2), Some(3));
        assert_eq!(tree.next_pane(3), Some(1)); // wraps
    }

    #[test]
    fn prev_pane_wraps() {
        let tree = SplitNode::split_h(
            SplitNode::leaf(1),
            SplitNode::split_v(SplitNode::leaf(2), SplitNode::leaf(3)),
        );
        assert_eq!(tree.prev_pane(1), Some(3)); // wraps
        assert_eq!(tree.prev_pane(2), Some(1));
        assert_eq!(tree.prev_pane(3), Some(2));
    }

    #[test]
    fn next_prev_single_pane_returns_none() {
        let tree = SplitNode::leaf(1);
        assert_eq!(tree.next_pane(1), None);
        assert_eq!(tree.prev_pane(1), None);
    }

    #[test]
    fn find_neighbor_horizontal() {
        let tree = SplitNode::split_h(SplitNode::leaf(1), SplitNode::leaf(2));
        assert_eq!(tree.find_neighbor(1, Direction::Horizontal), Some(2));
        assert_eq!(tree.find_neighbor(2, Direction::Horizontal), None);
    }

    #[test]
    fn find_neighbor_vertical() {
        let tree = SplitNode::split_v(SplitNode::leaf(1), SplitNode::leaf(2));
        assert_eq!(tree.find_neighbor(2, Direction::Vertical), Some(1));
        assert_eq!(tree.find_neighbor(1, Direction::Vertical), None);
    }
}
