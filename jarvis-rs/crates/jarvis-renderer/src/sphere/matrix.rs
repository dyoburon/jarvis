//! 4×4 matrix math for MVP transforms.
//!
//! Column-major layout matching WGSL `mat4x4<f32>`.
//! Minimal set: perspective, rotate X/Y, translate, scale.

/// 4×4 column-major matrix stored as `[f32; 16]`.
pub type Mat4 = [f32; 16];

/// Identity matrix.
pub const IDENTITY: Mat4 = [
    1.0, 0.0, 0.0, 0.0, // col 0
    0.0, 1.0, 0.0, 0.0, // col 1
    0.0, 0.0, 1.0, 0.0, // col 2
    0.0, 0.0, 0.0, 1.0, // col 3
];

/// Perspective projection matrix.
///
/// `fov_y` is vertical field of view in radians.
/// `near` and `far` are the clip planes (must be > 0).
pub fn perspective(fov_y: f32, aspect: f32, near: f32, far: f32) -> Mat4 {
    let f = 1.0 / (fov_y * 0.5).tan();
    let range_inv = 1.0 / (near - far);

    [
        f / aspect,
        0.0,
        0.0,
        0.0,
        0.0,
        f,
        0.0,
        0.0,
        0.0,
        0.0,
        (far + near) * range_inv,
        -1.0,
        0.0,
        0.0,
        2.0 * far * near * range_inv,
        0.0,
    ]
}

/// Rotation around the X axis.
pub fn rotate_x(angle: f32) -> Mat4 {
    let c = angle.cos();
    let s = angle.sin();
    [
        1.0, 0.0, 0.0, 0.0, 0.0, c, s, 0.0, 0.0, -s, c, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

/// Rotation around the Y axis.
pub fn rotate_y(angle: f32) -> Mat4 {
    let c = angle.cos();
    let s = angle.sin();
    [
        c, 0.0, -s, 0.0, 0.0, 1.0, 0.0, 0.0, s, 0.0, c, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

/// Translation matrix.
pub fn translate(x: f32, y: f32, z: f32) -> Mat4 {
    [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, x, y, z, 1.0,
    ]
}

/// Uniform scale matrix.
pub fn scale(s: f32) -> Mat4 {
    [
        s, 0.0, 0.0, 0.0, 0.0, s, 0.0, 0.0, 0.0, 0.0, s, 0.0, 0.0, 0.0, 0.0, 1.0,
    ]
}

/// Multiply two 4×4 column-major matrices: result = a × b.
pub fn mul(a: &Mat4, b: &Mat4) -> Mat4 {
    let mut out = [0.0f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[k * 4 + row] * b[col * 4 + k];
            }
            out[col * 4 + row] = sum;
        }
    }
    out
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: &Mat4, b: &Mat4, eps: f32) -> bool {
        a.iter().zip(b.iter()).all(|(x, y)| (x - y).abs() < eps)
    }

    #[test]
    fn identity_mul_identity() {
        let result = mul(&IDENTITY, &IDENTITY);
        assert!(approx_eq(&result, &IDENTITY, 1e-6));
    }

    #[test]
    fn translate_then_identity() {
        let t = translate(1.0, 2.0, 3.0);
        let result = mul(&t, &IDENTITY);
        assert!(approx_eq(&result, &t, 1e-6));
    }

    #[test]
    fn scale_doubles_position() {
        let s = scale(2.0);
        // A point at (1, 0, 0, 1) in column-major:
        // col0 = (2, 0, 0, 0), so x component is doubled
        assert!((s[0] - 2.0).abs() < 1e-6);
        assert!((s[5] - 2.0).abs() < 1e-6);
        assert!((s[10] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn rotate_x_90_degrees() {
        let r = rotate_x(std::f32::consts::FRAC_PI_2);
        // After 90° rotation around X, Y axis maps to Z axis
        // col1 should be (0, cos(90), sin(90), 0) = (0, 0, 1, 0)
        assert!((r[4] - 0.0).abs() < 1e-6); // col1[0]
        assert!((r[5] - 0.0).abs() < 1e-5); // col1[1] = cos(90) ≈ 0
        assert!((r[6] - 1.0).abs() < 1e-5); // col1[2] = sin(90) ≈ 1
    }

    #[test]
    fn rotate_y_90_degrees() {
        let r = rotate_y(std::f32::consts::FRAC_PI_2);
        // col0 = (cos(90), 0, -sin(90), 0) = (0, 0, -1, 0)
        assert!((r[0] - 0.0).abs() < 1e-5); // col0[0] = cos(90) ≈ 0
        assert!((r[2] - (-1.0)).abs() < 1e-5); // col0[2] = -sin(90) ≈ -1
                                               // col2 = (sin(90), 0, cos(90), 0) = (1, 0, 0, 0)
        assert!((r[8] - 1.0).abs() < 1e-5); // col2[0] = sin(90) ≈ 1
    }

    #[test]
    fn perspective_basic() {
        let p = perspective(std::f32::consts::FRAC_PI_4, 16.0 / 9.0, 0.1, 100.0);
        // p[0] = f / aspect, p[5] = f
        let f = 1.0 / (std::f32::consts::FRAC_PI_4 * 0.5).tan();
        assert!((p[0] - f / (16.0 / 9.0)).abs() < 1e-5);
        assert!((p[5] - f).abs() < 1e-5);
        // p[11] should be -1 (perspective divide)
        assert!((p[11] - (-1.0)).abs() < 1e-6);
    }
}
