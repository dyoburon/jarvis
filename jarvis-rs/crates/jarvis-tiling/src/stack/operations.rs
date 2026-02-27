//! Mutating operations on PaneStack: push, remove, cycle, set_active.

use super::PaneStack;

impl PaneStack {
    /// Push a new pane onto the stack and make it active.
    pub fn push(&mut self, pane_id: u32) {
        self.panes.push(pane_id);
        self.active_index = self.panes.len() - 1;
    }

    /// Remove a pane from the stack by ID. Returns `true` if found.
    /// If the active pane is removed, the previous pane becomes active.
    /// Returns `false` if the pane is not in the stack or it's the last one.
    pub fn remove(&mut self, pane_id: u32) -> bool {
        if self.panes.len() <= 1 {
            return false;
        }
        if let Some(idx) = self.panes.iter().position(|&id| id == pane_id) {
            self.panes.remove(idx);
            if self.active_index >= self.panes.len() {
                self.active_index = self.panes.len() - 1;
            } else if idx < self.active_index {
                self.active_index -= 1;
            }
            true
        } else {
            false
        }
    }

    /// Cycle to the next tab, wrapping around.
    pub fn cycle_next(&mut self) {
        if self.panes.len() > 1 {
            self.active_index = (self.active_index + 1) % self.panes.len();
        }
    }

    /// Cycle to the previous tab, wrapping around.
    pub fn cycle_prev(&mut self) {
        if self.panes.len() > 1 {
            self.active_index = (self.active_index + self.panes.len() - 1) % self.panes.len();
        }
    }

    /// Set a specific pane as active by ID. Returns `true` if found.
    pub fn set_active(&mut self, pane_id: u32) -> bool {
        if let Some(idx) = self.panes.iter().position(|&id| id == pane_id) {
            self.active_index = idx;
            true
        } else {
            false
        }
    }
}
