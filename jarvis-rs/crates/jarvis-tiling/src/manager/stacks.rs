//! Stack (tab) operations for TilingManager.

use jarvis_common::types::{PaneId, PaneKind};

use crate::pane::Pane;
use crate::stack::PaneStack;

use super::TilingManager;

impl TilingManager {
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
}
