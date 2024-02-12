struct Uniforms {
    sampling_factor: vec2<f32>,
    size: vec2<f32>,
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
    out.uv = vec2<f32>(v_pos.x * uniforms.scale.x, 1.0 - (v_pos.y * uniforms.scale.y));

    var transform: mat4x4<f32> = mat4x4<f32>(
        vec4<f32>(uniforms.size.x, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, uniforms.size.y, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(input.pos, 0.0, 1.0)
    );

    out.position = transform * vec4<f32>(v_pos, 0.0, 1.0);

    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // https://learn.microsoft.com/en-us/windows/win32/medfound/recommended-8-bit-yuv-formats-for-video-rendering#converting-8-bit-yuv-to-rgb888
    let c = textureSample(yuv_texture, yuv_sampler, input.uv, 0).x - 0.062745;
    let d = textureSample(yuv_texture, yuv_sampler, input.uv / uniforms.sampling_factor, 1).x - 0.5;
    let e = textureSample(yuv_texture, yuv_sampler, input.uv / uniforms.sampling_factor, 2).x - 0.5;

    return vec4<f32>(
        clamp(c + 1.596027 * e, 0.0, 1.0),
        clamp(c - 0.391762 * d - 0.812968 * e, 0.0, 1.0),
        clamp(c + 2.017232 * d, 0.0, 1.0),
        1.0
    );
}

fn vertex_position(vertex_index: u32) -> vec2<f32> {
    return vec2<f32>((vec2(1u, 2u) + vertex_index) % vec2(6u) < vec2(3u));
}
