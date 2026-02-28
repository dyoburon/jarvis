//! Layout engine types and configuration.

/// Configuration for the layout engine that computes pane positions.
pub struct LayoutEngine {
    /// Gap in pixels between panes.
    pub gap: u32,
    /// Outer padding in pixels around the entire tiling area.
    pub outer_padding: u32,
    /// Minimum size for any pane dimension.
    pub min_pane_size: f64,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self {
            gap: 6,
            outer_padding: 0,
            min_pane_size: 50.0,
        }
    }
}
