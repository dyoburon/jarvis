//! Post-processing effects configuration types.
//!
//! Controls CRT scanlines, vignette, bloom, and glow effects.
//! All effects are optional and can be toggled individually or
//! disabled entirely with `enabled = false`.

use serde::{Deserialize, Serialize};

/// CRT-style scanline overlay settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct ScanlineConfig {
    pub enabled: bool,
    /// Scanline darkness intensity (valid range: 0.0-1.0).
    pub intensity: f32,
}

impl Default for ScanlineConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            intensity: 0.08,
        }
    }
}

/// Screen-edge darkening effect settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct VignetteConfig {
    pub enabled: bool,
    /// Vignette strength (valid range: 0.0-3.0).
    pub intensity: f32,
}

impl Default for VignetteConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            intensity: 1.2,
        }
    }
}

/// Bloom (light bleed) effect settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct BloomConfig {
    pub enabled: bool,
    /// Bloom brightness multiplier (valid range: 0.0-3.0).
    pub intensity: f32,
    /// Number of blur passes (valid range: 1-5). More passes = smoother bloom.
    pub passes: u32,
}

impl Default for BloomConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            intensity: 0.9,
            passes: 2,
        }
    }
}

/// Glow effect around the active pane.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct GlowConfig {
    pub enabled: bool,
    /// Glow color as hex string.
    pub color: String,
    /// Glow width in pixels (valid range: 0.0-10.0).
    pub width: f32,
    /// Focus glow intensity for CSS box-shadow (valid range: 0.0-1.0).
    pub intensity: f64,
}

impl Default for GlowConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            color: "#cba6f7".into(),
            width: 2.0,
            intensity: 0.0,
        }
    }
}

/// Brightness flicker effect settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct FlickerConfig {
    pub enabled: bool,
    /// Flicker amplitude (valid range: 0.0-0.05).
    pub amplitude: f32,
}

impl Default for FlickerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            amplitude: 0.004,
        }
    }
}

/// Master effects configuration.
///
/// Controls all post-processing effects. Set `enabled = false` to
/// disable all effects regardless of individual settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct EffectsSchemaConfig {
    /// Master toggle — disables all effects when false.
    pub enabled: bool,
    /// Dim inactive (unfocused) panes.
    pub inactive_pane_dim: bool,
    /// Opacity multiplier for inactive panes (valid range: 0.0-1.0).
    pub dim_opacity: f32,
    pub scanlines: ScanlineConfig,
    pub vignette: VignetteConfig,
    pub bloom: BloomConfig,
    pub glow: GlowConfig,
    pub flicker: FlickerConfig,
    /// CRT barrel distortion (future — currently no-op).
    pub crt_curvature: bool,
    /// Backdrop blur radius in pixels for glassmorphic panels (valid range: 0-40).
    pub blur_radius: u32,
    /// Backdrop saturate multiplier for glassmorphic panels (valid range: 0.0-2.0).
    pub saturate: f64,
    /// CSS transition speed in milliseconds (valid range: 0-500).
    pub transition_speed: u32,
}

impl Default for EffectsSchemaConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            inactive_pane_dim: true,
            dim_opacity: 0.6,
            scanlines: ScanlineConfig::default(),
            vignette: VignetteConfig::default(),
            bloom: BloomConfig::default(),
            glow: GlowConfig::default(),
            flicker: FlickerConfig::default(),
            crt_curvature: false,
            blur_radius: 12,
            saturate: 1.1,
            transition_speed: 150,
        }
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effects_config_defaults() {
        let config = EffectsSchemaConfig::default();
        assert!(config.enabled);
        assert!(config.inactive_pane_dim);
        assert!((config.dim_opacity - 0.6).abs() < f32::EPSILON);
        assert!(config.scanlines.enabled);
        assert!((config.scanlines.intensity - 0.08).abs() < f32::EPSILON);
        assert!(config.vignette.enabled);
        assert!((config.vignette.intensity - 1.2).abs() < f32::EPSILON);
        assert!(config.bloom.enabled);
        assert!((config.bloom.intensity - 0.9).abs() < f32::EPSILON);
        assert_eq!(config.bloom.passes, 2);
        assert!(config.glow.enabled);
        assert_eq!(config.glow.color, "#cba6f7");
        assert!((config.glow.width - 2.0).abs() < f32::EPSILON);
        assert!((config.glow.intensity - 0.0).abs() < f64::EPSILON);
        assert!(config.flicker.enabled);
        assert!((config.flicker.amplitude - 0.004).abs() < f32::EPSILON);
        assert!(!config.crt_curvature);
        assert_eq!(config.blur_radius, 12);
        assert!((config.saturate - 1.1).abs() < f64::EPSILON);
        assert_eq!(config.transition_speed, 150);
    }

    #[test]
    fn scanline_config_partial_toml() {
        let toml_str = r#"
enabled = false
intensity = 0.15
"#;
        let config: ScanlineConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.enabled);
        assert!((config.intensity - 0.15).abs() < f32::EPSILON);
    }

    #[test]
    fn bloom_config_partial_toml() {
        let toml_str = r#"
intensity = 1.5
passes = 4
"#;
        let config: BloomConfig = toml::from_str(toml_str).unwrap();
        assert!(config.enabled); // default preserved
        assert!((config.intensity - 1.5).abs() < f32::EPSILON);
        assert_eq!(config.passes, 4);
    }

    #[test]
    fn glow_config_partial_toml() {
        let toml_str = r##"
color = "#ff6b00"
width = 4.0
intensity = 0.3
"##;
        let config: GlowConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.color, "#ff6b00");
        assert!((config.width - 4.0).abs() < f32::EPSILON);
        assert!((config.intensity - 0.3).abs() < f64::EPSILON);
        assert!(config.enabled); // default preserved
    }

    #[test]
    fn effects_master_toggle_in_toml() {
        let toml_str = r#"
enabled = false
"#;
        let config: EffectsSchemaConfig = toml::from_str(toml_str).unwrap();
        assert!(!config.enabled);
        // Sub-configs still have their defaults
        assert!(config.scanlines.enabled);
        assert!(config.bloom.enabled);
    }

    #[test]
    fn effects_full_toml() {
        let toml_str = r##"
enabled = true
inactive_pane_dim = false

[scanlines]
enabled = true
intensity = 0.12

[vignette]
enabled = false

[bloom]
enabled = true
intensity = 1.2
passes = 3

[glow]
color = "#ff0000"
width = 3.0

[flicker]
enabled = false
"##;
        let config: EffectsSchemaConfig = toml::from_str(toml_str).unwrap();
        assert!(config.enabled);
        assert!(!config.inactive_pane_dim);
        assert!((config.scanlines.intensity - 0.12).abs() < f32::EPSILON);
        assert!(!config.vignette.enabled);
        assert_eq!(config.bloom.passes, 3);
        assert_eq!(config.glow.color, "#ff0000");
        assert!(!config.flicker.enabled);
    }

    #[test]
    fn effects_glassmorphic_fields_in_toml() {
        let toml_str = r#"
blur_radius = 30
saturate = 1.5
transition_speed = 200
"#;
        let config: EffectsSchemaConfig = toml::from_str(toml_str).unwrap();
        assert_eq!(config.blur_radius, 30);
        assert!((config.saturate - 1.5).abs() < f64::EPSILON);
        assert_eq!(config.transition_speed, 200);
        // Defaults preserved
        assert!(config.enabled);
        assert!(config.bloom.enabled);
    }

    #[test]
    fn effects_serialization_roundtrip() {
        let config = EffectsSchemaConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: EffectsSchemaConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.enabled, config.enabled);
        assert_eq!(deserialized.bloom.passes, config.bloom.passes);
        assert_eq!(deserialized.glow.color, config.glow.color);
        assert_eq!(deserialized.blur_radius, config.blur_radius);
        assert_eq!(deserialized.transition_speed, config.transition_speed);
    }
}
