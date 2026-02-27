//! Glyph atlas management.
//!
//! A thin wrapper around atlas configuration for the glyph rasterization
//! pipeline. The actual atlas texture is managed by `glyphon` at runtime;
//! this module provides sizing defaults and configuration.

/// Configuration for the glyph atlas texture.
#[derive(Debug, Clone, Copy)]
pub struct AtlasConfig {
    /// Initial atlas texture size in pixels (width and height).
    pub initial_size: u32,
    /// Maximum atlas texture size the GPU is allowed to grow to.
    pub max_size: u32,
}

impl Default for AtlasConfig {
    fn default() -> Self {
        default_atlas_config()
    }
}

/// Returns an `AtlasConfig` with sensible defaults.
///
/// - `initial_size`: 512 (covers most common glyph sets)
/// - `max_size`: 4096 (typical GPU texture limit for atlas usage)
pub fn default_atlas_config() -> AtlasConfig {
    AtlasConfig {
        initial_size: 512,
        max_size: 4096,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_atlas_config_values() {
        let config = default_atlas_config();
        assert_eq!(config.initial_size, 512);
        assert_eq!(config.max_size, 4096);
    }

    #[test]
    fn atlas_config_default_trait() {
        let config = AtlasConfig::default();
        assert_eq!(config.initial_size, 512);
        assert_eq!(config.max_size, 4096);
    }

    #[test]
    fn atlas_config_is_copy() {
        let a = default_atlas_config();
        let b = a; // Copy
        assert_eq!(a.initial_size, b.initial_size);
        assert_eq!(a.max_size, b.max_size);
    }
}
