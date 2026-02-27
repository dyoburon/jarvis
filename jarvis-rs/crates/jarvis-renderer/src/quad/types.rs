/// A single filled rectangle to draw.
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct QuadInstance {
    /// Position and size in pixels: [x, y, width, height].
    pub rect: [f32; 4],
    /// RGBA color, each component 0.0..=1.0.
    pub color: [f32; 4],
}

/// Unit quad vertices (2D position).
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub(crate) struct Vertex {
    pub position: [f32; 2],
}

/// Uniform buffer for viewport resolution.
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub(crate) struct Uniforms {
    pub resolution: [f32; 2],
    pub _pad: [f32; 2],
}

pub(crate) const QUAD_VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.0],
    }, // top-left
    Vertex {
        position: [1.0, 0.0],
    }, // top-right
    Vertex {
        position: [1.0, 1.0],
    }, // bottom-right
    Vertex {
        position: [0.0, 1.0],
    }, // bottom-left
];

pub(crate) const QUAD_INDICES: &[u16] = &[0, 1, 2, 0, 2, 3];

pub(crate) const MAX_INSTANCES: u32 = 256;
