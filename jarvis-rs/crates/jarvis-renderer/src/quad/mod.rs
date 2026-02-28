//! GPU-accelerated filled rectangle renderer using instanced drawing.
//!
//! Draws colored quads for UI chrome elements like status bar backgrounds,
//! tab bar backgrounds, and pane borders.

mod pipeline;
mod renderer;
mod types;

pub use renderer::*;
pub use types::QuadInstance;

#[cfg(test)]
mod tests {
    use super::types::*;

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
