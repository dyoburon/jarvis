//! Pane stacking — multiple panes occupying the same leaf position (tabs).

use serde::{Deserialize, Serialize};

/// A stack of pane IDs occupying a single leaf position. The active pane is
/// rendered on top; other panes are hidden but preserved. This enables a
/// tabbed interface within a single tiling slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneStack {
    /// Ordered list of pane IDs in this stack.
    panes: Vec<u32>,
    /// Index of the currently active (visible) pane.
    active_index: usize,
}

impl PaneStack {
    /// Create a new stack with a single pane.
    pub fn new(initial_pane_id: u32) -> Self {
        Self {
            panes: vec![initial_pane_id],
            active_index: 0,
        }
    }

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

    /// Get the active (visible) pane ID.
    pub fn active(&self) -> u32 {
        self.panes[self.active_index]
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

    /// How many panes are in this stack.
    pub fn len(&self) -> usize {
        self.panes.len()
    }

    /// Whether this stack is empty (should never be in normal operation).
    pub fn is_empty(&self) -> bool {
        self.panes.is_empty()
    }

    /// Check if a pane is in this stack.
    pub fn contains(&self, pane_id: u32) -> bool {
        self.panes.contains(&pane_id)
    }

    /// Get all pane IDs in order.
    pub fn pane_ids(&self) -> &[u32] {
        &self.panes
    }

    /// Get the active index.
    pub fn active_index(&self) -> usize {
        self.active_index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stack_has_one_pane() {
        let stack = PaneStack::new(1);
        assert_eq!(stack.len(), 1);
        assert_eq!(stack.active(), 1);
    }

    #[test]
    fn push_makes_new_pane_active() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        assert_eq!(stack.len(), 2);
        assert_eq!(stack.active(), 2);
    }

    #[test]
    fn remove_adjusts_active() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        stack.push(3);
        // Active is 3 (index 2)
        assert!(stack.remove(3));
        assert_eq!(stack.active(), 2); // Falls back to previous
    }

    #[test]
    fn remove_earlier_pane_adjusts_index() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        stack.push(3);
        // Active is 3 (index 2), remove pane 1 (index 0)
        assert!(stack.remove(1));
        assert_eq!(stack.active(), 3); // Index shifted but active stays same
        assert_eq!(stack.active_index(), 1);
    }

    #[test]
    fn remove_last_pane_fails() {
        let mut stack = PaneStack::new(1);
        assert!(!stack.remove(1));
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn remove_nonexistent_fails() {
        let mut stack = PaneStack::new(1);
        assert!(!stack.remove(99));
    }

    #[test]
    fn cycle_next_wraps() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        stack.push(3);
        stack.set_active(1);
        assert_eq!(stack.active(), 1);
        stack.cycle_next();
        assert_eq!(stack.active(), 2);
        stack.cycle_next();
        assert_eq!(stack.active(), 3);
        stack.cycle_next();
        assert_eq!(stack.active(), 1); // wrapped
    }

    #[test]
    fn cycle_prev_wraps() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        stack.push(3);
        stack.set_active(1);
        stack.cycle_prev();
        assert_eq!(stack.active(), 3); // wrapped
        stack.cycle_prev();
        assert_eq!(stack.active(), 2);
    }

    #[test]
    fn cycle_single_pane_no_change() {
        let mut stack = PaneStack::new(1);
        stack.cycle_next();
        assert_eq!(stack.active(), 1);
        stack.cycle_prev();
        assert_eq!(stack.active(), 1);
    }

    #[test]
    fn set_active_by_id() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        stack.push(3);
        assert!(stack.set_active(1));
        assert_eq!(stack.active(), 1);
        assert!(!stack.set_active(99));
    }

    #[test]
    fn contains_works() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        assert!(stack.contains(1));
        assert!(stack.contains(2));
        assert!(!stack.contains(3));
    }

    #[test]
    fn pane_ids_returns_ordered() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        stack.push(3);
        assert_eq!(stack.pane_ids(), &[1, 2, 3]);
    }

    #[test]
    fn serialization_roundtrip() {
        let mut stack = PaneStack::new(1);
        stack.push(2);
        stack.push(3);
        stack.set_active(2);
        let json = serde_json::to_string(&stack).unwrap();
        let deserialized: PaneStack = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.active(), 2);
        assert_eq!(deserialized.len(), 3);
    }
}
