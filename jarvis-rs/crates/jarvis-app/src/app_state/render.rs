//! Frame rendering logic.

use std::collections::HashMap;

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

            // Collect dirty info first (needs &mut), then build
            // immutable grid refs for the renderer.
            let mut dirty_map: HashMap<u32, Vec<bool>> = HashMap::new();
            for (id, _) in &layout {
                if let Some(pane) = self.panes.get_mut(id) {
                    dirty_map.insert(*id, pane.vte.take_dirty());
                }
            }

            let pane_grids: Vec<(
                u32,
                jarvis_common::types::Rect,
                &jarvis_terminal::Grid,
                Vec<bool>,
            )> = layout
                .iter()
                .filter_map(|(id, rect)| {
                    let dirty = dirty_map.remove(id).unwrap_or_default();
                    self.panes
                        .get(id)
                        .map(|p| (*id, *rect, p.vte.grid(), dirty))
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
