// Composite shader — final pass combining all layers.
//
// Blends: background + sphere + bloom + post-processing effects.
// Reads from the shared Uniforms buffer for effect parameters.

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

@group(1) @binding(0)
var sphere_texture: texture_2d<f32>;

@group(1) @binding(1)
var bloom_texture: texture_2d<f32>;

@group(1) @binding(2)
var tex_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vi & 1u)) * 4.0 - 1.0;
    let y = f32(i32((vi >> 1u) & 1u)) * 4.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample sphere and bloom textures
    let sphere_color = textureSample(sphere_texture, tex_sampler, in.uv);
    let bloom_color = textureSample(bloom_texture, tex_sampler, in.uv);

    // Dark circle behind sphere (soft shadow)
    let orb_center = vec2<f32>(
        (u.orb_center_x + 1.0) * 0.5,
        (1.0 - u.orb_center_y) * 0.5
    );
    let dist_to_orb = length((in.uv - orb_center) * vec2<f32>(u.aspect_ratio, 1.0));
    let shadow_radius = u.orb_scale * 0.15;
    let shadow = smoothstep(shadow_radius, shadow_radius * 0.5, dist_to_orb) * 0.3;

    // Center dot (audio-reactive)
    let dot_radius = 0.003 + u.audio_level * 0.002;
    let dot = smoothstep(dot_radius, dot_radius * 0.5, dist_to_orb);

    // Combine sphere + bloom (additive)
    var color = sphere_color.rgb + bloom_color.rgb;

    // Add shadow darkening
    color = color - vec3<f32>(shadow);

    // Add center dot
    let dot_color = vec3<f32>(u.hex_color_r, u.hex_color_g, u.hex_color_b);
    color = color + dot_color * dot * u.audio_level;

    // CRT scan lines
    if u.scanline_intensity > 0.001 {
        let scan_y = in.uv.y * u.screen_height;
        let scanline = sin(scan_y * 3.14159265) * 0.5 + 0.5;
        color = color * (1.0 - u.scanline_intensity * (1.0 - scanline));
    }

    // Vignette
    if u.vignette_intensity > 0.001 {
        let vig_uv = in.uv * 2.0 - 1.0;
        let vig_dist = dot(vig_uv, vig_uv);
        let vig = 1.0 - vig_dist * 0.5 * u.vignette_intensity;
        color = color * max(vig, 0.0);
    }

    // Subtle flicker
    if u.flicker_amplitude > 0.0001 {
        let flicker = 1.0 + sin(u.time * 60.0) * u.flicker_amplitude;
        color = color * flicker;
    }

    // Alpha for window transparency
    let alpha = max(sphere_color.a, bloom_color.a);

    return vec4<f32>(color, alpha * u.bg_alpha);
}
