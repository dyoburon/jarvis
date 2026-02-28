// Bloom shader — two-pass Gaussian blur (horizontal + vertical).
//
// Pass 1 (fs_blur_h): reads source texture, blurs horizontally.
// Pass 2 (fs_blur_v): reads h-blurred texture, blurs vertically.
// Both use a 9-tap Gaussian kernel for smooth light bleed.

struct BloomUniforms {
    texel_size: vec2<f32>,  // 1.0 / texture_dimensions
    intensity: f32,
    _padding: f32,
};

@group(0) @binding(0)
var<uniform> bloom: BloomUniforms;

@group(0) @binding(1)
var source_texture: texture_2d<f32>;

@group(0) @binding(2)
var source_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

// Full-screen triangle vertex shader (shared by both passes).
@vertex
fn vs_main(@builtin(vertex_index) vi: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(i32(vi & 1u)) * 4.0 - 1.0;
    let y = f32(i32((vi >> 1u) & 1u)) * 4.0 - 1.0;
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.uv = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

// 9-tap Gaussian weights (sigma ≈ 2.0, normalized).
const WEIGHTS: array<f32, 5> = array<f32, 5>(
    0.227027, 0.1945946, 0.1216216, 0.054054, 0.016216
);

// Horizontal blur pass.
@fragment
fn fs_blur_h(in: VertexOutput) -> @location(0) vec4<f32> {
    var result = textureSample(source_texture, source_sampler, in.uv) * WEIGHTS[0];

    for (var i = 1; i < 5; i = i + 1) {
        let offset = vec2<f32>(bloom.texel_size.x * f32(i), 0.0);
        result += textureSample(source_texture, source_sampler, in.uv + offset) * WEIGHTS[i];
        result += textureSample(source_texture, source_sampler, in.uv - offset) * WEIGHTS[i];
    }

    return result * bloom.intensity;
}

// Vertical blur pass.
@fragment
fn fs_blur_v(in: VertexOutput) -> @location(0) vec4<f32> {
    var result = textureSample(source_texture, source_sampler, in.uv) * WEIGHTS[0];

    for (var i = 1; i < 5; i = i + 1) {
        let offset = vec2<f32>(0.0, bloom.texel_size.y * f32(i));
        result += textureSample(source_texture, source_sampler, in.uv + offset) * WEIGHTS[i];
        result += textureSample(source_texture, source_sampler, in.uv - offset) * WEIGHTS[i];
    }

    return result * bloom.intensity;
}
