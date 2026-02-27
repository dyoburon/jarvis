use std::sync::Arc;
use winit::window::Window;

use crate::gpu::{GpuContext, RendererError};
use crate::quad::QuadRenderer;

/// Core rendering state holding GPU context and quad renderer.
///
/// Text rendering has been removed — all UI text is now rendered via
/// webview panels (wry + HTML/CSS). This struct handles only the wgpu
/// background visuals (hex grid, orb, bloom, composite) and UI chrome
/// quad backgrounds.
pub struct RenderState {
    pub gpu: GpuContext,
    pub quad: QuadRenderer,
    pub clear_color: wgpu::Color,
}

impl RenderState {
    /// Create a fully initialized render state from a window.
    pub async fn new(window: Arc<Window>) -> Result<Self, RendererError> {
        let gpu = GpuContext::new(window).await?;
        let quad = QuadRenderer::new(&gpu.device, gpu.format());

        Ok(Self {
            gpu,
            quad,
            clear_color: wgpu::Color {
                r: 0.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            },
        })
    }

    /// Handle a window resize by reconfiguring the surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.gpu.resize(width, height);
    }

    /// Set the background clear color for frame rendering.
    pub fn set_clear_color(&mut self, r: f64, g: f64, b: f64) {
        self.clear_color = wgpu::Color { r, g, b, a: 1.0 };
    }

    /// Render a frame with just the background (quads).
    ///
    /// Webview panels are composited on top by the OS window manager.
    pub fn render_background(&mut self) -> Result<(), RendererError> {
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

        let mut encoder = self
            .gpu
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("jarvis background encoder"),
            });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("jarvis background pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.clear_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.quad.render(&mut pass);
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        super::helpers::log_first_frame(self.gpu.size.width, self.gpu.size.height, self.gpu.format());

        Ok(())
    }
}
