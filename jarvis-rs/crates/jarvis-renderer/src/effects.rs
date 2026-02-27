//! Subtle GPU visual effects.
//!
//! Stubs for glow, blur, dim, and scanline effects. These will be implemented
//! as fragment shader passes once the rendering pipeline is fully wired up.

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_values() {
        let config = EffectsConfig::default();
        assert!(config.active_pane_glow);
        assert!(config.inactive_pane_dim);
        assert!((config.dim_opacity - 0.6).abs() < 1e-6);
        assert!((config.glow_color[0] - 0.0).abs() < 1e-3);
        assert!((config.glow_color[1] - 0.83).abs() < 1e-3);
        assert!((config.glow_color[2] - 1.0).abs() < 1e-3);
        assert!((config.glow_color[3] - 0.5).abs() < 1e-3);
        assert!((config.glow_width - 2.0).abs() < 1e-6);
        assert!(!config.scanlines);
    }

    #[test]
    fn from_performance_preset_low_disables_all() {
        let renderer = EffectsRenderer::from_performance_preset("low");
        assert!(!renderer.enabled);
        assert!(!renderer.config.active_pane_glow);
        assert!(!renderer.config.inactive_pane_dim);
        assert!(!renderer.config.scanlines);
    }

    #[test]
    fn from_performance_preset_medium_enables_glow_only() {
        let renderer = EffectsRenderer::from_performance_preset("medium");
        assert!(renderer.enabled);
        assert!(renderer.config.active_pane_glow);
        assert!(!renderer.config.inactive_pane_dim);
        assert!(!renderer.config.scanlines);
    }

    #[test]
    fn from_performance_preset_high_enables_all() {
        let renderer = EffectsRenderer::from_performance_preset("high");
        assert!(renderer.enabled);
        assert!(renderer.config.active_pane_glow);
        assert!(renderer.config.inactive_pane_dim);
        assert!(renderer.config.scanlines);
    }

    #[test]
    fn from_performance_preset_ultra_enables_all() {
        let renderer = EffectsRenderer::from_performance_preset("ultra");
        assert!(renderer.enabled);
        assert!(renderer.config.active_pane_glow);
        assert!(renderer.config.inactive_pane_dim);
        assert!(renderer.config.scanlines);
    }

    #[test]
    fn from_performance_preset_unknown_defaults_to_high() {
        let renderer = EffectsRenderer::from_performance_preset("unknown");
        assert!(renderer.enabled);
        assert!(renderer.config.active_pane_glow);
        assert!(renderer.config.inactive_pane_dim);
        assert!(renderer.config.scanlines);
    }

    #[test]
    fn dim_factor_focused_returns_one() {
        let renderer = EffectsRenderer::new(EffectsConfig::default());
        assert!((renderer.dim_factor(true) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn dim_factor_unfocused_returns_dim_opacity() {
        let renderer = EffectsRenderer::new(EffectsConfig::default());
        assert!((renderer.dim_factor(false) - 0.6).abs() < 1e-6);
    }

    #[test]
    fn dim_factor_unfocused_when_disabled() {
        let renderer = EffectsRenderer::from_performance_preset("low");
        // Effects globally disabled, so unfocused should still return 1.0
        assert!((renderer.dim_factor(false) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn dim_factor_unfocused_when_dim_disabled_but_effects_on() {
        let renderer = EffectsRenderer::from_performance_preset("medium");
        // Effects enabled but dim is off
        assert!((renderer.dim_factor(false) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn default_renderer_is_enabled() {
        let renderer = EffectsRenderer::default();
        assert!(renderer.enabled);
    }
}
