//! Core types and constructors for TilingManager.

use std::collections::HashMap;

use jarvis_common::types::{PaneId, PaneKind};

use crate::layout::LayoutEngine;
use crate::pane::Pane;
use crate::stack::PaneStack;
use crate::tree::SplitNode;

/// Manages the entire tiling state: the split tree, the pane registry,
/// focus tracking, zoom mode, and pane stacks (tabs).
pub struct TilingManager {
    /// The root of the split tree.
    pub(super) tree: SplitNode,
    /// Registry of all panes by their numeric ID.
    pub(super) panes: HashMap<u32, Pane>,
    /// Optional stacks at leaf positions (for tabbed panes).
    pub(super) stacks: HashMap<u32, PaneStack>,
    /// The currently focused pane ID.
    pub(super) focused: u32,
    /// If `Some(id)`, that pane is zoomed to fill the viewport.
    pub(super) zoomed: Option<u32>,
    /// Layout engine configuration.
    pub(super) layout_engine: LayoutEngine,
    /// Auto-incrementing counter for pane IDs.
    pub(super) next_id: u32,
}

impl TilingManager {
    /// Create a new TilingManager with a single terminal pane.
    pub fn new() -> Self {
        let initial_id = 1;
        let pane = Pane::new_terminal(PaneId(initial_id), "Terminal");
        let mut panes = HashMap::new();
        panes.insert(initial_id, pane);

        Self {
            tree: SplitNode::leaf(initial_id),
            panes,
            stacks: HashMap::new(),
            focused: initial_id,
            zoomed: None,
            layout_engine: LayoutEngine::default(),
            next_id: 2,
        }
    }

    /// Create with a custom layout engine.
    pub fn with_layout(layout_engine: LayoutEngine) -> Self {
        let mut mgr = Self::new();
        mgr.layout_engine = layout_engine;
        mgr
    }

    // -- Accessors --

    pub fn focused_id(&self) -> u32 {
        self.focused
    }

    pub fn is_zoomed(&self) -> bool {
        self.zoomed.is_some()
    }

    pub fn zoomed_id(&self) -> Option<u32> {
        self.zoomed
    }

    pub fn pane_count(&self) -> usize {
        self.panes.len()
    }

    pub fn pane(&self, id: u32) -> Option<&Pane> {
        self.panes.get(&id)
    }

    pub fn pane_mut(&mut self, id: u32) -> Option<&mut Pane> {
        self.panes.get_mut(&id)
    }

    pub fn tree(&self) -> &SplitNode {
        &self.tree
    }

    pub fn tree_mut(&mut self) -> &mut SplitNode {
        &mut self.tree
    }

    pub fn gap(&self) -> u32 {
        self.layout_engine.gap
    }

    /// Update the gap between panes (called when settings change).
    pub fn set_gap(&mut self, gap: u32) {
        self.layout_engine.gap = gap;
    }

    /// Update the outer padding around the tiling area.
    pub fn set_outer_padding(&mut self, padding: u32) {
        self.layout_engine.outer_padding = padding;
    }

    /// Get the current outer padding.
    pub fn outer_padding(&self) -> u32 {
        self.layout_engine.outer_padding
    }

    /// Get the stack at a given leaf position, if one exists.
    pub fn stack(&self, leaf_id: u32) -> Option<&PaneStack> {
        self.stacks.get(&leaf_id)
    }

    /// Return all pane IDs that match the given `PaneKind`.
    pub fn panes_by_kind(&self, kind: PaneKind) -> Vec<u32> {
        self.panes
            .iter()
            .filter(|(_, p)| p.kind == kind)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Return pane IDs in depth-first left-to-right order (matches visual layout).
    pub fn ordered_pane_ids(&self) -> Vec<u32> {
        self.tree.collect_pane_ids()
    }
}

impl Default for TilingManager {
    fn default() -> Self {
        Self::new()
    }
}
