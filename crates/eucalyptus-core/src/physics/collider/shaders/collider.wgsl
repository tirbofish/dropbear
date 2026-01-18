struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var <uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) model_0: vec4<f32>,
    @location(2) model_1: vec4<f32>,
    @location(3) model_2: vec4<f32>,
    @location(4) model_3: vec4<f32>,
    @location(5) color: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let model = mat4x4<f32>(
        input.model_0,
        input.model_1,
        input.model_2,
        input.model_3,
    );

    let world_position = model * vec4<f32>(input.position, 1.0);
    output.clip_position = camera.view_proj * world_position;

    output.color = input.color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}