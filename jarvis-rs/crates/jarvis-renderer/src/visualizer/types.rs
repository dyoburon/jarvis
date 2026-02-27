//! Visualizer trait and state types.

use crate::gpu::GpuUniforms;

/// Visual state the orb/visualizer can be in.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum VisualizerState {
    Idle,
    #[default]
    Listening,
    Speaking,
    Skill,
    Chat,
}

/// Trait for all visualizer implementations (orb, particle, waveform, null).
///
/// Each implementation controls how the orb/effect looks and how it
/// writes per-frame data into the shared GPU uniforms buffer.
pub trait Visualizer: Send + Sync {
    /// Whether this visualizer should be rendered.
    fn is_visible(&self) -> bool;

    /// Advance animation by `dt` seconds with current audio level.
    fn update(&mut self, dt: f32, audio_level: f32);

    /// Transition to a new visual state.
    fn apply_state(&mut self, state: VisualizerState);

    /// Write visualizer-specific values into the shared GPU uniforms.
    fn write_uniforms(&self, uniforms: &mut GpuUniforms);
}
