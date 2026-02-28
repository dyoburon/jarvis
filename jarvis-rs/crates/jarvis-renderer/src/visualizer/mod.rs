//! Visualizer system: orb, particle, waveform, or null.
//!
//! The active visualizer is selected by config. It controls how the
//! sphere/effect looks and writes per-frame data into GPU uniforms.

mod null;
mod orb;
mod types;

pub use null::*;
pub use orb::*;
pub use types::*;

use jarvis_config::schema::{JarvisConfig, VisualizerType};

/// Create the appropriate visualizer from config.
pub fn create_visualizer(config: &JarvisConfig) -> Box<dyn Visualizer> {
    if !config.visualizer.enabled {
        return Box::new(NullVisualizer::new());
    }

    match config.visualizer.visualizer_type {
        VisualizerType::Orb => Box::new(OrbVisualizer::from_config(config)),
        // Particle and Waveform are stubs — fall back to orb for now
        _ => Box::new(OrbVisualizer::from_config(config)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_visualizer_orb_when_enabled() {
        let config = JarvisConfig::default();
        let vis = create_visualizer(&config);
        assert!(vis.is_visible());
    }

    #[test]
    fn create_visualizer_null_when_disabled() {
        let mut config = JarvisConfig::default();
        config.visualizer.enabled = false;
        let vis = create_visualizer(&config);
        assert!(!vis.is_visible());
    }
}
