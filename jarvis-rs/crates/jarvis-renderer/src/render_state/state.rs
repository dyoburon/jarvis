//! Core rendering state: GPU context, background pipeline, and UI quads.

use std::sync::Arc;
use std::time::Instant;

use winit::window::Window;

use jarvis_config::schema::JarvisConfig;

use crate::background::BackgroundPipeline;
use crate::gpu::{GpuContext, GpuUniforms, RendererError};
use crate::quad::{QuadInstance, QuadRenderer};
use crate::ui::UiChrome;

/// Core rendering state holding GPU context, all shader pipelines,
/// and UI chrome quad renderer.
///
/// Render order per frame:
/// 1. Clear + hex grid background
/// 2. UI chrome quads
pub struct RenderState {
    pub gpu: GpuContext,
    pub quad: QuadRenderer,
    bg_pipeline: BackgroundPipeline,
    uniforms: GpuUniforms,
    last_frame: Instant,
    pub clear_color: wgpu::Color,
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
        })
    }

    /// Return the timestamp of the last rendered frame.
    pub fn last_frame_instant(&self) -> Instant {
        self.last_frame
    }

    /// Mark a frame as rendered (update last_frame to now).
    pub fn mark_frame(&mut self) {
        self.last_frame = Instant::now();
    }

    /// Handle a window resize by reconfiguring the surface and textures.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.gpu.resize(width, height);
        self.uniforms.update_viewport(width, height);
    }

    /// Set the background clear color for frame rendering.
    pub fn set_clear_color(&mut self, r: f64, g: f64, b: f64) {
        self.clear_color = wgpu::Color { r, g, b, a: 1.0 };
    }

    /// Set the background clear color with alpha for transparency.
    pub fn set_clear_color_alpha(&mut self, r: f64, g: f64, b: f64, a: f64) {
        self.clear_color = wgpu::Color { r, g, b, a };
    }

    /// Generate QuadInstance data from UI chrome and upload to GPU.
    pub fn prepare_chrome_quads(&mut self, chrome: &UiChrome, vw: f32, vh: f32) {
        let mut quads = Vec::new();

        // Tab bar background + active tab highlight
        if let Some(ref tab_bar) = chrome.tab_bar {
            if let Some(bar_rect) = chrome.tab_bar_rect(vw) {
                // Full-width dark background
                quads.push(QuadInstance {
                    rect: [
                        bar_rect.x as f32,
                        bar_rect.y as f32,
                        bar_rect.width as f32,
                        bar_rect.height as f32,
                    ],
                    color: [0.12, 0.12, 0.14, 1.0],
                });
                // Highlight the active tab
                let tab_count = tab_bar.tabs.len().max(1);
                let tab_w = vw / tab_count as f32;
                for (i, tab) in tab_bar.tabs.iter().enumerate() {
                    if tab.is_active {
                        quads.push(QuadInstance {
                            rect: [tab_w * i as f32, 0.0, tab_w, tab_bar.height],
                            color: [0.22, 0.22, 0.26, 1.0],
                        });
                    }
                }
            }
        }

        // Status bar background
        if let Some(ref sb) = chrome.status_bar {
            if let Some(bar_rect) = chrome.status_bar_rect(vw, vh) {
                quads.push(QuadInstance {
                    rect: [
                        bar_rect.x as f32,
                        bar_rect.y as f32,
                        bar_rect.width as f32,
                        bar_rect.height as f32,
                    ],
                    color: sb.bg_color,
                });
            }
        }

        self.quad.prepare(&self.gpu.queue, &quads, vw, vh);
    }

    /// Render a frame: hex grid background + UI quads.
    pub fn render_background(&mut self) -> Result<(), RendererError> {
        let now = Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;
        self.uniforms.update_time(dt);

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

        // Pass 2: UI chrome quads (loads existing content)
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
}
