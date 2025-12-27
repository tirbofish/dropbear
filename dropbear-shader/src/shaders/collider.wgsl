struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var <uniform> camera: CameraUniform;

struct ColliderUniform {
    model_matrix: mat4x4<f32>,
    color: vec4<f32>,
}

@group(1) @binding(0)
var <uniform> collider: ColliderUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;

    let world_position = collider.model_matrix * vec4<f32>(input.position, 1.0);
    output.clip_position = camera.view_proj * world_position;

    output.color = collider.color;

    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return input.color;
}