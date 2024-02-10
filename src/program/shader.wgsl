struct Uniforms {
    sampling_factor: vec2<f32>,
    size: vec2<f32>,
    image_dimensions: vec2<f32>,
    scale: vec2<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(1) @binding(0) var yuv_texture: texture_2d_array<f32>;
@group(1) @binding(1) var yuv_sampler: sampler;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) pos: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;

    let v_pos = vertex_position(input.vertex_index);
    out.uv = vec2<f32>(v_pos.x * uniforms.image_dimensions.x, 1.0 - (v_pos.y * uniforms.image_dimensions.y));

    var transform: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(uniforms.size.x, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, uniforms.size.y, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(input.pos, 0.0, 1.0)
    );

    out.position = transform * vec4<f32>(v_pos * uniforms.scale * 2.0, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let y = textureSample(yuv_texture, yuv_sampler, input.uv, 0) * 255.0;
    let u = textureSample(yuv_texture, yuv_sampler, input.uv / uniforms.sampling_factor, 1) * 255.0;
    let v = textureSample(yuv_texture, yuv_sampler, input.uv / uniforms.sampling_factor, 2) * 255.0;

    return vec4<f32>(
        clamp((1.164 * (y.x - 16.0) + 1.596 * (v.x - 128.0)) / 255.0, 0.0, 1.0),
        clamp((1.164 * (y.x - 16.0) - 0.813 * (v.x - 128.0) - 0.391 * (u.x - 128.0)) / 255.0, 0.0, 1.0),
        clamp((1.164 * (y.x - 16.0) + 2.018 * (u.x - 128.0)) / 255.0, 0.0, 1.0),
        1.0
    );
}

fn vertex_position(vertex_index: u32) -> vec2<f32> {
    return vec2<f32>((vec2(1u, 2u) + vertex_index) % vec2(6u) < vec2(3u));
}
