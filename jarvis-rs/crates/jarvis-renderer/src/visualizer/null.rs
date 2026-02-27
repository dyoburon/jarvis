//! Null visualizer — used when `visualizer.enabled = false`.
//!
//! Does nothing, renders nothing, costs nothing.

use super::types::{Visualizer, VisualizerState};
use crate::gpu::GpuUniforms;

/// A no-op visualizer for when the orb is disabled.
pub struct NullVisualizer;

impl NullVisualizer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullVisualizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Visualizer for NullVisualizer {
    fn is_visible(&self) -> bool {
        false
    }

    fn update(&mut self, _dt: f32, _audio_level: f32) {}

    fn apply_state(&mut self, _state: VisualizerState) {}

    fn write_uniforms(&self, _uniforms: &mut GpuUniforms) {}
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use jarvis_config::schema::JarvisConfig;

    #[test]
    fn null_visualizer_is_not_visible() {
        let vis = NullVisualizer::new();
        assert!(!vis.is_visible());
    }

    #[test]
    fn null_visualizer_write_uniforms_is_noop() {
        let vis = NullVisualizer::new();
        let config = JarvisConfig::default();
        let mut uniforms = GpuUniforms::from_config(&config);
        let before = uniforms.audio_level;
        vis.write_uniforms(&mut uniforms);
        assert!((uniforms.audio_level - before).abs() < f32::EPSILON);
    }
}
