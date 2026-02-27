//! The TilingManager coordinates tree layout, panes, focus, and zoom.

use std::collections::HashMap;

use jarvis_common::types::{PaneId, PaneKind, Rect};

use crate::commands::TilingCommand;
use crate::layout::LayoutEngine;
use crate::pane::Pane;
use crate::stack::PaneStack;
use crate::tree::{Direction, SplitNode};

/// Manages the entire tiling state: the split tree, the pane registry,
/// focus tracking, zoom mode, and pane stacks (tabs).
pub struct TilingManager {
    /// The root of the split tree.
    tree: SplitNode,
    /// Registry of all panes by their numeric ID.
    panes: HashMap<u32, Pane>,
    /// Optional stacks at leaf positions (for tabbed panes).
    stacks: HashMap<u32, PaneStack>,
    /// The currently focused pane ID.
    focused: u32,
    /// If `Some(id)`, that pane is zoomed to fill the viewport.
    zoomed: Option<u32>,
    /// Layout engine configuration.
    layout_engine: LayoutEngine,
    /// Auto-incrementing counter for pane IDs.
    next_id: u32,
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

    // -----------------------------------------------------------------------
    // Accessors
    // -----------------------------------------------------------------------

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

    pub fn tree(&self) -> &SplitNode {
        &self.tree
    }

    /// Get the stack at a given leaf position, if one exists.
    pub fn stack(&self, leaf_id: u32) -> Option<&PaneStack> {
        self.stacks.get(&leaf_id)
    }

    // -----------------------------------------------------------------------
    // Command dispatch
    // -----------------------------------------------------------------------

    /// Execute a tiling command. Returns `true` if the command was handled.
    pub fn execute(&mut self, cmd: TilingCommand) -> bool {
        match cmd {
            TilingCommand::SplitHorizontal => self.split(Direction::Horizontal),
            TilingCommand::SplitVertical => self.split(Direction::Vertical),
            TilingCommand::Close => self.close_focused(),
            TilingCommand::Resize(dir, delta) => self.resize(dir, delta),
            TilingCommand::Swap(dir) => self.swap(dir),
            TilingCommand::FocusNext => self.focus_next(),
            TilingCommand::FocusPrev => self.focus_prev(),
            TilingCommand::FocusDirection(dir) => self.focus_direction(dir),
            TilingCommand::Zoom => self.zoom_toggle(),
        }
    }

    // -----------------------------------------------------------------------
    // Split
    // -----------------------------------------------------------------------

    /// Split the focused pane, creating a new terminal pane.
    pub fn split(&mut self, direction: Direction) -> bool {
        // Unzoom first
        self.zoomed = None;

        let new_id = self.next_id;
        self.next_id += 1;

        if self.tree.split_at(self.focused, new_id, direction) {
            let pane = Pane::new_terminal(PaneId(new_id), "Terminal");
            self.panes.insert(new_id, pane);
            self.focused = new_id;
            true
        } else {
            false
        }
    }

    /// Split the focused pane with a specific kind and title.
    pub fn split_with(
        &mut self,
        direction: Direction,
        kind: PaneKind,
        title: impl Into<String>,
    ) -> Option<u32> {
        self.zoomed = None;
        let new_id = self.next_id;
        self.next_id += 1;

        if self.tree.split_at(self.focused, new_id, direction) {
            let pane = Pane {
                id: PaneId(new_id),
                kind,
                title: title.into(),
            };
            self.panes.insert(new_id, pane);
            self.focused = new_id;
            Some(new_id)
        } else {
            None
        }
    }

    // -----------------------------------------------------------------------
    // Close
    // -----------------------------------------------------------------------

    /// Close the focused pane. If it's the last pane, returns `false`.
    pub fn close_focused(&mut self) -> bool {
        if self.panes.len() <= 1 {
            return false;
        }

        let to_close = self.focused;
        self.zoomed = None;

        // Move focus before removing
        if let Some(next) = self.tree.next_pane(to_close) {
            self.focused = next;
        }

        if self.tree.remove_pane(to_close) {
            self.panes.remove(&to_close);
            self.stacks.remove(&to_close);
            true
        } else {
            false
        }
    }

    /// Close a specific pane by ID.
    pub fn close_pane(&mut self, id: u32) -> bool {
        if self.panes.len() <= 1 {
            return false;
        }

        if id == self.focused {
            return self.close_focused();
        }

        if self.tree.remove_pane(id) {
            self.panes.remove(&id);
            self.stacks.remove(&id);
            if self.zoomed == Some(id) {
                self.zoomed = None;
            }
            true
        } else {
            false
        }
    }

    // -----------------------------------------------------------------------
    // Resize
    // -----------------------------------------------------------------------

    /// Resize the focused pane's split ratio in the given direction.
    pub fn resize(&mut self, _direction: Direction, delta: i32) -> bool {
        let delta_f = delta as f64 * 0.05; // 5% per step
        self.tree.adjust_ratio(self.focused, delta_f)
    }

    // -----------------------------------------------------------------------
    // Swap
    // -----------------------------------------------------------------------

    /// Swap the focused pane with its neighbor in the given direction.
    pub fn swap(&mut self, direction: Direction) -> bool {
        if let Some(neighbor) = self.tree.find_neighbor(self.focused, direction) {
            self.tree.swap_panes(self.focused, neighbor)
        } else {
            false
        }
    }

    // -----------------------------------------------------------------------
    // Focus
    // -----------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // Zoom
    // -----------------------------------------------------------------------

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

    // -----------------------------------------------------------------------
    // Stacks (tabs)
    // -----------------------------------------------------------------------

    /// Add a pane to the stack at the focused leaf position.
    pub fn push_to_stack(&mut self, kind: PaneKind, title: impl Into<String>) -> u32 {
        let new_id = self.next_id;
        self.next_id += 1;

        let pane = Pane {
            id: PaneId(new_id),
            kind,
            title: title.into(),
        };
        self.panes.insert(new_id, pane);

        let stack = self
            .stacks
            .entry(self.focused)
            .or_insert_with(|| PaneStack::new(self.focused));
        stack.push(new_id);

        new_id
    }

    /// Cycle to the next tab in the focused pane's stack.
    pub fn cycle_stack_next(&mut self) -> bool {
        if let Some(stack) = self.stacks.get_mut(&self.focused) {
            stack.cycle_next();
            true
        } else {
            false
        }
    }

    /// Cycle to the previous tab in the focused pane's stack.
    pub fn cycle_stack_prev(&mut self) -> bool {
        if let Some(stack) = self.stacks.get_mut(&self.focused) {
            stack.cycle_prev();
            true
        } else {
            false
        }
    }

    // -----------------------------------------------------------------------
    // Layout computation
    // -----------------------------------------------------------------------

    /// Compute the layout for all panes within the given viewport.
    /// If a pane is zoomed, it fills the entire viewport.
    pub fn compute_layout(&self, viewport: Rect) -> Vec<(u32, Rect)> {
        if let Some(zoomed_id) = self.zoomed {
            // Zoomed pane fills the whole viewport
            vec![(zoomed_id, viewport)]
        } else {
            self.layout_engine.compute(&self.tree, viewport)
        }
    }
}

impl Default for TilingManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn viewport() -> Rect {
        Rect {
            x: 0.0,
            y: 0.0,
            width: 1920.0,
            height: 1080.0,
        }
    }

    #[test]
    fn new_manager_has_one_pane() {
        let mgr = TilingManager::new();
        assert_eq!(mgr.pane_count(), 1);
        assert_eq!(mgr.focused_id(), 1);
        assert!(!mgr.is_zoomed());
    }

    #[test]
    fn split_horizontal_creates_two_panes() {
        let mut mgr = TilingManager::new();
        assert!(mgr.split(Direction::Horizontal));
        assert_eq!(mgr.pane_count(), 2);
        // Focus moves to new pane
        assert_eq!(mgr.focused_id(), 2);
    }

    #[test]
    fn split_vertical_creates_two_panes() {
        let mut mgr = TilingManager::new();
        assert!(mgr.split(Direction::Vertical));
        assert_eq!(mgr.pane_count(), 2);
    }

    #[test]
    fn close_focused_reduces_panes() {
        let mut mgr = TilingManager::new();
        mgr.split(Direction::Horizontal);
        assert_eq!(mgr.pane_count(), 2);
        assert!(mgr.close_focused());
        assert_eq!(mgr.pane_count(), 1);
    }

    #[test]
    fn close_last_pane_fails() {
        let mut mgr = TilingManager::new();
        assert!(!mgr.close_focused());
        assert_eq!(mgr.pane_count(), 1);
    }

    #[test]
    fn close_specific_pane() {
        let mut mgr = TilingManager::new();
        mgr.split(Direction::Horizontal);
        // Panes: 1, 2. Focused: 2
        assert!(mgr.close_pane(1));
        assert_eq!(mgr.pane_count(), 1);
        assert_eq!(mgr.focused_id(), 2);
    }

    #[test]
    fn focus_next_and_prev() {
        let mut mgr = TilingManager::new();
        mgr.split(Direction::Horizontal);
        // Focus is on 2
        assert_eq!(mgr.focused_id(), 2);
        assert!(mgr.focus_next());
        assert_eq!(mgr.focused_id(), 1); // wraps
        assert!(mgr.focus_prev());
        assert_eq!(mgr.focused_id(), 2);
    }

    #[test]
    fn focus_direction() {
        let mut mgr = TilingManager::new();
        mgr.split(Direction::Horizontal);
        mgr.focus_pane(1);
        assert!(mgr.focus_direction(Direction::Horizontal));
        assert_eq!(mgr.focused_id(), 2);
    }

    #[test]
    fn focus_pane_by_id() {
        let mut mgr = TilingManager::new();
        mgr.split(Direction::Horizontal);
        assert!(mgr.focus_pane(1));
        assert_eq!(mgr.focused_id(), 1);
        assert!(!mgr.focus_pane(99));
    }

    #[test]
    fn zoom_toggle() {
        let mut mgr = TilingManager::new();
        mgr.split(Direction::Horizontal);
        mgr.focus_pane(1);
        assert!(!mgr.is_zoomed());

        assert!(mgr.zoom_toggle());
        assert!(mgr.is_zoomed());
        assert_eq!(mgr.zoomed_id(), Some(1));

        // Toggle off
        assert!(mgr.zoom_toggle());
        assert!(!mgr.is_zoomed());
    }

    #[test]
    fn zoom_single_pane_fails() {
        let mut mgr = TilingManager::new();
        assert!(!mgr.zoom_toggle());
    }

    #[test]
    fn layout_normal() {
        let mgr = TilingManager::new();
        let layout = mgr.compute_layout(viewport());
        assert_eq!(layout.len(), 1);
        assert_eq!(layout[0].0, 1);
        assert!((layout[0].1.width - 1920.0).abs() < 0.01);
    }

    #[test]
    fn layout_split() {
        let mut mgr = TilingManager::with_layout(LayoutEngine {
            gap: 0,
            min_pane_size: 10.0,
        });
        mgr.split(Direction::Horizontal);
        let layout = mgr.compute_layout(viewport());
        assert_eq!(layout.len(), 2);
        // Each should be roughly half width
        assert!((layout[0].1.width - 960.0).abs() < 0.01);
        assert!((layout[1].1.width - 960.0).abs() < 0.01);
    }

    #[test]
    fn layout_zoomed() {
        let mut mgr = TilingManager::new();
        mgr.split(Direction::Horizontal);
        mgr.focus_pane(1);
        mgr.zoom_toggle();

        let layout = mgr.compute_layout(viewport());
        assert_eq!(layout.len(), 1);
        assert_eq!(layout[0].0, 1);
        assert!((layout[0].1.width - 1920.0).abs() < 0.01);
    }

    #[test]
    fn resize_adjusts_ratio() {
        let mut mgr = TilingManager::new();
        mgr.split(Direction::Horizontal);
        mgr.focus_pane(1);
        assert!(mgr.resize(Direction::Horizontal, 2)); // +10%
    }

    #[test]
    fn swap_with_neighbor() {
        let mut mgr = TilingManager::new();
        mgr.split(Direction::Horizontal);
        mgr.focus_pane(1);
        assert!(mgr.swap(Direction::Horizontal));
        // After swap, pane order should be reversed in tree
        let ids = mgr.tree().collect_pane_ids();
        assert_eq!(ids, vec![2, 1]);
    }

    #[test]
    fn execute_command_dispatch() {
        let mut mgr = TilingManager::new();
        assert!(mgr.execute(TilingCommand::SplitHorizontal));
        assert_eq!(mgr.pane_count(), 2);
        assert!(mgr.execute(TilingCommand::FocusNext));
        assert!(mgr.execute(TilingCommand::SplitVertical));
        assert_eq!(mgr.pane_count(), 3);
        assert!(mgr.execute(TilingCommand::Close));
        assert_eq!(mgr.pane_count(), 2);
    }

    #[test]
    fn split_with_custom_kind() {
        let mut mgr = TilingManager::new();
        let id = mgr.split_with(Direction::Horizontal, PaneKind::WebView, "Browser");
        assert!(id.is_some());
        let id = id.unwrap();
        let pane = mgr.pane(id).unwrap();
        assert_eq!(pane.kind, PaneKind::WebView);
        assert_eq!(pane.title, "Browser");
    }

    #[test]
    fn split_unzooms() {
        let mut mgr = TilingManager::new();
        mgr.split(Direction::Horizontal);
        mgr.focus_pane(1);
        mgr.zoom_toggle();
        assert!(mgr.is_zoomed());
        mgr.split(Direction::Vertical);
        assert!(!mgr.is_zoomed());
    }

    #[test]
    fn multiple_splits_and_closes() {
        let mut mgr = TilingManager::new();
        // Create a 4-pane layout
        mgr.split(Direction::Horizontal); // 1 | 2
        mgr.focus_pane(1);
        mgr.split(Direction::Vertical); // 1/3 | 2
        mgr.focus_pane(2);
        mgr.split(Direction::Vertical); // 1/3 | 2/4
        assert_eq!(mgr.pane_count(), 4);

        // Close all but one
        mgr.focus_pane(4);
        mgr.close_focused();
        mgr.focus_pane(3);
        mgr.close_focused();
        mgr.close_focused();
        assert_eq!(mgr.pane_count(), 1);
    }

    // -----------------------------------------------------------------------
    // Stack (tab) tests
    // -----------------------------------------------------------------------

    #[test]
    fn push_to_stack_creates_tab() {
        let mut mgr = TilingManager::new();
        let tab_id = mgr.push_to_stack(PaneKind::Terminal, "Tab 2");
        assert_eq!(mgr.pane_count(), 2);
        let stack = mgr.stack(1).unwrap();
        assert_eq!(stack.len(), 2);
        assert!(stack.contains(tab_id));
    }

    #[test]
    fn cycle_stack() {
        let mut mgr = TilingManager::new();
        mgr.push_to_stack(PaneKind::Terminal, "Tab 2");
        mgr.push_to_stack(PaneKind::Terminal, "Tab 3");
        mgr.focus_pane(1);
        let stack = mgr.stack(1).unwrap();
        assert_eq!(stack.active(), 3); // last pushed is active
        assert!(mgr.cycle_stack_next());
    }

    #[test]
    fn default_impl() {
        let mgr = TilingManager::default();
        assert_eq!(mgr.pane_count(), 1);
    }
}
