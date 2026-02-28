// Per-pane effects shader — glow border + dim overlay.
//
// Applied as a post-processing pass over each pane's rendered content.
// The composite shader handles full-screen effects (scanlines, vignette, flicker);
// this shader handles per-pane visual cues: active glow and inactive dim.

struct PaneUniforms {
    // Pane bounds in normalized [0,1] coordinates
    pane_x: f32,
    pane_y: f32,
    pane_w: f32,
    pane_h: f32,
    // Glow parameters
    glow_color_r: f32,
    glow_color_g: f32,
    glow_color_b: f32,
    glow_color_a: f32,
    glow_width: f32,
    // Dim parameters
    dim_factor: f32,       // 1.0 = fully bright, <1.0 = dimmed
    is_focused: f32,       // 1.0 = focused, 0.0 = unfocused
    // Viewport
    screen_width: f32,
    screen_height: f32,
    // Scanline (per-pane CRT effect, optional)
    scanline_intensity: f32,
    _padding0: f32,
    _padding1: f32,
};

@group(0) @binding(0)
var<uniform> u: PaneUniforms;

@group(1) @binding(0)
var pane_texture: texture_2d<f32>;

@group(1) @binding(1)
var tex_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Full-screen triangle (3 vertices, no vertex buffer needed)
@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vi & 1u)) * 4.0 - 1.0;
    let y = f32(i32((vi >> 1u) & 1u)) * 4.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// Signed distance from UV to the pane rectangle edge (negative = inside).
fn sdf_rect(uv: vec2<f32>) -> f32 {
    let pane_min = vec2<f32>(u.pane_x, u.pane_y);
    let pane_max = vec2<f32>(u.pane_x + u.pane_w, u.pane_y + u.pane_h);
    let center = (pane_min + pane_max) * 0.5;
    let half_size = (pane_max - pane_min) * 0.5;
    let d = abs(uv - center) - half_size;
    return length(max(d, vec2<f32>(0.0))) + min(max(d.x, d.y), 0.0);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var color = textureSample(pane_texture, tex_sampler, in.uv);

    // --- Glow border (focused pane only) ---
    if u.is_focused > 0.5 && u.glow_width > 0.0 {
        // Convert glow width from pixels to UV space
        let pixel_size = vec2<f32>(1.0 / u.screen_width, 1.0 / u.screen_height);
        let glow_uv = u.glow_width * max(pixel_size.x, pixel_size.y);

        let dist = sdf_rect(in.uv);

        // Glow only outside the pane (dist > 0) fading over glow_uv distance
        if dist > 0.0 && dist < glow_uv {
            let glow_strength = 1.0 - smoothstep(0.0, glow_uv, dist);
            let glow = vec4<f32>(
                u.glow_color_r,
                u.glow_color_g,
                u.glow_color_b,
                u.glow_color_a * glow_strength
            );
            // Additive blend
            color = vec4<f32>(
                color.rgb + glow.rgb * glow.a,
                max(color.a, glow.a)
            );
        }

        // Subtle inner edge highlight (1px inside the border)
        if dist > -glow_uv * 0.25 && dist <= 0.0 {
            let inner = 1.0 - smoothstep(-glow_uv * 0.25, 0.0, dist);
            let highlight = vec3<f32>(u.glow_color_r, u.glow_color_g, u.glow_color_b);
            color = vec4<f32>(
                mix(color.rgb, color.rgb + highlight * 0.15, inner),
                color.a
            );
        }
    }

    // --- Dim overlay (unfocused panes) ---
    if u.is_focused < 0.5 && u.dim_factor < 0.999 {
        color = vec4<f32>(color.rgb * u.dim_factor, color.a);
    }

    // --- Per-pane scanlines (optional, controlled by scanline_intensity) ---
    if u.scanline_intensity > 0.001 {
        let scan_y = in.uv.y * u.screen_height;
        let scanline = sin(scan_y * 3.14159265) * 0.5 + 0.5;
        color = vec4<f32>(
            color.rgb * (1.0 - u.scanline_intensity * (1.0 - scanline)),
            color.a
        );
    }

    return color;
}
