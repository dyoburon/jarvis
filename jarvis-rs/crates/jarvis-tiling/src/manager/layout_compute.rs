//! Layout computation and command dispatch for TilingManager.

use jarvis_common::types::Rect;

use crate::commands::TilingCommand;
use crate::tree::Direction;

use super::TilingManager;

impl TilingManager {
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
