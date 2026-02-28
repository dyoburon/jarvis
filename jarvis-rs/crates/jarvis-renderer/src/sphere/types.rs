//! Sphere mesh vertex types and buffer layout.

/// A single vertex of the sphere mesh.
///
/// Layout: position(vec3) + normal(vec3) + barycentric(vec3) = 36 bytes.
/// The barycentric coordinates enable wireframe-style edge rendering
/// in the fragment shader without a geometry shader.
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SphereVertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub barycentric: [f32; 3],
}

impl SphereVertex {
    /// wgpu vertex buffer layout for `SphereVertex`.
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<SphereVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            // position: vec3<f32> at offset 0
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 0,
                shader_location: 0,
            },
            // normal: vec3<f32> at offset 12
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 12,
                shader_location: 1,
            },
            // barycentric: vec3<f32> at offset 24
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x3,
                offset: 24,
                shader_location: 2,
            },
        ],
    };
}

/// Sphere mesh quality presets matching `OrbQuality` config.
#[derive(Debug, Clone, Copy)]
pub struct SphereLod {
    pub latitudes: u32,
    pub longitudes: u32,
}

impl SphereLod {
    pub const LOW: Self = Self {
        latitudes: 16,
        longitudes: 24,
    };
    pub const MEDIUM: Self = Self {
        latitudes: 32,
        longitudes: 48,
    };
    pub const HIGH: Self = Self {
        latitudes: 48,
        longitudes: 64,
    };
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_vertex_size_is_36_bytes() {
        assert_eq!(std::mem::size_of::<SphereVertex>(), 36);
    }

    #[test]
    fn sphere_vertex_alignment_is_4_bytes() {
        assert_eq!(std::mem::align_of::<SphereVertex>(), 4);
    }

    #[test]
    fn sphere_lod_presets() {
        assert_eq!(SphereLod::LOW.latitudes, 16);
        assert_eq!(SphereLod::LOW.longitudes, 24);
        assert_eq!(SphereLod::MEDIUM.latitudes, 32);
        assert_eq!(SphereLod::MEDIUM.longitudes, 48);
        assert_eq!(SphereLod::HIGH.latitudes, 48);
        assert_eq!(SphereLod::HIGH.longitudes, 64);
    }

    #[test]
    fn bytemuck_cast_works() {
        let v = SphereVertex {
            position: [1.0, 2.0, 3.0],
            normal: [0.0, 1.0, 0.0],
            barycentric: [1.0, 0.0, 0.0],
        };
        let bytes: &[u8] = bytemuck::bytes_of(&v);
        assert_eq!(bytes.len(), 36);
    }
}
