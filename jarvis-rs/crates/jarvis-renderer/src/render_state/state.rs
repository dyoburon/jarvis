//! Core rendering state: GPU context, background, orb pipelines, and
//! UI quads.

use std::sync::Arc;
use std::time::Instant;

use winit::window::Window;

use jarvis_config::schema::JarvisConfig;

use crate::background::BackgroundPipeline;
use crate::gpu::{GpuContext, GpuUniforms, RendererError};
use crate::quad::QuadRenderer;

use super::orb::{self, OrbPipelines};

/// Core rendering state holding GPU context, all shader pipelines,
/// and UI chrome quad renderer.
///
/// Render order per frame:
/// 1. Clear + hex grid background
/// 2. Sphere → offscreen rgba16float texture
/// 3. Bloom (2-pass Gaussian blur)
/// 4. Composite (sphere + bloom → surface with alpha blend)
/// 5. UI chrome quads
pub struct RenderState {
    pub gpu: GpuContext,
    pub quad: QuadRenderer,
    bg_pipeline: BackgroundPipeline,
    uniforms: GpuUniforms,
    last_frame: Instant,
    pub clear_color: wgpu::Color,
    orb: Option<OrbPipelines>,
}

impl RenderState {
    /// Create a fully initialized render state from a window and config.
    pub async fn new(
        window: Arc<Window>,
        config: &JarvisConfig,
    ) -> Result<Self, RendererError> {
        let gpu = GpuContext::new(window).await?;
        let quad = QuadRenderer::new(&gpu.device, gpu.format());
        let bg_pipeline = BackgroundPipeline::new(&gpu.device, gpu.format());

        let mut uniforms = GpuUniforms::from_config(config);
        uniforms.update_viewport(gpu.size.width, gpu.size.height);

        let orb_pipelines = OrbPipelines::try_create(
            &gpu.device,
            gpu.format(),
            gpu.size.width,
            gpu.size.height,
            &bg_pipeline,
            config,
        );

        Ok(Self {
            gpu,
            quad,
            bg_pipeline,
            uniforms,
            last_frame: Instant::now(),
            clear_color: wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
            orb: orb_pipelines,
        })
    }

    /// Handle a window resize by reconfiguring the surface and textures.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.gpu.resize(width, height);
        self.uniforms.update_viewport(width, height);

        if let Some(orb) = &mut self.orb {
            orb.resize(&self.gpu.device, width, height);
        }
    }

    /// Set the background clear color for frame rendering.
    pub fn set_clear_color(&mut self, r: f64, g: f64, b: f64) {
        self.clear_color = wgpu::Color { r, g, b, a: 1.0 };
    }

    /// Set the background clear color with alpha for transparency.
    pub fn set_clear_color_alpha(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.clear_color = wgpu::Color { r, g, b, a };
    }

    /// Render a frame: background + sphere/bloom/composite + UI quads.
    pub fn render_background(&mut self) -> Result<(), RendererError> {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.uniforms.update_time(dt);

        // Let the visualizer update animation + write into uniforms
        if let Some(orb) = &mut self.orb {
            orb.visualizer.update(dt, self.uniforms.audio_level);
            orb.visualizer.write_uniforms(&mut self.uniforms);
        }

        // Upload shared uniforms to background pipeline
        self.bg_pipeline
            .update_uniforms(&self.gpu.queue, &self.uniforms);

        let output = match self.gpu.current_texture() {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("Failed to get surface texture: {e}");
                return Err(RendererError::SurfaceError(e.to_string()));
            }
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.gpu.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("jarvis frame encoder"),
            },
        );

        // Pass 1: Clear + hex grid background
        self.bg_pipeline
            .render(&mut encoder, &view, Some(self.clear_color));

        // Passes 2-4: Sphere → Bloom → Composite (if enabled)
        if let Some(orb_ref) = &self.orb {
            self.render_orb_passes(&mut encoder, &view, orb_ref);
        }

        // Pass 5: UI chrome quads (loads existing content)
        {
            let mut pass = encoder.begin_render_pass(
                &wgpu::RenderPassDescriptor {
                    label: Some("jarvis quad pass"),
                    color_attachments: &[Some(
                        wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Load,
                                store: wgpu::StoreOp::Store,
                            },
                        },
                    )],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                },
            );
            self.quad.render(&mut pass);
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        super::helpers::log_first_frame(
            self.gpu.size.width,
            self.gpu.size.height,
            self.gpu.format(),
        );

        Ok(())
    }

    /// Record sphere, bloom, and composite passes.
    fn render_orb_passes(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        surface_view: &wgpu::TextureView,
        orb: &OrbPipelines,
    ) {
        let (w, h) = (self.gpu.size.width, self.gpu.size.height);
        let aspect = self.uniforms.aspect_ratio;

        // Compute MVP for the sphere
        let sphere_uniforms = orb::compute_sphere_uniforms(
            &self.uniforms,
            aspect,
            self.uniforms.time,
        );
        orb.sphere
            .update_uniforms(&self.gpu.queue, &sphere_uniforms);

        // Upload bloom uniforms (texel size + intensity)
        orb.bloom.update_uniforms(&self.gpu.queue, w, h);

        // Pass 2: Sphere → offscreen texture
        orb.sphere
            .render(encoder, &self.bg_pipeline.bind_group);

        // Pass 3-4: Bloom (horizontal + vertical blur)
        orb.bloom.render(encoder);

        // Pass 5: Composite onto surface
        orb.composite.render(
            encoder,
            surface_view,
            &self.bg_pipeline.bind_group,
        );
    }
}
