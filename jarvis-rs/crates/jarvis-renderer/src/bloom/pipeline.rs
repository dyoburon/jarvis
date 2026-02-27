//! Two-pass Gaussian blur pipeline for bloom effects.
//!
//! Pass 1: horizontal blur (source → tex_h)
//! Pass 2: vertical blur (tex_h → tex_v)
//! The final blurred texture is composited over the scene.

use super::types::{BloomSettings, BloomUniforms};

/// Manages the two-pass bloom pipeline: horizontal blur → vertical blur.
pub struct BloomPipeline {
    pub pipeline_h: wgpu::RenderPipeline,
    pub pipeline_v: wgpu::RenderPipeline,
    pub uniform_buffer: wgpu::Buffer,
    pub bind_group_h: wgpu::BindGroup,
    pub bind_group_v: wgpu::BindGroup,
    pub texture_h: wgpu::Texture,
    pub texture_v: wgpu::Texture,
    pub view_h: wgpu::TextureView,
    pub view_v: wgpu::TextureView,
    pub settings: BloomSettings,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
}

impl BloomPipeline {
    /// Create the bloom pipeline.
    ///
    /// - `source_view`: texture view of the sphere offscreen render
    /// - `width`/`height`: texture dimensions
    pub fn new(
        device: &wgpu::Device,
        source_view: &wgpu::TextureView,
        width: u32,
        height: u32,
        settings: BloomSettings,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("bloom shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/bloom.wgsl").into()),
        });

        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("bloom sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bloom uniforms"),
            size: std::mem::size_of::<BloomUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(
                            std::mem::size_of::<BloomUniforms>() as u64,
                        ),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("bloom pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let create_pipeline = |label: &str, entry_point: &str| -> wgpu::RenderPipeline {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(entry_point),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba16Float,
                        blend: None,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: wgpu::PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState::default(),
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
            })
        };

        let pipeline_h = create_pipeline("bloom h pipeline", "fs_blur_h");
        let pipeline_v = create_pipeline("bloom v pipeline", "fs_blur_v");

        let (texture_h, view_h) = Self::create_texture(device, width, height, "bloom_h");
        let (texture_v, view_v) = Self::create_texture(device, width, height, "bloom_v");

        let bind_group_h = Self::create_bind_group(
            device,
            &bind_group_layout,
            &uniform_buffer,
            source_view,
            &sampler,
            "bloom bind group h",
        );

        let bind_group_v = Self::create_bind_group(
            device,
            &bind_group_layout,
            &uniform_buffer,
            &view_h,
            &sampler,
            "bloom bind group v",
        );

        Self {
            pipeline_h,
            pipeline_v,
            uniform_buffer,
            bind_group_h,
            bind_group_v,
            texture_h,
            texture_v,
            view_h,
            view_v,
            settings,
            bind_group_layout,
            sampler,
        }
    }

    /// Recreate textures and bind groups after a resize.
    pub fn resize(
        &mut self,
        device: &wgpu::Device,
        source_view: &wgpu::TextureView,
        width: u32,
        height: u32,
    ) {
        let (tex_h, v_h) = Self::create_texture(device, width, height, "bloom_h");
        let (tex_v, v_v) = Self::create_texture(device, width, height, "bloom_v");

        self.bind_group_h = Self::create_bind_group(
            device,
            &self.bind_group_layout,
            &self.uniform_buffer,
            source_view,
            &self.sampler,
            "bloom bind group h",
        );
        self.bind_group_v = Self::create_bind_group(
            device,
            &self.bind_group_layout,
            &self.uniform_buffer,
            &v_h,
            &self.sampler,
            "bloom bind group v",
        );

        self.texture_h = tex_h;
        self.texture_v = tex_v;
        self.view_h = v_h;
        self.view_v = v_v;
    }

    /// Upload bloom uniforms for the current frame.
    pub fn update_uniforms(&self, queue: &wgpu::Queue, width: u32, height: u32) {
        let uniforms = BloomUniforms {
            texel_size: [1.0 / width.max(1) as f32, 1.0 / height.max(1) as f32],
            intensity: self.settings.intensity,
            _padding: 0.0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));
    }

    /// Record the two-pass bloom into the command encoder.
    ///
    /// Returns early if bloom is disabled.
    pub fn render(&self, encoder: &mut wgpu::CommandEncoder) {
        if !self.settings.enabled {
            return;
        }

        // Pass 1: horizontal blur → texture_h
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom h pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view_h,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline_h);
            pass.set_bind_group(0, &self.bind_group_h, &[]);
            pass.draw(0..3, 0..1);
        }

        // Pass 2: vertical blur → texture_v
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom v pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.view_v,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            pass.set_pipeline(&self.pipeline_v);
            pass.set_bind_group(0, &self.bind_group_v, &[]);
            pass.draw(0..3, 0..1);
        }
    }

    /// The final blurred texture view (output of vertical pass).
    pub fn output_view(&self) -> &wgpu::TextureView {
        &self.view_v
    }

    fn create_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        label: &str,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    fn create_bind_group(
        device: &wgpu::Device,
        layout: &wgpu::BindGroupLayout,
        uniform_buffer: &wgpu::Buffer,
        texture_view: &wgpu::TextureView,
        sampler: &wgpu::Sampler,
        label: &str,
    ) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some(label),
            layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(sampler),
                },
            ],
        })
    }
}
