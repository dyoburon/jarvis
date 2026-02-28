//! wgpu render pipeline for the orb sphere.
//!
//! Renders the sphere to an offscreen `rgba16float` texture that feeds
//! into the bloom and composite passes.

use super::types::SphereVertex;

/// Per-draw uniforms for the sphere: MVP matrix + colors.
///
/// Uploaded to bind group 1 each frame.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SphereUniforms {
    /// Model-View-Projection matrix (column-major).
    pub mvp: [f32; 16],
    /// Model matrix for world-space normal transform.
    pub model: [f32; 16],
    /// Primary orb color (RGBA).
    pub orb_color: [f32; 4],
    /// Secondary orb color (RGBA).
    pub orb_secondary: [f32; 4],
}

/// Manages the wgpu pipeline, buffers, and offscreen texture for sphere rendering.
pub struct SpherePipeline {
    pub pipeline: wgpu::RenderPipeline,
    pub vertex_buffer: wgpu::Buffer,
    pub vertex_count: u32,
    pub sphere_uniform_buffer: wgpu::Buffer,
    pub sphere_bind_group: wgpu::BindGroup,
    pub offscreen_texture: wgpu::Texture,
    pub offscreen_view: wgpu::TextureView,
}

impl SpherePipeline {
    /// Create the sphere pipeline.
    ///
    /// - `shared_bind_group_layout`: layout for bind group 0 (shared `GpuUniforms`)
    /// - `vertices`: pre-generated sphere mesh
    /// - `width`/`height`: offscreen texture dimensions
    pub fn new(
        device: &wgpu::Device,
        shared_bind_group_layout: &wgpu::BindGroupLayout,
        vertices: &[SphereVertex],
        width: u32,
        height: u32,
    ) -> Self {
        use wgpu::util::DeviceExt;

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("sphere shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/sphere.wgsl").into()),
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sphere vertex buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let sphere_uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("sphere uniforms"),
            size: std::mem::size_of::<SphereUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let sphere_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("sphere bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<
                            SphereUniforms,
                        >()
                            as u64),
                    },
                    count: None,
                }],
            });

        let sphere_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sphere bind group"),
            layout: &sphere_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: sphere_uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sphere pipeline layout"),
            bind_group_layouts: &[shared_bind_group_layout, &sphere_bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("sphere pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[SphereVertex::LAYOUT],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba16Float,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let (offscreen_texture, offscreen_view) =
            Self::create_offscreen_texture(device, width, height);

        Self {
            pipeline,
            vertex_buffer,
            vertex_count: vertices.len() as u32,
            sphere_uniform_buffer,
            sphere_bind_group,
            offscreen_texture,
            offscreen_view,
        }
    }

    /// Recreate the offscreen texture after a resize.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let (tex, view) = Self::create_offscreen_texture(device, width, height);
        self.offscreen_texture = tex;
        self.offscreen_view = view;
    }

    /// Upload sphere-specific uniforms (MVP, colors).
    pub fn update_uniforms(&self, queue: &wgpu::Queue, uniforms: &SphereUniforms) {
        queue.write_buffer(&self.sphere_uniform_buffer, 0, bytemuck::bytes_of(uniforms));
    }

    /// Record a sphere render pass to the offscreen texture.
    pub fn render(&self, encoder: &mut wgpu::CommandEncoder, shared_bind_group: &wgpu::BindGroup) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("sphere pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.offscreen_view,
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

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, shared_bind_group, &[]);
        pass.set_bind_group(1, &self.sphere_bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.draw(0..self.vertex_count, 0..1);
    }

    fn create_offscreen_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sphere offscreen"),
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
}
