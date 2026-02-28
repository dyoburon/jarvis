//! Orb pipeline wiring: sphere + bloom + composite creation and per-frame
//! MVP computation.

use jarvis_config::schema::JarvisConfig;

use crate::background::BackgroundPipeline;
use crate::bloom::{BloomPipeline, BloomSettings};
use crate::gpu::GpuUniforms;
use crate::sphere::matrix as mat;
use crate::sphere::{SphereLod, SpherePipeline, SphereUniforms};
use crate::visualizer::{self, Visualizer};

use super::composite::CompositePipeline;

/// Sphere + bloom + composite pipelines (created when effects enabled).
pub(super) struct OrbPipelines {
    pub sphere: SpherePipeline,
    pub bloom: BloomPipeline,
    pub composite: CompositePipeline,
    pub visualizer: Box<dyn Visualizer>,
}

impl OrbPipelines {
    /// Create all orb pipelines if effects + visualizer are enabled.
    pub fn try_create(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
        bg: &BackgroundPipeline,
        config: &JarvisConfig,
    ) -> Option<Self> {
        if !config.effects.enabled || !config.visualizer.enabled {
            return None;
        }

        let vis = visualizer::create_visualizer(config);
        let mesh = crate::sphere::generate_sphere_mesh_lod(SphereLod::MEDIUM);

        let sphere =
            SpherePipeline::new(device, &bg.shared_bind_group_layout, &mesh, width, height);

        let bloom_settings = BloomSettings::from_config(config);
        let bloom = BloomPipeline::new(
            device,
            &sphere.offscreen_view,
            width,
            height,
            bloom_settings,
        );

        let composite = CompositePipeline::new(
            device,
            &bg.shared_bind_group_layout,
            &sphere.offscreen_view,
            bloom.output_view(),
            format,
        );

        Some(Self {
            sphere,
            bloom,
            composite,
            visualizer: vis,
        })
    }

    /// Resize all offscreen textures and rebuild bind groups.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.sphere.resize(device, width, height);
        self.bloom
            .resize(device, &self.sphere.offscreen_view, width, height);
        self.composite.resize(
            device,
            &self.sphere.offscreen_view,
            self.bloom.output_view(),
        );
    }
}

/// Build MVP matrix + colors for the sphere shader.
pub(super) fn compute_sphere_uniforms(
    uniforms: &GpuUniforms,
    aspect: f32,
    time: f32,
) -> SphereUniforms {
    let fov = std::f32::consts::FRAC_PI_4;
    let proj = mat::perspective(fov, aspect, 0.1, 100.0);

    // Slow rotation driven by time
    let rot_y = mat::rotate_y(time * 0.15);
    let rot_x = mat::rotate_x(0.3); // slight tilt

    // Scale from visualizer + translate to orb center
    let s = mat::scale(uniforms.orb_scale);
    let t = mat::translate(uniforms.orb_center_x, uniforms.orb_center_y, -3.0);

    // Model = translate * rotate_y * rotate_x * scale
    let model = mat::mul(&t, &mat::mul(&rot_y, &mat::mul(&rot_x, &s)));
    let mvp = mat::mul(&proj, &model);

    // Orb colors from hex grid color (primary) + dimmed secondary
    let primary = [
        uniforms.hex_color_r,
        uniforms.hex_color_g,
        uniforms.hex_color_b,
        uniforms.intensity,
    ];
    let secondary = [
        uniforms.hex_color_r * 0.3,
        uniforms.hex_color_g * 0.3,
        uniforms.hex_color_b * 0.3,
        uniforms.intensity * 0.5,
    ];

    SphereUniforms {
        mvp,
        model,
        orb_color: primary,
        orb_secondary: secondary,
    }
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_sphere_uniforms_produces_valid_mvp() {
        let config = JarvisConfig::default();
        let uniforms = GpuUniforms::from_config(&config);
        let su = compute_sphere_uniforms(&uniforms, 16.0 / 9.0, 1.0);

        // MVP should not be all zeros
        let sum: f32 = su.mvp.iter().map(|v| v.abs()).sum();
        assert!(sum > 0.0, "MVP matrix should not be all zeros");
    }

    #[test]
    fn compute_sphere_uniforms_colors_from_hex_grid() {
        let config = JarvisConfig::default();
        let mut uniforms = GpuUniforms::from_config(&config);
        uniforms.hex_color_r = 1.0;
        uniforms.hex_color_g = 0.5;
        uniforms.hex_color_b = 0.0;
        uniforms.intensity = 1.0;

        let su = compute_sphere_uniforms(&uniforms, 1.0, 0.0);

        assert!((su.orb_color[0] - 1.0).abs() < 1e-6);
        assert!((su.orb_color[1] - 0.5).abs() < 1e-6);
        assert!((su.orb_color[2] - 0.0).abs() < 1e-6);
        // Secondary is 30% of primary
        assert!((su.orb_secondary[0] - 0.3).abs() < 1e-6);
        assert!((su.orb_secondary[1] - 0.15).abs() < 1e-6);
    }

    #[test]
    fn compute_sphere_uniforms_model_not_identity() {
        let config = JarvisConfig::default();
        let uniforms = GpuUniforms::from_config(&config);
        let su = compute_sphere_uniforms(&uniforms, 1.0, 0.5);

        // Model matrix should differ from identity due to translate z=-3
        // Identity col3 = (0, 0, 0, 1), model col3 should have z ≈ -3
        assert!(
            (su.model[14] - (-3.0)).abs() < 0.5,
            "model z-translate should be near -3.0, got {}",
            su.model[14]
        );
    }

    #[test]
    fn compute_sphere_uniforms_rotation_changes_with_time() {
        let config = JarvisConfig::default();
        let uniforms = GpuUniforms::from_config(&config);

        let su_t0 = compute_sphere_uniforms(&uniforms, 1.0, 0.0);
        let su_t1 = compute_sphere_uniforms(&uniforms, 1.0, 10.0);

        // Different time → different model matrix
        let diff: f32 = su_t0
            .model
            .iter()
            .zip(su_t1.model.iter())
            .map(|(a, b)| (a - b).abs())
            .sum();
        assert!(diff > 0.01, "rotation should change with time");
    }
}
