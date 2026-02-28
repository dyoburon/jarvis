//! Orb visualizer — the default Jarvis sphere effect.

use jarvis_config::schema::JarvisConfig;

use super::types::{Visualizer, VisualizerState};
use crate::gpu::GpuUniforms;

/// The orb visualizer: a glowing sphere that reacts to audio and state.
pub struct OrbVisualizer {
    visible: bool,
    state: VisualizerState,
    audio_level: f32,
    /// Smoothed audio level for less jittery animation.
    smoothed_audio: f32,
    /// Current scale (interpolates toward target).
    scale: f32,
    target_scale: f32,
    /// Current intensity (interpolates toward target).
    intensity: f32,
    target_intensity: f32,
    /// Orb center position in NDC.
    center_x: f32,
    center_y: f32,
}

impl OrbVisualizer {
    /// Create from application config.
    pub fn from_config(config: &JarvisConfig) -> Self {
        let vis = &config.visualizer;
        Self {
            visible: vis.enabled,
            state: VisualizerState::Listening,
            audio_level: 0.0,
            smoothed_audio: 0.0,
            scale: vis.state_listening.scale as f32,
            target_scale: vis.state_listening.scale as f32,
            intensity: vis.state_listening.intensity as f32,
            target_intensity: vis.state_listening.intensity as f32,
            center_x: 0.0,
            center_y: 0.0,
        }
    }
}

impl Visualizer for OrbVisualizer {
    fn is_visible(&self) -> bool {
        self.visible
    }

    fn update(&mut self, _dt: f32, audio_level: f32) {
        self.audio_level = audio_level;
        // Exponential smoothing (α ≈ 0.15)
        self.smoothed_audio += (audio_level - self.smoothed_audio) * 0.15;
        // Lerp scale and intensity toward targets
        self.scale += (self.target_scale - self.scale) * 0.1;
        self.intensity += (self.target_intensity - self.intensity) * 0.1;
    }

    fn apply_state(&mut self, state: VisualizerState) {
        self.state = state;
        // State-specific targets (matching config defaults)
        match state {
            VisualizerState::Idle => {
                self.target_scale = 0.8;
                self.target_intensity = 0.6;
            }
            VisualizerState::Listening => {
                self.target_scale = 1.0;
                self.target_intensity = 1.0;
            }
            VisualizerState::Speaking => {
                self.target_scale = 1.1;
                self.target_intensity = 1.4;
            }
            VisualizerState::Skill => {
                self.target_scale = 0.9;
                self.target_intensity = 1.2;
            }
            VisualizerState::Chat => {
                self.target_scale = 0.55;
                self.target_intensity = 0.8;
                self.center_x = -0.8; // offset to left
                self.center_y = 0.4; // offset up
            }
        }
        // Reset position for non-chat states
        if state != VisualizerState::Chat {
            self.center_x = 0.0;
            self.center_y = 0.0;
        }
    }

    fn write_uniforms(&self, uniforms: &mut GpuUniforms) {
        uniforms.audio_level = self.smoothed_audio;
        uniforms.orb_scale = self.scale;
        uniforms.intensity = self.intensity;
        uniforms.orb_center_x = self.center_x;
        uniforms.orb_center_y = self.center_y;
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn orb_from_default_config() {
        let config = JarvisConfig::default();
        let orb = OrbVisualizer::from_config(&config);
        assert!(orb.is_visible());
        assert!((orb.scale - 1.0).abs() < 1e-3);
        assert!((orb.intensity - 1.0).abs() < 1e-3);
    }

    #[test]
    fn orb_disabled_when_visualizer_off() {
        let mut config = JarvisConfig::default();
        config.visualizer.enabled = false;
        let orb = OrbVisualizer::from_config(&config);
        assert!(!orb.is_visible());
    }

    #[test]
    fn orb_apply_state_speaking() {
        let config = JarvisConfig::default();
        let mut orb = OrbVisualizer::from_config(&config);
        orb.apply_state(VisualizerState::Speaking);
        assert!((orb.target_scale - 1.1).abs() < 1e-3);
        assert!((orb.target_intensity - 1.4).abs() < 1e-3);
    }

    #[test]
    fn orb_apply_state_chat_offsets_position() {
        let config = JarvisConfig::default();
        let mut orb = OrbVisualizer::from_config(&config);
        orb.apply_state(VisualizerState::Chat);
        assert!(orb.center_x < 0.0); // offset left
        assert!(orb.center_y > 0.0); // offset up
    }

    #[test]
    fn orb_update_smooths_audio() {
        let config = JarvisConfig::default();
        let mut orb = OrbVisualizer::from_config(&config);
        orb.update(0.016, 1.0);
        // After one step with α=0.15: smoothed = 0 + (1-0)*0.15 = 0.15
        assert!((orb.smoothed_audio - 0.15).abs() < 1e-3);
    }

    #[test]
    fn orb_write_uniforms() {
        let config = JarvisConfig::default();
        let mut orb = OrbVisualizer::from_config(&config);
        orb.update(0.016, 0.5);
        let mut uniforms = GpuUniforms::from_config(&config);
        orb.write_uniforms(&mut uniforms);
        assert!(uniforms.audio_level > 0.0);
        assert!((uniforms.orb_scale - orb.scale).abs() < 1e-6);
    }
}
