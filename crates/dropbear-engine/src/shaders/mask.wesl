struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@group(0) @binding(0)
var s_mask: sampler;
@group(0) @binding(1)
var t_mask: texture_2d<f32>;

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;

    // draw fullscreen triangle
    let x = f32((in_vertex_index << 1u) & 2u);
    let y = f32(in_vertex_index & 2u);
    out.clip_position = vec4<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(x, 1.0-y);
    return out;
}

@fragment
fn fs_mask(in: VertexOutput) {
    let sample = textureSample(t_mask, s_mask, in.uv);
    // We invert this check so that the mask will render objects in
    // the center
    if (sample.a > 0.1) {
        discard;
    }
}

@fragment
fn fs_colour(in: VertexOutput) -> @location(0) vec4<f32> {
    let sample = textureSample(t_mask, s_mask, in.uv);
    return sample;
}