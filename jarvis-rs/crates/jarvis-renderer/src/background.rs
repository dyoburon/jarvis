//! Background rendering modes for the terminal grid.
//!
//! Supports solid colors, gradients, hex grid animations, and none.
//! GPU pipeline creation is deferred until a wgpu device is available.

use jarvis_config::schema::{BackgroundConfig, BackgroundMode as ConfigBackgroundMode};
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
                let [r, g, b] =
                    hex_to_rgb(&config.hex_grid.color).unwrap_or([0.0, 0.824, 1.0]);
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

/// Parse a "#RRGGBB" hex string into normalized `[f64; 3]` values in 0.0..=1.0.
///
/// Returns `None` if the string is not a valid 6-digit hex color (with or
/// without the leading `#`).
pub fn hex_to_rgb(hex: &str) -> Option<[f64; 3]> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some([r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_to_rgb_parses_correctly() {
        let rgb = hex_to_rgb("#ff8000").unwrap();
        assert!((rgb[0] - 1.0).abs() < 1e-3);
        assert!((rgb[1] - 0.502).abs() < 1e-3);
        assert!((rgb[2] - 0.0).abs() < 1e-3);
    }

    #[test]
    fn hex_to_rgb_without_hash() {
        let rgb = hex_to_rgb("00ff00").unwrap();
        assert!((rgb[0] - 0.0).abs() < 1e-3);
        assert!((rgb[1] - 1.0).abs() < 1e-3);
        assert!((rgb[2] - 0.0).abs() < 1e-3);
    }

    #[test]
    fn hex_to_rgb_returns_none_for_invalid() {
        assert!(hex_to_rgb("").is_none());
        assert!(hex_to_rgb("#abc").is_none());
        assert!(hex_to_rgb("zzzzzz").is_none());
        assert!(hex_to_rgb("#gggggg").is_none());
        assert!(hex_to_rgb("#12345").is_none());
    }

    #[test]
    fn from_config_solid_mode() {
        let config = BackgroundConfig {
            mode: ConfigBackgroundMode::Solid,
            solid_color: "#ff0000".into(),
            ..Default::default()
        };
        let renderer = BackgroundRenderer::from_config(&config);
        match &renderer.mode {
            BackgroundMode::Solid { r, g, b } => {
                assert!((r - 1.0).abs() < 1e-3);
                assert!((g - 0.0).abs() < 1e-3);
                assert!((b - 0.0).abs() < 1e-3);
            }
            other => panic!("expected Solid, got {:?}", other),
        }
    }

    #[test]
    fn from_config_hex_grid_mode() {
        let config = BackgroundConfig {
            mode: ConfigBackgroundMode::HexGrid,
            ..Default::default()
        };
        let renderer = BackgroundRenderer::from_config(&config);
        match &renderer.mode {
            BackgroundMode::HexGrid {
                color,
                opacity,
                time,
            } => {
                // Default hex grid color is "#00d4ff"
                assert!(color[0] < 0.01); // red ~0
                assert!(color[1] > 0.8); // green ~0.83
                assert!(color[2] > 0.99); // blue ~1.0
                assert!((*opacity - 0.08).abs() < 1e-3);
                assert!((*time - 0.0).abs() < 1e-3);
            }
            other => panic!("expected HexGrid, got {:?}", other),
        }
    }

    #[test]
    fn from_config_gradient_mode() {
        let config = BackgroundConfig {
            mode: ConfigBackgroundMode::Gradient,
            ..Default::default()
        };
        let renderer = BackgroundRenderer::from_config(&config);
        match &renderer.mode {
            BackgroundMode::Gradient { colors, angle } => {
                // Default gradient has 2 colors: "#000000" and "#0a1520"
                assert_eq!(colors.len(), 2);
                assert!((angle - 180.0).abs() < 1e-3);
            }
            other => panic!("expected Gradient, got {:?}", other),
        }
    }

    #[test]
    fn clear_color_returns_solid_color() {
        let renderer = BackgroundRenderer {
            mode: BackgroundMode::Solid {
                r: 0.5,
                g: 0.25,
                b: 0.75,
            },
            pipeline: None,
            time: 0.0,
        };
        let c = renderer.clear_color();
        assert!((c.r - 0.5).abs() < 1e-6);
        assert!((c.g - 0.25).abs() < 1e-6);
        assert!((c.b - 0.75).abs() < 1e-6);
        assert!((c.a - 1.0).abs() < 1e-6);
    }

    #[test]
    fn clear_color_returns_black_for_non_solid() {
        let renderer = BackgroundRenderer {
            mode: BackgroundMode::HexGrid {
                color: [1.0, 1.0, 1.0],
                opacity: 1.0,
                time: 0.0,
            },
            pipeline: None,
            time: 0.0,
        };
        let c = renderer.clear_color();
        assert!((c.r - 0.0).abs() < 1e-6);
        assert!((c.g - 0.0).abs() < 1e-6);
        assert!((c.b - 0.0).abs() < 1e-6);
    }

    #[test]
    fn clear_color_returns_black_for_none() {
        let renderer = BackgroundRenderer {
            mode: BackgroundMode::None,
            pipeline: None,
            time: 0.0,
        };
        let c = renderer.clear_color();
        assert!((c.r - 0.0).abs() < 1e-6);
    }

    #[test]
    fn needs_render_pass_returns_correct_values() {
        assert!(!BackgroundRenderer {
            mode: BackgroundMode::Solid {
                r: 0.0,
                g: 0.0,
                b: 0.0,
            },
            pipeline: None,
            time: 0.0,
        }
        .needs_render_pass());

        assert!(!BackgroundRenderer {
            mode: BackgroundMode::None,
            pipeline: None,
            time: 0.0,
        }
        .needs_render_pass());

        assert!(BackgroundRenderer {
            mode: BackgroundMode::HexGrid {
                color: [0.0, 0.0, 0.0],
                opacity: 1.0,
                time: 0.0,
            },
            pipeline: None,
            time: 0.0,
        }
        .needs_render_pass());

        assert!(BackgroundRenderer {
            mode: BackgroundMode::Gradient {
                colors: vec![],
                angle: 0.0,
            },
            pipeline: None,
            time: 0.0,
        }
        .needs_render_pass());
    }

    #[test]
    fn update_advances_time() {
        let mut renderer = BackgroundRenderer::new();
        assert!((renderer.time - 0.0).abs() < 1e-6);
        renderer.update(0.016);
        assert!((renderer.time - 0.016).abs() < 1e-6);
        renderer.update(0.016);
        assert!((renderer.time - 0.032).abs() < 1e-6);
    }

    #[test]
    fn update_advances_hex_grid_time() {
        let config = BackgroundConfig {
            mode: ConfigBackgroundMode::HexGrid,
            ..Default::default()
        };
        let mut renderer = BackgroundRenderer::from_config(&config);
        renderer.update(1.0);
        if let BackgroundMode::HexGrid { time, .. } = renderer.mode {
            assert!((time - 1.0).abs() < 1e-6);
        } else {
            panic!("expected HexGrid mode");
        }
    }

    #[test]
    fn default_renderer_is_solid_black() {
        let renderer = BackgroundRenderer::new();
        match &renderer.mode {
            BackgroundMode::Solid { r, g, b } => {
                assert!((r - 0.0).abs() < 1e-6);
                assert!((g - 0.0).abs() < 1e-6);
                assert!((b - 0.0).abs() < 1e-6);
            }
            other => panic!("expected Solid black, got {:?}", other),
        }
        assert!(renderer.pipeline.is_none());
    }
}
