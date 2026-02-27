/// Configuration for visual effects applied to panes and the terminal grid.
#[derive(Debug, Clone)]
pub struct EffectsConfig {
    /// Draw a colored glow around the active (focused) pane.
    pub active_pane_glow: bool,
    /// Dim inactive (unfocused) panes.
    pub inactive_pane_dim: bool,
    /// Opacity multiplier for inactive panes (0.0 = fully transparent, 1.0 = opaque).
    pub dim_opacity: f32,
    /// RGBA color of the glow effect around the active pane.
    pub glow_color: [f32; 4],
    /// Width of the glow effect in pixels.
    pub glow_width: f32,
    /// Enable CRT-style scanline overlay.
    pub scanlines: bool,
}

impl Default for EffectsConfig {
    fn default() -> Self {
        Self {
            active_pane_glow: true,
            inactive_pane_dim: true,
            dim_opacity: 0.6,
            glow_color: [0.0, 0.83, 1.0, 0.5], // cyan-ish, semi-transparent
            glow_width: 2.0,
            scanlines: false,
        }
    }
}
