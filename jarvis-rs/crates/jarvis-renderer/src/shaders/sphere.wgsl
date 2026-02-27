// Sphere (orb) vertex + fragment shaders.
//
// Renders a UV sphere with Fresnel rim glow, scan lines, equator bars,
// and audio-reactive color gradient. Reads from shared Uniforms buffer
// and a per-draw SphereUniforms buffer for MVP matrix + orb colors.

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

struct SphereUniforms {
    mvp: mat4x4<f32>,
    model: mat4x4<f32>,
    orb_color: vec4<f32>,
    orb_secondary: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> u: Uniforms;

@group(1) @binding(0)
var<uniform> sphere: SphereUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) barycentric: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) barycentric: vec3<f32>,
    @location(3) uv_y: f32,
};

// ---------------------------------------------------------------------------
// Vertex shader
// ---------------------------------------------------------------------------

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    // Noise displacement along normal for organic feel
    let noise_freq = 3.0;
    let noise_amp = 0.02 * (1.0 + u.audio_level * 0.5);
    let noise_val = sin(in.position.x * noise_freq + u.time * 0.5)
                  * cos(in.position.y * noise_freq * 1.3 + u.time * 0.3)
                  * sin(in.position.z * noise_freq * 0.7 + u.time * 0.7);
    let displaced = in.position + in.normal * noise_val * noise_amp;

    let world_pos = sphere.model * vec4<f32>(displaced, 1.0);
    out.clip_position = sphere.mvp * vec4<f32>(displaced, 1.0);
    out.world_normal = normalize((sphere.model * vec4<f32>(in.normal, 0.0)).xyz);
    out.world_position = world_pos.xyz;
    out.barycentric = in.barycentric;

    // UV Y for equator effects: acos(normal.y) / PI → 0 at north, 1 at south
    out.uv_y = acos(clamp(in.normal.y, -1.0, 1.0)) / 3.14159265;

    return out;
}

// ---------------------------------------------------------------------------
// Fragment shader
// ---------------------------------------------------------------------------

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let view_dir = normalize(-in.world_position);
    let n = normalize(in.world_normal);

    // Fresnel rim glow
    let ndotv = max(dot(n, view_dir), 0.0);
    let fresnel = pow(1.0 - ndotv, 3.0) * u.intensity;

    // Base color: gradient from primary to secondary based on latitude
    let base_color = mix(sphere.orb_color.rgb, sphere.orb_secondary.rgb, in.uv_y);

    // Scan lines: horizontal bands
    let scan = sin(in.uv_y * 120.0 + u.time * 2.0) * 0.5 + 0.5;
    let scan_mask = smoothstep(0.3, 0.7, scan) * u.scanline_intensity * 2.0;

    // Equator bars: audio-reactive horizontal bands near equator
    let equator_dist = abs(in.uv_y - 0.5);
    let bar_spread = 0.15 + u.audio_level * 0.2;
    let bar_mask = smoothstep(bar_spread, bar_spread - 0.05, equator_dist);
    let bar_pulse = sin(in.uv_y * 40.0 + u.time * 3.0) * 0.5 + 0.5;
    let bars = bar_mask * bar_pulse * u.audio_level;

    // Wireframe edges via barycentric coordinates
    let bary = in.barycentric;
    let edge_width = 0.02;
    let edge = 1.0 - smoothstep(0.0, edge_width, min(min(bary.x, bary.y), bary.z));
    let wireframe = edge * 0.15;

    // Combine
    var color = base_color * (0.3 + fresnel * 0.7);
    color = color + sphere.orb_color.rgb * scan_mask * 0.1;
    color = color + sphere.orb_secondary.rgb * bars * 0.4;
    color = color + vec3<f32>(wireframe);

    // Alpha: solid core with fresnel edge fade
    let alpha = 0.85 + fresnel * 0.15;

    return vec4<f32>(color, alpha);
}
