@group(0) @binding(0)
var<uniform> transform: mat4x4<f32>;

@group(0) @binding(1)
var<uniform> projection: mat4x4<f32>;

@group(0) @binding(2)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(3)
var s_diffuse: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(
    @location(0) inPos: vec3<f32>,
    @location(1) inTexCoords: vec2<f32>
) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = projection * transform * vec4<f32>(inPos, 1.0);
    out.tex_coords = inTexCoords;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSampleLevel(t_diffuse, s_diffuse, in.tex_coords, 0);
}