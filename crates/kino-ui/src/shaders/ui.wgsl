@group(0) @binding(0)
var texture_binding: texture_2d<f32>;

@group(0) @binding(1)
var texture_sampler: sampler;

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) colour: vec4<f32>,
    @location(2) tex_coords: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) colour: vec4<f32>,
    @location(1) tex_coords: vec2<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    output.position = camera.view_proj * vec4<f32>(input.position, 0.0, 1.0);
    output.colour = input.colour;
    output.tex_coords = input.tex_coords;
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(texture_binding, texture_sampler, input.tex_coords) * input.colour;
}