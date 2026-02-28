//! GPU-rendered boot screen for the JARVIS initialization sequence.
//!
//! Renders a full-screen surveillance-style HUD with sweeping scan line,
//! corner brackets, cycling status messages, and a progress bar.
//! All colors are TOML-configurable.

mod text;
mod types;

pub use text::{BootTextRenderer, TextEntry};
pub use types::{BootScreenConfig, BootUniforms};

use crate::gpu::RendererError;
use crate::quad::{QuadInstance, QuadRenderer};

/// Full boot screen renderer.
///
/// Owns the shader pipeline for the surveillance HUD background,
/// a text renderer for title/status/percentage, and delegates
/// to a [`QuadRenderer`] for the progress bar.
pub struct BootScreen {
    // Shader pipeline
    pipeline: wgpu::RenderPipeline,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    // Text
    text_renderer: BootTextRenderer,
    // Quads (progress bar) — owned, not shared with normal render
    quad_renderer: QuadRenderer,
    // State
    config: BootScreenConfig,
    elapsed: f32,
    message_index: usize,
    message_timer: f32,
}

impl BootScreen {
    /// Create a new boot screen renderer.
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
        config: BootScreenConfig,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("boot screen shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/boot.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("boot uniforms"),
            size: std::mem::size_of::<BootUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("boot bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: std::num::NonZeroU64::new(
                        std::mem::size_of::<BootUniforms>() as u64,
                    ),
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("boot bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("boot pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("boot pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let text_renderer = BootTextRenderer::new(device, queue, format);
        let quad_renderer = QuadRenderer::new(device, format);

        Self {
            pipeline,
            uniform_buffer,
            bind_group,
            text_renderer,
            quad_renderer,
            config,
            elapsed: 0.0,
            message_index: 0,
            message_timer: 0.0,
        }
    }

    /// Advance time and cycle status messages.
    pub fn update(&mut self, dt: f32) {
        self.elapsed += dt;
        self.message_timer += dt;

        if self.message_timer >= self.config.message_interval {
            self.message_timer -= self.config.message_interval;
            let max = self.config.messages.len().saturating_sub(1);
            if self.message_index < max {
                self.message_index += 1;
            }
        }
    }

    /// Current progress as 0.0–1.0.
    pub fn progress(&self) -> f32 {
        (self.elapsed / self.config.duration).clamp(0.0, 1.0)
    }

    /// Whether the boot animation has finished.
    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.config.duration
    }

    /// Force the boot screen to complete immediately (skip).
    pub fn skip(&mut self) {
        self.elapsed = self.config.duration;
        self.message_index = self.config.messages.len().saturating_sub(1);
    }

    /// Render the full boot screen into the given surface texture.
    pub fn render(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) -> Result<(), RendererError> {
        let progress = self.progress();
        let w = width as f32;
        let h = height as f32;

        // --- Upload shader uniforms ---
        let uniforms = BootUniforms {
            time: self.elapsed,
            progress,
            screen_width: w,
            screen_height: h,
            accent_r: self.config.accent_color[0],
            accent_g: self.config.accent_color[1],
            accent_b: self.config.accent_color[2],
            bg_r: self.config.bg_color[0],
            bg_g: self.config.bg_color[1],
            bg_b: self.config.bg_color[2],
            opacity: 1.0,
            _pad: 0.0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        // --- Prepare progress bar quads ---
        let bar_w = w * 0.35;
        let bar_h = 4.0;
        let bar_x = (w - bar_w) * 0.5;
        let bar_y = h * 0.62;

        let quads = [
            // Track (dark background)
            QuadInstance {
                rect: [bar_x, bar_y, bar_w, bar_h],
                color: [
                    self.config.track_color[0],
                    self.config.track_color[1],
                    self.config.track_color[2],
                    1.0,
                ],
            },
            // Fill (accent)
            QuadInstance {
                rect: [bar_x, bar_y, bar_w * progress, bar_h],
                color: [
                    self.config.accent_color[0],
                    self.config.accent_color[1],
                    self.config.accent_color[2],
                    1.0,
                ],
            },
        ];
        self.quad_renderer.prepare(queue, &quads, w, h);

        // --- Prepare text ---
        let title_size = (h * 0.04).max(24.0);
        let status_size = (h * 0.018).max(12.0);
        let pct_size = status_size;

        let message = self
            .config
            .messages
            .get(self.message_index)
            .map(|s| s.as_str())
            .unwrap_or("SYSTEM ONLINE");

        let pct_text = format!("{}%", (progress * 100.0) as u32);

        let accent = glyphon::Color::rgba(
            (self.config.accent_color[0] * 255.0) as u8,
            (self.config.accent_color[1] * 255.0) as u8,
            (self.config.accent_color[2] * 255.0) as u8,
            255,
        );
        let muted = glyphon::Color::rgba(
            (self.config.muted_color[0] * 255.0) as u8,
            (self.config.muted_color[1] * 255.0) as u8,
            (self.config.muted_color[2] * 255.0) as u8,
            255,
        );

        let entries = [
            // Title: "J A R V I S"
            TextEntry {
                text: "J A R V I S",
                left: (w - title_size * 5.5) * 0.5,
                top: h * 0.44,
                font_size: title_size,
                line_height: title_size * 1.2,
                color: accent,
                max_width: None,
            },
            // Status message
            TextEntry {
                text: message,
                left: bar_x,
                top: bar_y - status_size * 2.0,
                font_size: status_size,
                line_height: status_size * 1.4,
                color: muted,
                max_width: Some(bar_w),
            },
            // Percentage
            TextEntry {
                text: &pct_text,
                left: bar_x + bar_w + 8.0,
                top: bar_y - (pct_size * 0.3),
                font_size: pct_size,
                line_height: pct_size * 1.4,
                color: muted,
                max_width: None,
            },
        ];

        self.text_renderer
            .prepare(device, queue, width, height, &entries)?;

        // --- Render passes ---
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("boot screen encoder"),
        });

        // Pass 1: Full-screen shader (background + scan line + brackets)
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("boot shader pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: self.config.bg_color[0] as f64,
                            g: self.config.bg_color[1] as f64,
                            b: self.config.bg_color[2] as f64,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.bind_group, &[]);
            pass.draw(0..3, 0..1);
        }

        // Pass 2: Progress bar quads (load existing, overlay)
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("boot quad pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.quad_renderer.render(&mut pass);
        }

        // Pass 3: Text (load existing, overlay)
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("boot text pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            self.text_renderer.render(&mut pass)?;
        }

        queue.submit(std::iter::once(encoder.finish()));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_clamps_at_1() {
        let mut config = BootScreenConfig::default();
        config.duration = 2.0;
        // Can't create BootScreen without GPU, test the math directly
        let elapsed = 5.0_f32;
        let progress = (elapsed / config.duration).clamp(0.0, 1.0);
        assert!((progress - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn progress_at_zero() {
        let config = BootScreenConfig::default();
        let elapsed = 0.0_f32;
        let progress = (elapsed / config.duration).clamp(0.0, 1.0);
        assert!((progress - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn progress_at_halfway() {
        let config = BootScreenConfig::default();
        let elapsed = config.duration * 0.5;
        let progress = (elapsed / config.duration).clamp(0.0, 1.0);
        assert!((progress - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn message_index_stays_in_bounds() {
        let config = BootScreenConfig::default();
        let max = config.messages.len().saturating_sub(1);
        // Simulate many cycles
        let mut idx = 0_usize;
        for _ in 0..100 {
            if idx < max {
                idx += 1;
            }
        }
        assert_eq!(idx, max);
        assert_eq!(config.messages[idx], "SYSTEM ONLINE");
    }
}
