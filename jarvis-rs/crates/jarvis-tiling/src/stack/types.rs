//! Core types for pane stacking (tabs).

use serde::{Deserialize, Serialize};

/// A stack of pane IDs occupying a single leaf position. The active pane is
/// rendered on top; other panes are hidden but preserved. This enables a
/// tabbed interface within a single tiling slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaneStack {
    /// Ordered list of pane IDs in this stack.
    pub(super) panes: Vec<u32>,
    /// Index of the currently active (visible) pane.
    pub(super) active_index: usize,
}

impl PaneStack {
    /// Create a new stack with a single pane.
    pub fn new(initial_pane_id: u32) -> Self {
        Self {
            panes: vec![initial_pane_id],
            active_index: 0,
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

    /// Get the active (visible) pane ID.
    pub fn active(&self) -> u32 {
        self.panes[self.active_index]
    }
}
