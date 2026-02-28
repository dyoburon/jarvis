pub(crate) const SHADER_SOURCE: &str = r#"
struct Uniforms {
    resolution: vec2<f32>,
    _pad: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct InstanceInput {
    @location(1) rect: vec4<f32>,
    @location(2) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
};

@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    var out: VertexOutput;

    // Scale unit quad by instance rect size and translate by instance position
    let pixel_x = instance.rect.x + vertex.position.x * instance.rect.z;
    let pixel_y = instance.rect.y + vertex.position.y * instance.rect.w;

    // Convert from pixel coordinates to NDC (-1..1)
    let ndc_x = (pixel_x / uniforms.resolution.x) * 2.0 - 1.0;
    let ndc_y = 1.0 - (pixel_y / uniforms.resolution.y) * 2.0;

    out.clip_position = vec4<f32>(ndc_x, ndc_y, 0.0, 1.0);
    out.color = instance.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
"#;
