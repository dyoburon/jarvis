//! The TilingManager coordinates tree layout, panes, focus, and zoom.

mod focus;
mod layout_compute;
mod operations;
mod stacks;
mod types;

pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::TilingCommand;
    use crate::layout::LayoutEngine;
    use crate::tree::Direction;
    use jarvis_common::types::{PaneKind, Rect};

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

    // -- Stack (tab) tests --

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
