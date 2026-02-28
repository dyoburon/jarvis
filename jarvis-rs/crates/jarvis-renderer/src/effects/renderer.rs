use super::types::EffectsConfig;

/// Renderer for visual effects, applied as post-processing passes.
///
/// The actual GPU pipeline is not yet created; this struct manages the
/// configuration and provides helper queries for the render loop.
pub struct EffectsRenderer {
    /// Current effects configuration.
    pub config: EffectsConfig,
    /// Master switch for all effects.
    pub enabled: bool,
}

impl EffectsRenderer {
    /// Create a new effects renderer with the given configuration.
    pub fn new(config: EffectsConfig) -> Self {
        Self {
            config,
            enabled: true,
        }
    }

    /// Create an effects renderer tuned for the given performance preset.
    ///
    /// - `"low"` — disables all effects.
    /// - `"medium"` — enables glow only, disables dim and scanlines.
    /// - `"high"` or `"ultra"` — enables all effects including scanlines.
    ///
    /// Any unrecognized preset is treated as `"high"`.
    pub fn from_performance_preset(preset: &str) -> Self {
        match preset {
            "low" => Self {
                config: EffectsConfig {
                    active_pane_glow: false,
                    inactive_pane_dim: false,
                    dim_opacity: 1.0,
                    glow_color: [0.0, 0.83, 1.0, 0.5],
                    glow_width: 2.0,
                    scanlines: false,
                },
                enabled: false,
            },
            "medium" => Self {
                config: EffectsConfig {
                    active_pane_glow: true,
                    inactive_pane_dim: false,
                    dim_opacity: 1.0,
                    glow_color: [0.0, 0.83, 1.0, 0.5],
                    glow_width: 2.0,
                    scanlines: false,
                },
                enabled: true,
            },
            // "high", "ultra", or anything else
            _ => Self {
                config: EffectsConfig {
                    active_pane_glow: true,
                    inactive_pane_dim: true,
                    dim_opacity: 0.6,
                    glow_color: [0.0, 0.83, 1.0, 0.5],
                    glow_width: 2.0,
                    scanlines: true,
                },
                enabled: true,
            },
        }
    }

    /// Return the opacity multiplier for a pane based on its focus state.
    ///
    /// - Focused panes always return 1.0.
    /// - Unfocused panes return `dim_opacity` when dimming is enabled,
    ///   or 1.0 when dimming is disabled or effects are globally off.
    pub fn dim_factor(&self, is_focused: bool) -> f32 {
        if is_focused {
            return 1.0;
        }
        if self.enabled && self.config.inactive_pane_dim {
            self.config.dim_opacity
        } else {
            1.0
        }
    }
}

impl Default for EffectsRenderer {
    fn default() -> Self {
        Self::new(EffectsConfig::default())
    }
}
