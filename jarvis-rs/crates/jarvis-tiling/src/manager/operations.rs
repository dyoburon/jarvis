//! Split, close, resize, and swap operations on the TilingManager.

use jarvis_common::types::{PaneId, PaneKind, Rect};

use crate::pane::Pane;
use crate::tree::Direction;

use super::TilingManager;

impl TilingManager {
    /// Choose the optimal split direction based on the focused pane's aspect ratio.
    /// Wide panes split horizontally (side-by-side), tall panes split vertically.
    pub fn auto_split_direction(&self, viewport: Rect) -> Direction {
        let layout = self.compute_layout(viewport);
        let focused_rect = layout
            .iter()
            .find(|(id, _)| *id == self.focused)
            .map(|(_, r)| r);
        match focused_rect {
            Some(r) if r.width >= r.height => Direction::Horizontal,
            Some(_) => Direction::Vertical,
            None => Direction::Horizontal,
        }
    }

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

    /// Resize the focused pane's split ratio in the given direction.
    pub fn resize(&mut self, _direction: Direction, delta: i32) -> bool {
        let delta_f = delta as f64 * 0.05; // 5% per step
        self.tree.adjust_ratio(self.focused, delta_f)
    }

    /// Swap the focused pane with its neighbor in the given direction.
    pub fn swap(&mut self, direction: Direction) -> bool {
        if let Some(neighbor) = self.tree.find_neighbor(self.focused, direction) {
            self.tree.swap_panes(self.focused, neighbor)
        } else {
            false
        }
    }
}
