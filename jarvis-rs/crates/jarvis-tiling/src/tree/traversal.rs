//! Traversal and neighbor-finding operations on the split tree.

use super::{Direction, SplitNode};

impl SplitNode {
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
