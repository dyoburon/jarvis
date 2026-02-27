//! Bloom (Gaussian blur) post-processing pipeline.
//!
//! Two-pass blur: horizontal → vertical. Reads from the sphere offscreen
//! texture and produces a soft glow texture for compositing.
//! Disabled when `effects.bloom.enabled = false`.

mod pipeline;
mod types;

pub use pipeline::*;
pub use types::*;
