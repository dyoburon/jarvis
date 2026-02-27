//! Background rendering modes for the terminal grid.
//!
//! Supports solid colors, gradients, hex grid animations, and none.
//! GPU pipeline creation is deferred until a wgpu device is available.

mod helpers;
mod renderer;
mod types;

pub use helpers::*;
pub use renderer::*;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use jarvis_config::schema::{BackgroundConfig, BackgroundMode as ConfigBackgroundMode};

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
