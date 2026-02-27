use jarvis_config::schema::{BackgroundConfig, BackgroundMode as ConfigBackgroundMode};

use super::helpers::hex_to_rgb;
use super::types::BackgroundMode;

/// Renders backgrounds behind the terminal grid using wgpu.
///
/// The GPU pipeline is kept as `Option` because it can only be created once
/// a wgpu device and surface format are available at runtime.
pub struct BackgroundRenderer {
    pub mode: BackgroundMode,
    pub pipeline: Option<wgpu::RenderPipeline>,
    pub time: f32,
}

impl BackgroundRenderer {
    /// Create a new renderer defaulting to solid black.
    pub fn new() -> Self {
        Self {
            mode: BackgroundMode::Solid {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            },
            pipeline: None,
            time: 0.0,
        }
    }

    /// Create a renderer from the application configuration.
    ///
    /// Converts config hex color strings to floating-point RGB values.
    pub fn from_config(config: &BackgroundConfig) -> Self {
        let mode = match config.mode {
            ConfigBackgroundMode::Solid => {
                let [r, g, b] = hex_to_rgb(&config.solid_color).unwrap_or([0.0, 0.0, 0.0]);
                BackgroundMode::Solid { r, g, b }
            }
            ConfigBackgroundMode::Gradient => {
                let colors: Vec<[f64; 3]> = config
                    .gradient
                    .colors
                    .iter()
                    .filter_map(|c| hex_to_rgb(c))
                    .collect();
                let angle = config.gradient.angle as f32;
                BackgroundMode::Gradient { colors, angle }
            }
            ConfigBackgroundMode::HexGrid => {
                let [r, g, b] = hex_to_rgb(&config.hex_grid.color).unwrap_or([0.0, 0.824, 1.0]);
                BackgroundMode::HexGrid {
                    color: [r as f32, g as f32, b as f32],
                    opacity: config.hex_grid.opacity as f32,
                    time: 0.0,
                }
            }
            ConfigBackgroundMode::None => BackgroundMode::None,
            // Image and Video modes are not yet supported by the GPU renderer;
            // fall back to solid black.
            _ => {
                let [r, g, b] = hex_to_rgb(&config.solid_color).unwrap_or([0.0, 0.0, 0.0]);
                BackgroundMode::Solid { r, g, b }
            }
        };

        Self {
            mode,
            pipeline: None,
            time: 0.0,
        }
    }

    /// Advance animation time by `dt` seconds.
    pub fn update(&mut self, dt: f32) {
        self.time += dt;
        if let BackgroundMode::HexGrid { ref mut time, .. } = self.mode {
            *time = self.time;
        }
    }

    /// Return the appropriate wgpu clear color for the current mode.
    ///
    /// For `Solid`, returns the exact color. For modes that need a shader pass
    /// (Gradient, HexGrid) or `None`, returns black.
    pub fn clear_color(&self) -> wgpu::Color {
        match &self.mode {
            BackgroundMode::Solid { r, g, b } => wgpu::Color {
                r: *r,
                g: *g,
                b: *b,
                a: 1.0,
            },
            _ => wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        }
    }

    /// Whether this background mode requires a separate render pass with shaders.
    ///
    /// `Solid` is handled entirely through the clear color. `None` needs nothing.
    /// `Gradient` and `HexGrid` require fragment shader rendering.
    pub fn needs_render_pass(&self) -> bool {
        matches!(
            self.mode,
            BackgroundMode::Gradient { .. } | BackgroundMode::HexGrid { .. }
        )
    }
}

impl Default for BackgroundRenderer {
    fn default() -> Self {
        Self::new()
    }
}
