//! Mutating operations on the split tree: split, remove, swap, adjust ratio.

use super::{Direction, SplitNode};

impl SplitNode {
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
    pub fn swap_panes(&mut self, a: u32, b: u32) -> bool {
        let mut found_a = false;
        let mut found_b = false;
        self.for_each_leaf_mut(&mut |id: &mut u32| {
            if *id == a {
                *id = b;
                found_a = true;
            } else if *id == b {
                *id = a;
                found_b = true;
            }
        });
        found_a && found_b
    }

    fn for_each_leaf_mut(&mut self, f: &mut impl FnMut(&mut u32)) {
        match self {
            SplitNode::Leaf { pane_id } => f(pane_id),
            SplitNode::Split { first, second, .. } => {
                first.for_each_leaf_mut(f);
                second.for_each_leaf_mut(f);
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
}
