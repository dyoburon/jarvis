use crate::gpu::RendererError;

use super::helpers::log_first_frame;
use super::state::RenderState;

impl RenderState {
    /// Render a complete frame: clear, prepare text, draw.
    pub fn render_frame(&mut self, grid: &jarvis_terminal::Grid) -> Result<(), RendererError> {
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
                label: Some("jarvis frame encoder"),
            });

        let viewport_width = self.gpu.size.width as f32;
        let viewport_height = self.gpu.size.height as f32;
        let scale_factor = self.gpu.scale_factor as f32;

        self.text.prepare_grid(
            &self.gpu.device,
            &self.gpu.queue,
            grid,
            0.0,
            0.0,
            viewport_width,
            viewport_height,
            scale_factor,
        );

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("jarvis main pass"),
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

            self.text.render(&mut pass);
        }

        self.gpu.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        log_first_frame(self.gpu.size.width, self.gpu.size.height, self.gpu.format());

        Ok(())
    }
}
