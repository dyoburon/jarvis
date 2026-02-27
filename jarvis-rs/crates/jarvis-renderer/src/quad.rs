//! GPU-accelerated filled rectangle renderer using instanced drawing.
//!
//! Draws colored quads for UI chrome elements like status bar backgrounds,
//! tab bar backgrounds, and pane borders.

use wgpu::util::DeviceExt;

// ---------------------------------------------------------------------------
// QuadInstance
// ---------------------------------------------------------------------------

/// A single filled rectangle to draw.
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct QuadInstance {
    /// Position and size in pixels: [x, y, width, height].
    pub rect: [f32; 4],
    /// RGBA color, each component 0.0..=1.0.
    pub color: [f32; 4],
}

// ---------------------------------------------------------------------------
// QuadRenderer
// ---------------------------------------------------------------------------

/// Renders filled rectangles via instanced drawing.
pub struct QuadRenderer {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    uniform_buffer: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    instance_count: u32,
    max_instances: u32,
}

/// Unit quad vertices (2D position).
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Vertex {
    position: [f32; 2],
}

/// Uniform buffer for viewport resolution.
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct Uniforms {
    resolution: [f32; 2],
    _pad: [f32; 2],
}

const QUAD_VERTICES: &[Vertex] = &[
    Vertex { position: [0.0, 0.0] }, // top-left
    Vertex { position: [1.0, 0.0] }, // top-right
    Vertex { position: [1.0, 1.0] }, // bottom-right
    Vertex { position: [0.0, 1.0] }, // bottom-left
];

const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

const MAX_INSTANCES: u32 = 256;

const SHADER_SOURCE: &str = r#"
struct Uniforms {
    resolution: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct InstanceInput {
    @location(1) rect: vec4<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Scale unit quad by instance rect size and translate by instance position
    let pixel_x = instance.rect.x + vertex.position.x * instance.rect.z;
    let pixel_y = instance.rect.y + vertex.position.y * instance.rect.w;

    // Convert from pixel coordinates to NDC (-1..1)
    let ndc_x = (pixel_x / uniforms.resolution.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (pixel_y / uniforms.resolution.y) * 2.0;

    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = instance.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;

impl QuadRenderer {
    /// Create a new QuadRenderer with the given surface format.
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("quad shader"),
            source: wgpu::ShaderSource::Wgsl(SHADER_SOURCE.into()),
        });

        // Uniform buffer for resolution
        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad uniforms"),
            contents: bytemuck::cast_slice(&[Uniforms {
                resolution: [1280.0, 800.0],
                _pad: [0.0; 2],
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("quad bind group layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("quad bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("quad pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("quad pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[
                    // Vertex buffer (per-vertex)
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Vertex>() as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 0,
                            format: wgpu::VertexFormat::Float32x2,
                        }],
                    },
                    // Instance buffer (per-instance)
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<QuadInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            // rect: vec4<f32>
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                            // color: vec4<f32>
                            wgpu::VertexAttribute {
                                offset: 16,
                                shader_location: 2,
                                format: wgpu::VertexFormat::Float32x4,
                            },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad vertices"),
            contents: bytemuck::cast_slice(QUAD_VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quad indices"),
            contents: bytemuck::cast_slice(QUAD_INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad instances"),
            size: (MAX_INSTANCES as u64) * std::mem::size_of::<QuadInstance>() as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            vertex_buffer,
            index_buffer,
            instance_buffer,
            uniform_buffer,
            bind_group,
            instance_count: 0,
            max_instances: MAX_INSTANCES,
        }
    }

    /// Upload quad instances and update the viewport resolution.
    pub fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        quads: &[QuadInstance],
        viewport_width: f32,
        viewport_height: f32,
    ) {
        let count = quads.len().min(self.max_instances as usize);
        self.instance_count = count as u32;

        if count > 0 {
            queue.write_buffer(
                &self.instance_buffer,
                0,
                bytemuck::cast_slice(&quads[..count]),
            );
        }

        queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[Uniforms {
                resolution: [viewport_width, viewport_height],
                _pad: [0.0; 2],
            }]),
        );
    }

    /// Draw all prepared quads into the render pass.
    pub fn render<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        if self.instance_count == 0 {
            return;
        }

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..6, 0, 0..self.instance_count);
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quad_instance_size() {
        assert_eq!(std::mem::size_of::<QuadInstance>(), 32); // 8 floats * 4 bytes
    }

    #[test]
    fn vertex_size() {
        assert_eq!(std::mem::size_of::<Vertex>(), 8); // 2 floats * 4 bytes
    }

    #[test]
    fn uniforms_size() {
        assert_eq!(std::mem::size_of::<Uniforms>(), 16); // 4 floats * 4 bytes
    }

    #[test]
    fn quad_indices_form_two_triangles() {
        assert_eq!(QUAD_INDICES.len(), 6);
        // Triangle 1: 0-1-2, Triangle 2: 0-2-3
        assert_eq!(QUAD_INDICES[0], 0);
        assert_eq!(QUAD_INDICES[1], 1);
        assert_eq!(QUAD_INDICES[2], 2);
        assert_eq!(QUAD_INDICES[3], 0);
        assert_eq!(QUAD_INDICES[4], 2);
        assert_eq!(QUAD_INDICES[5], 3);
    }

    #[test]
    fn quad_vertices_form_unit_quad() {
        assert_eq!(QUAD_VERTICES.len(), 4);
        assert_eq!(QUAD_VERTICES[0].position, [0.0, 0.0]);
        assert_eq!(QUAD_VERTICES[1].position, [1.0, 0.0]);
        assert_eq!(QUAD_VERTICES[2].position, [1.0, 1.0]);
        assert_eq!(QUAD_VERTICES[3].position, [0.0, 1.0]);
    }
}
