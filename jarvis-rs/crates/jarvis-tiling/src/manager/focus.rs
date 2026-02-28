//! Focus and zoom handling for TilingManager.

use crate::tree::Direction;

use super::TilingManager;

impl TilingManager {
    /// Focus the next pane in order.
    pub fn focus_next(&mut self) -> bool {
        if let Some(next) = self.tree.next_pane(self.focused) {
            self.focused = next;
            true
        } else {
            false
        }
    }

    /// Focus the previous pane in order.
    pub fn focus_prev(&mut self) -> bool {
        if let Some(prev) = self.tree.prev_pane(self.focused) {
            self.focused = prev;
            true
        } else {
            false
        }
    }

    /// Focus the neighbor in a specific direction.
    pub fn focus_direction(&mut self, direction: Direction) -> bool {
        if let Some(neighbor) = self.tree.find_neighbor(self.focused, direction) {
            self.focused = neighbor;
            true
        } else {
            false
        }
    }

    /// Set focus to a specific pane by ID.
    pub fn focus_pane(&mut self, id: u32) -> bool {
        if self.panes.contains_key(&id) {
            self.focused = id;
            true
        } else {
            false
        }
    }

    /// Toggle zoom on the focused pane.
    pub fn zoom_toggle(&mut self) -> bool {
        if self.panes.len() <= 1 {
            return false;
        }
        if self.zoomed == Some(self.focused) {
            self.zoomed = None;
        } else {
            self.zoomed = Some(self.focused);
        }
        true
    }
}
