//! Layout engine types and configuration.

/// Configuration for the layout engine that computes pane positions.
pub struct LayoutEngine {
    /// Gap in pixels between panes.
    pub gap: u32,
    /// Minimum size for any pane dimension.
    pub min_pane_size: f64,
}

impl Default for LayoutEngine {
    fn default() -> Self {
        Self {
            gap: 2,
            min_pane_size: 50.0,
        }
    }
}
