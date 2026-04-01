import super::common::animation;
import super::common::material;
import super::common::environment;

struct Globals {
    num_lights:       u32,
    ambient_strength: f32,
}

struct CameraUniform {
    view_pos:  vec4<f32>,
    view:      mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inv_proj:  mat4x4<f32>,
    inv_view:  mat4x4<f32>,
}

struct Light {
    position:  vec4<f32>,
    direction: vec4<f32>, // x, y, z, outer_cutoff_angle
    color:     vec4<f32>, // r, g, b, light_type (0=directional, 1=point, 2=spot)
    constant:  f32,
    lin:       f32,
    quadratic: f32,
    cutoff:    f32,
}

// per-frame
@group(0) @binding(0) var<uniform>       u_globals:     Globals;
@group(0) @binding(1) var<uniform>       u_camera:      CameraUniform;
@group(0) @binding(2) var<storage, read> s_light_array: array<Light>;

struct InstanceInput {
    @location(8)  model_matrix_0: vec4<f32>,
    @location(9)  model_matrix_1: vec4<f32>,
    @location(10) model_matrix_2: vec4<f32>,
    @location(11) model_matrix_3: vec4<f32>,

    @location(12) normal_matrix_0: vec3<f32>,
    @location(13) normal_matrix_1: vec3<f32>,
    @location(14) normal_matrix_2: vec3<f32>,
};

struct VertexInput {
    @builtin(vertex_index) vertex_id:  u32,
    @location(0) position:   vec3<f32>,
    @location(1) normal:     vec3<f32>,
    @location(2) tangent:    vec4<f32>,
    @location(3) tex_coords0: vec2<f32>,
    @location(4) tex_coords1: vec2<f32>,
    @location(5) colour0:    vec4<f32>,
    @location(6) joints:     vec4<u32>,
    @location(7) weights:    vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position:     vec4<f32>,
    @location(0) tex_coords:              vec2<f32>,
    @location(1) world_normal:            vec3<f32>,
    @location(2) world_position:          vec3<f32>,
    @location(3) world_tangent:           vec3<f32>,
    @location(4) world_bitangent:         vec3<f32>,
    @location(5) world_view_position:     vec3<f32>,
};



@vertex
fn vs_main(model: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    let normal_matrix = mat3x3<f32>(
        instance.normal_matrix_0,
        instance.normal_matrix_1,
        instance.normal_matrix_2,
    );

    let anim = animation::apply_animation(
        model.position,
        model.normal,
        model.tangent.xyz,
        model.joints,
        model.weights,
        model.vertex_id,
    );

    let world_position  = model_matrix * vec4<f32>(anim.position, 1.0);
    let world_normal    = normalize(normal_matrix * anim.normal);
    let world_tangent   = normalize(normal_matrix * anim.tangent);
    let world_bitangent = normalize(cross(world_normal, world_tangent) * model.tangent.w);

    var out: VertexOutput;
    out.clip_position       = u_camera.view_proj * world_position;
    out.tex_coords          = model.tex_coords0;
    out.world_normal        = world_normal;
    out.world_position      = world_position.xyz;
    out.world_tangent       = world_tangent;
    out.world_bitangent     = world_bitangent;
    out.world_view_position = u_camera.view_pos.xyz;
    return out;
}

@fragment
fn s_fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.tex_coords * material::u_material.uv_tiling;

    return textureSample(material::t_diffuse, material::s_diffuse, uv);
}