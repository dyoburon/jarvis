// Boot screen shader — surveillance HUD style
//
// Renders: dark background, sweeping scan line, corner brackets.
// All colors from uniforms (TOML-configurable).

struct Uniforms {
    time: f32,
    progress: f32,
    screen_width: f32,
    screen_height: f32,
    accent_r: f32,
    accent_g: f32,
    accent_b: f32,
    bg_r: f32,
    bg_g: f32,
    bg_b: f32,
    opacity: f32,
    _pad: f32,
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

// Corner bracket — returns 1.0 if pixel is on a bracket line.
fn corner_bracket(uv: vec2<f32>, aspect: f32) -> f32 {
    let px = uv * vec2<f32>(u.screen_width, u.screen_height);
    let w = u.screen_width;
    let h = u.screen_height;

    // Bracket size in pixels — scales with screen
    let arm = min(w, h) * 0.06;
    let thick = 2.0;
    let margin = min(w, h) * 0.04;

    var hit = 0.0;

    // Top-left
    if px.x >= margin && px.x <= margin + arm && px.y >= margin && px.y <= margin + thick { hit = 1.0; }
    if px.x >= margin && px.x <= margin + thick && px.y >= margin && px.y <= margin + arm { hit = 1.0; }

    // Top-right
    if px.x >= w - margin - arm && px.x <= w - margin && px.y >= margin && px.y <= margin + thick { hit = 1.0; }
    if px.x >= w - margin - thick && px.x <= w - margin && px.y >= margin && px.y <= margin + arm { hit = 1.0; }

    // Bottom-left
    if px.x >= margin && px.x <= margin + arm && px.y >= h - margin - thick && px.y <= h - margin { hit = 1.0; }
    if px.x >= margin && px.x <= margin + thick && px.y >= h - margin - arm && px.y <= h - margin { hit = 1.0; }

    // Bottom-right
    if px.x >= w - margin - arm && px.x <= w - margin && px.y >= h - margin - thick && px.y <= h - margin { hit = 1.0; }
    if px.x >= w - margin - thick && px.x <= w - margin && px.y >= h - margin - arm && px.y <= h - margin { hit = 1.0; }

    return hit;
}

// Horizontal scan line sweeping top to bottom.
fn scan_line(uv: vec2<f32>) -> f32 {
    let period = 1.2; // seconds per sweep
    let y_pos = fract(u.time / period);
    let dist = abs(uv.y - y_pos);
    // Bright core + soft glow tail
    let core = smoothstep(0.003, 0.0, dist);
    let glow = smoothstep(0.06, 0.0, dist) * 0.15;
    return core + glow;
}

// Subtle vignette — darkens edges.
fn vignette(uv: vec2<f32>) -> f32 {
    let d = distance(uv, vec2<f32>(0.5, 0.5));
    return 1.0 - smoothstep(0.4, 0.9, d) * 0.4;
}

// sRGB to linear conversion for a single channel.
fn srgb_to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        return c / 12.92;
    }
    return pow((c + 0.055) / 1.055, 2.4);
}

fn srgb3(c: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(srgb_to_linear(c.x), srgb_to_linear(c.y), srgb_to_linear(c.z));
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = u.screen_width / u.screen_height;
    let bg = srgb3(vec3<f32>(u.bg_r, u.bg_g, u.bg_b));
    let accent = srgb3(vec3<f32>(u.accent_r, u.accent_g, u.accent_b));

    // Start with solid dark background
    var color = bg;

    // Scan line — accent colored
    let scan = scan_line(in.uv);
    color = color + accent * scan * 0.5;

    // Corner brackets — accent colored
    let bracket = corner_bracket(in.uv, aspect);
    color = color + accent * bracket * 0.6;

    // Vignette
    let vig = vignette(in.uv);
    color = color * vig;

    return vec4<f32>(color, u.opacity);
}
