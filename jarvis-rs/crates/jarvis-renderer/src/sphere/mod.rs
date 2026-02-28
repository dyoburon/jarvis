//! Orb sphere rendering: mesh generation, MVP math, and wgpu pipeline.
//!
//! The sphere renders to an offscreen `rgba16float` texture that feeds
//! into bloom and composite passes. Disabled when `visualizer.enabled = false`.

pub mod matrix;
mod mesh;
mod pipeline;
mod types;

pub use mesh::*;
pub use pipeline::*;
pub use types::*;
