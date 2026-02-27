use serde::{Deserialize, Serialize};

/// Renderer-side background mode with concrete color values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackgroundMode {
    /// Flat solid color.
    Solid { r: f64, g: f64, b: f64 },
    /// Linear gradient between multiple colors at a given angle (degrees).
    Gradient { colors: Vec<[f64; 3]>, angle: f32 },
    /// Animated hex grid overlay.
    HexGrid {
        color: [f32; 3],
        opacity: f32,
        time: f32,
    },
    /// No background rendering.
    None,
}
