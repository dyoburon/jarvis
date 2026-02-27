// Background shader — hex grid animation with simplex noise
//
// Renders a full-screen animated hex grid overlay. The grid fades in/out
// with simplex noise and pulses gently over time. All parameters come
// from the Uniforms buffer so they can be driven by config.

struct Uniforms {
    time: f32,
    audio_level: f32,
    power_level: f32,
    intensity: f32,
    scanline_intensity: f32,
    vignette_intensity: f32,
    screen_width: f32,
    screen_height: f32,
    aspect_ratio: f32,
    orb_center_x: f32,
    orb_center_y: f32,
    orb_scale: f32,
    bg_opacity: f32,
    bg_alpha: f32,
    hex_color_r: f32,
    hex_color_g: f32,
    hex_color_b: f32,
    flicker_amplitude: f32,
    _padding0: f32,
    _padding1: f32,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Full-screen triangle — 3 vertices cover the entire viewport.
@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vi & 1u)) * 4.0 - 1.0;
    let y = f32(i32((vi >> 1u) & 1u)) * 4.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// ---------------------------------------------------------------------------
// Simplex noise (2D) — based on Ashima Arts / Stefan Gustavson
// ---------------------------------------------------------------------------

fn mod289_3(x: vec3<f32>) -> vec3<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn mod289_2(x: vec2<f32>) -> vec2<f32> {
    return x - floor(x * (1.0 / 289.0)) * 289.0;
}

fn permute(x: vec3<f32>) -> vec3<f32> {
    return mod289_3(((x * 34.0) + 1.0) * x);
}

fn snoise(v: vec2<f32>) -> f32 {
    let C = vec4<f32>(
        0.211324865405187,   // (3.0 - sqrt(3.0)) / 6.0
        0.366025403784439,   // 0.5 * (sqrt(3.0) - 1.0)
        -0.577350269189626,  // -1.0 + 2.0 * C.x
        0.024390243902439    // 1.0 / 41.0
    );

    // First corner
    var i = floor(v + dot(v, C.yy));
    let x0 = v - i + dot(i, C.xx);

    // Other corners
    var i1: vec2<f32>;
    if x0.x > x0.y {
        i1 = vec2<f32>(1.0, 0.0);
    } else {
        i1 = vec2<f32>(0.0, 1.0);
    }
    let x12 = vec4<f32>(x0.xy + C.xx, x0.xy + C.zz) - vec4<f32>(i1, 0.0, 0.0);

    // Permutations
    i = mod289_2(i);
    let p = permute(permute(i.y + vec3<f32>(0.0, i1.y, 1.0)) + i.x + vec3<f32>(0.0, i1.x, 1.0));

    var m = max(vec3<f32>(0.5) - vec3<f32>(dot(x0, x0), dot(x12.xy, x12.xy), dot(x12.zw, x12.zw)), vec3<f32>(0.0));
    m = m * m;
    m = m * m;

    // Gradients
    let x_ = 2.0 * fract(p * C.www) - 1.0;
    let h = abs(x_) - 0.5;
    let ox = floor(x_ + 0.5);
    let a0 = x_ - ox;

    m = m * (1.79284291400159 - 0.85373472095314 * (a0 * a0 + h * h));

    let g = vec3<f32>(
        a0.x * x0.x + h.x * x0.y,
        a0.y * x12.x + h.y * x12.y,
        a0.z * x12.z + h.z * x12.w
    );

    return 130.0 * dot(m, g);
}

// ---------------------------------------------------------------------------
// Hex grid distance functions
// ---------------------------------------------------------------------------

/// Distance from point to nearest hex edge.
fn hex_dist(p: vec2<f32>) -> f32 {
    let q = abs(p);
    return max(q.x * 0.866025 + q.y * 0.5, q.y);
}

/// Compute hex grid coordinates and cell-local position.
/// Returns (cell_id, local_pos).
fn hex_coords(p: vec2<f32>) -> vec2<f32> {
    let r = vec2<f32>(1.0, 1.732);
    let h = r * 0.5;

    let a = p - r * floor(p / r + 0.5);
    let b = p - h - r * floor((p - h) / r + 0.5);

    if length(a) < length(b) {
        return a;
    } else {
        return b;
    }
}

// ---------------------------------------------------------------------------
// Fragment shader
// ---------------------------------------------------------------------------

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Aspect-corrected coordinates centered at origin
    let uv = (in.uv - 0.5) * vec2<f32>(u.aspect_ratio, 1.0);

    // Scale for hex grid density
    let scale = 12.0;
    let p = uv * scale;

    // Hex grid
    let hex_local = hex_coords(p);
    let d = hex_dist(hex_local);

    // Edge glow: bright at edges (d close to 0.5), dark in center
    let edge = smoothstep(0.45, 0.5, d);

    // Noise-based fade: some cells brighter than others, animated
    let noise_val = snoise(p * 0.3 + vec2<f32>(u.time * 0.05, u.time * 0.03));
    let cell_fade = smoothstep(-0.2, 0.6, noise_val);

    // Pulse: gentle global brightness oscillation
    let pulse = 0.8 + 0.2 * sin(u.time * 0.5);

    // Combine
    let hex_alpha = edge * cell_fade * pulse * u.bg_opacity;

    let color = vec3<f32>(u.hex_color_r, u.hex_color_g, u.hex_color_b);

    return vec4<f32>(color * hex_alpha, hex_alpha * u.bg_alpha);
}
