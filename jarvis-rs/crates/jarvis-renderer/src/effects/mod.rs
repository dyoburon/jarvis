//! Subtle GPU visual effects.
//!
//! Stubs for glow, blur, dim, and scanline effects. These will be implemented
//! as fragment shader passes once the rendering pipeline is fully wired up.

mod renderer;
mod types;

pub use renderer::*;
pub use types::*;

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

    #[test]
    fn from_config_default_maps_all_fields() {
        let config = jarvis_config::schema::JarvisConfig::default();
        let renderer = EffectsRenderer::from_config(&config);
        assert!(renderer.enabled);
        assert!(renderer.config.active_pane_glow);
        assert!(renderer.config.inactive_pane_dim);
        assert!((renderer.config.dim_opacity - 0.6).abs() < 1e-6);
        assert!(renderer.config.scanlines);
        assert!((renderer.config.glow_width - 2.0).abs() < 1e-6);
        // Default glow color #ffcc66 → normalized [1.0, 0.8, 0.4, 0.5]
        assert!((renderer.config.glow_color[0] - 1.0).abs() < 1e-3); // R = 1.0
        assert!((renderer.config.glow_color[1] - 0.8).abs() < 1e-3); // G ≈ 0.8
        assert!((renderer.config.glow_color[2] - 0.4).abs() < 1e-3); // B ≈ 0.4
        assert!((renderer.config.glow_color[3] - 0.5).abs() < 1e-3);
    }

    #[test]
    fn from_config_disabled_master_disables_all() {
        let mut config = jarvis_config::schema::JarvisConfig::default();
        config.effects.enabled = false;
        let renderer = EffectsRenderer::from_config(&config);
        assert!(!renderer.enabled);
        assert!(!renderer.config.active_pane_glow);
        assert!(!renderer.config.inactive_pane_dim);
        assert!(!renderer.config.scanlines);
    }

    #[test]
    fn from_config_invalid_glow_color_uses_fallback() {
        let mut config = jarvis_config::schema::JarvisConfig::default();
        config.effects.glow.color = "not-a-color".into();
        let renderer = EffectsRenderer::from_config(&config);
        // Fallback is cyan-ish [0.0, 0.83, 1.0, 0.5]
        assert!((renderer.config.glow_color[0] - 0.0).abs() < 1e-3);
        assert!((renderer.config.glow_color[1] - 0.83).abs() < 1e-3);
        assert!((renderer.config.glow_color[2] - 1.0).abs() < 1e-3);
        assert!((renderer.config.glow_color[3] - 0.5).abs() < 1e-3);
    }
}
