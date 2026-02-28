//! Bloom pipeline types.

/// Per-pass uniforms for the bloom shader.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct BloomUniforms {
    /// 1.0 / texture_width, 1.0 / texture_height.
    pub texel_size: [f32; 2],
    /// Bloom brightness multiplier.
    pub intensity: f32,
    pub _padding: f32,
}

/// Bloom configuration derived from app config at pipeline creation.
#[derive(Debug, Clone, Copy)]
pub struct BloomSettings {
    /// Whether bloom is enabled.
    pub enabled: bool,
    /// Bloom brightness multiplier.
    pub intensity: f32,
    /// Number of blur passes (1-5).
    pub passes: u32,
}

impl Default for BloomSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            intensity: 0.9,
            passes: 2,
        }
    }
}

impl BloomSettings {
    /// Create bloom settings from the application config.
    pub fn from_config(config: &jarvis_config::schema::JarvisConfig) -> Self {
        Self {
            enabled: config.effects.enabled && config.effects.bloom.enabled,
            intensity: config.effects.bloom.intensity,
            passes: config.effects.bloom.passes.clamp(1, 5),
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
    fn bloom_uniforms_size_is_16_bytes() {
        assert_eq!(std::mem::size_of::<BloomUniforms>(), 16);
    }

    #[test]
    fn bloom_settings_default() {
        let s = BloomSettings::default();
        assert!(s.enabled);
        assert!((s.intensity - 0.9).abs() < f32::EPSILON);
        assert_eq!(s.passes, 2);
    }

    #[test]
    fn bloom_settings_from_config_enabled() {
        let config = jarvis_config::schema::JarvisConfig::default();
        let s = BloomSettings::from_config(&config);
        assert!(s.enabled);
        assert!((s.intensity - 0.9).abs() < f32::EPSILON);
        assert_eq!(s.passes, 2);
    }

    #[test]
    fn bloom_settings_from_config_disabled_master() {
        let mut config = jarvis_config::schema::JarvisConfig::default();
        config.effects.enabled = false;
        let s = BloomSettings::from_config(&config);
        assert!(!s.enabled);
    }

    #[test]
    fn bloom_settings_from_config_disabled_bloom() {
        let mut config = jarvis_config::schema::JarvisConfig::default();
        config.effects.bloom.enabled = false;
        let s = BloomSettings::from_config(&config);
        assert!(!s.enabled);
    }

    #[test]
    fn bloom_settings_clamps_passes() {
        let mut config = jarvis_config::schema::JarvisConfig::default();
        config.effects.bloom.passes = 99;
        let s = BloomSettings::from_config(&config);
        assert_eq!(s.passes, 5);

        config.effects.bloom.passes = 0;
        let s = BloomSettings::from_config(&config);
        assert_eq!(s.passes, 1);
    }
}
