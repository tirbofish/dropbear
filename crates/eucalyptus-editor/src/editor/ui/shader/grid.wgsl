struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    viewport_size: vec2<f32>,
    _padding: vec2<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

fn grid_alpha(pixel_uv: vec2<f32>, cell_size: f32, line_width: f32) -> f32 {
    let uv = pixel_uv / cell_size;
    let derivative = fwidth(uv) * line_width;
    let grid = abs(fract(uv - 0.5) - 0.5) / derivative;
    let l = min(grid.x, grid.y);
    return 1.0 - clamp(l, 0.0, 1.0);
}

fn axis_alpha(v: f32, line_width: f32) -> f32 {
    let d = abs(v);
    let fw = max(fwidth(v), 1e-5) * line_width;
    return 1.0 - smoothstep(0.0, fw, d);
}

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> @builtin(position) vec4<f32> {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(3.0, -1.0),
        vec2<f32>(-1.0, 3.0),
    );

    let position = positions[vertex_index];
    return vec4<f32>(position, 0.0, 1.0);
}

@fragment
fn fs_main(@builtin(position) frag_pos: vec4<f32>) -> @location(0) vec4<f32> {
    let ndc = vec2<f32>(
        (frag_pos.x / camera.viewport_size.x) * 2.0 - 1.0,
        1.0 - (frag_pos.y / camera.viewport_size.y) * 2.0,
    );

    let world = camera.inv_view_proj * vec4<f32>(ndc, 0.0, 1.0);
    let world_pos = world.xy / world.w;

    let fine = grid_alpha(world_pos, 10.0, 0.8);
    let coarse = grid_alpha(world_pos, 100.0, 1.0);
    let axis_x = axis_alpha(world_pos.y, 1.8);
    let axis_y = axis_alpha(world_pos.x, 1.8);
    let axis = max(axis_x, axis_y);

    let fine_color = vec3<f32>(0.25, 0.25, 0.25);
    let coarse_color = vec3<f32>(0.38, 0.38, 0.38);

    var color = vec3<f32>(0.13, 0.13, 0.13);

    var alpha = 0.0;
    alpha = max(alpha, fine * 0.6);
    color = mix(color, fine_color, fine * 0.6);
    alpha = max(alpha, coarse);
    color = mix(color, coarse_color, coarse);

    color = mix(color, vec3<f32>(1.0, 1.0, 1.0), axis);
    alpha = max(alpha, axis);

    return vec4<f32>(color, alpha);
}