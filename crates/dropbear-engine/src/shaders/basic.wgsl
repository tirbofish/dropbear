// basic.wgsl - shader used for drawing debug stuff like lines and shii

@group(0) @binding(0) var<uniform> camera: mat4x4<f32>;

struct VertexInput {
    @location(0) position: vec4<f32>, // ignore w value
    @location(1) colour: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) colour: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera * vec4<f32>(in.position.xyz, 1.0);
    out.colour = in.colour;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.colour;
}