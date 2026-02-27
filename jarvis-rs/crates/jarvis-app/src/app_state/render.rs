//! Frame rendering logic.

use jarvis_terminal::{Cell, Colors, Dimensions, Grid};

use super::core::JarvisApp;

impl JarvisApp {
    /// Render a single frame: collect pane grids and call the renderer.
    pub(super) fn render_frame(&mut self) {
        if let Some(ref mut rs) = self.render_state {
            let vw = rs.gpu.size.width as f32;
            let vh = rs.gpu.size.height as f32;
            let content = self.chrome.content_rect(vw, vh);
            let layout = self.tiling.compute_layout(content);
            let focused_id = self.tiling.focused_id();

            // For now, mark all rows as dirty on every frame.
            // TODO: Use alacritty's damage tracking (term.damage() / term.reset_damage())
            // to only re-render changed rows.
            let pane_grids: Vec<(
                u32,
                jarvis_common::types::Rect,
                &Grid<Cell>,
                &Colors,
                Vec<bool>,
            )> = layout
                .iter()
                .filter_map(|(id, rect)| {
                    let pane = self.panes.get(id)?;
                    let grid = pane.term.grid();
                    let colors = pane.term.colors();
                    let screen_lines = grid.screen_lines();
                    // Mark all rows dirty until we implement damage tracking.
                    let dirty = vec![true; screen_lines];
                    Some((*id, *rect, grid, colors, dirty))
                })
                .collect();

            if let Err(e) = rs.render_frame_multi(
                &pane_grids,
                focused_id,
                &self.chrome,
                self.assistant_panel.as_ref(),
            ) {
                tracing::error!("Render error: {e}");
            }
        }
    }
}
