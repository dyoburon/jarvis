//! Sphere mesh generation.
//!
//! Generates a UV sphere with position, normal, and barycentric coordinates.
//! The barycentric coords enable wireframe-style edge rendering in the
//! fragment shader without needing a geometry shader pass.

use super::types::{SphereLod, SphereVertex};

/// Generate a UV sphere mesh.
///
/// `n_lat` = number of latitude bands (rows), `n_lon` = number of longitude
/// segments (columns). Each quad is split into 2 triangles, producing
/// `n_lat * n_lon * 6` vertices (no index buffer — simple triangle list).
///
/// The sphere has radius 1.0, centered at origin. North pole is (0, 1, 0).
pub fn generate_sphere_mesh(n_lat: u32, n_lon: u32) -> Vec<SphereVertex> {
    let n_lat = n_lat.max(2);
    let n_lon = n_lon.max(3);

    let mut vertices = Vec::with_capacity((n_lat * n_lon * 6) as usize);

    for lat in 0..n_lat {
        for lon in 0..n_lon {
            // Spherical coordinates for the 4 corners of this quad
            let p00 = sphere_point(lat, lon, n_lat, n_lon);
            let p10 = sphere_point(lat + 1, lon, n_lat, n_lon);
            let p01 = sphere_point(lat, lon + 1, n_lat, n_lon);
            let p11 = sphere_point(lat + 1, lon + 1, n_lat, n_lon);

            // Triangle 1: p00, p10, p01
            vertices.push(SphereVertex {
                position: p00,
                normal: p00, // unit sphere: normal == position
                barycentric: [1.0, 0.0, 0.0],
            });
            vertices.push(SphereVertex {
                position: p10,
                normal: p10,
                barycentric: [0.0, 1.0, 0.0],
            });
            vertices.push(SphereVertex {
                position: p01,
                normal: p01,
                barycentric: [0.0, 0.0, 1.0],
            });

            // Triangle 2: p10, p11, p01
            vertices.push(SphereVertex {
                position: p10,
                normal: p10,
                barycentric: [1.0, 0.0, 0.0],
            });
            vertices.push(SphereVertex {
                position: p11,
                normal: p11,
                barycentric: [0.0, 1.0, 0.0],
            });
            vertices.push(SphereVertex {
                position: p01,
                normal: p01,
                barycentric: [0.0, 0.0, 1.0],
            });
        }
    }

    vertices
}

/// Generate a sphere mesh from a quality preset.
pub fn generate_sphere_mesh_lod(lod: SphereLod) -> Vec<SphereVertex> {
    generate_sphere_mesh(lod.latitudes, lod.longitudes)
}

/// Compute a point on the unit sphere from latitude/longitude indices.
fn sphere_point(lat: u32, lon: u32, n_lat: u32, n_lon: u32) -> [f32; 3] {
    let theta = std::f32::consts::PI * (lat as f32) / (n_lat as f32);
    let phi = 2.0 * std::f32::consts::PI * (lon as f32) / (n_lon as f32);

    let sin_theta = theta.sin();
    let cos_theta = theta.cos();
    let sin_phi = phi.sin();
    let cos_phi = phi.cos();

    [sin_theta * cos_phi, cos_theta, sin_theta * sin_phi]
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_mesh_vertex_count() {
        let mesh = generate_sphere_mesh(4, 8);
        // 4 lat bands × 8 lon segments × 6 vertices per quad = 192
        assert_eq!(mesh.len(), 192);
    }

    #[test]
    fn sphere_mesh_first_vertex_is_north_pole() {
        let mesh = generate_sphere_mesh(4, 8);
        let v = &mesh[0];
        // lat=0, lon=0: theta=0 → (sin(0)*cos(0), cos(0), sin(0)*sin(0)) = (0, 1, 0)
        assert!((v.position[0] - 0.0).abs() < 1e-6);
        assert!((v.position[1] - 1.0).abs() < 1e-6);
        assert!((v.position[2] - 0.0).abs() < 1e-6);
    }

    #[test]
    fn sphere_mesh_normals_equal_positions() {
        let mesh = generate_sphere_mesh(4, 8);
        for v in &mesh {
            assert!((v.position[0] - v.normal[0]).abs() < 1e-6);
            assert!((v.position[1] - v.normal[1]).abs() < 1e-6);
            assert!((v.position[2] - v.normal[2]).abs() < 1e-6);
        }
    }

    #[test]
    fn sphere_mesh_barycentric_coords_valid() {
        let mesh = generate_sphere_mesh(4, 8);
        for (i, v) in mesh.iter().enumerate() {
            let sum = v.barycentric[0] + v.barycentric[1] + v.barycentric[2];
            assert!(
                (sum - 1.0).abs() < 1e-6,
                "vertex {i}: barycentric sum = {sum}"
            );
        }
    }

    #[test]
    fn sphere_mesh_lod_low() {
        let mesh = generate_sphere_mesh_lod(SphereLod::LOW);
        assert_eq!(mesh.len(), (16 * 24 * 6) as usize);
    }

    #[test]
    fn sphere_mesh_lod_high() {
        let mesh = generate_sphere_mesh_lod(SphereLod::HIGH);
        assert_eq!(mesh.len(), (48 * 64 * 6) as usize);
    }

    #[test]
    fn sphere_mesh_minimum_clamp() {
        // n_lat < 2 and n_lon < 3 should be clamped
        let mesh = generate_sphere_mesh(1, 1);
        assert_eq!(mesh.len(), (2 * 3 * 6) as usize);
    }

    #[test]
    fn sphere_point_south_pole() {
        let p = super::sphere_point(4, 0, 4, 8);
        // lat=n_lat → theta=PI → (sin(PI)*cos(0), cos(PI), sin(PI)*sin(0)) ≈ (0, -1, 0)
        assert!((p[0] - 0.0).abs() < 1e-5);
        assert!((p[1] - (-1.0)).abs() < 1e-5);
        assert!((p[2] - 0.0).abs() < 1e-5);
    }
}
