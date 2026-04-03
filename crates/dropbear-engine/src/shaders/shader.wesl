import super::common::animation;
import super::common::material;
import super::common::environment;
import super::common::light;

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


// per-frame
@group(0) @binding(0) var<uniform>       u_globals:     Globals;
@group(0) @binding(1) var<uniform>       u_camera:      CameraUniform;

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
    @location(1) world_position:          vec3<f32>,
    @location(2) world_tangent:           vec3<f32>,
    @location(3) world_bitangent:         vec3<f32>,
    @location(4) world_normal:            vec3<f32>,
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
    let model_mat3      = mat3x3<f32>(model_matrix[0].xyz, model_matrix[1].xyz, model_matrix[2].xyz);
    let world_tangent   = normalize(model_mat3 * anim.tangent);
    let world_bitangent = normalize(cross(world_normal, world_tangent) * model.tangent.w);

    var out: VertexOutput;
    out.clip_position   = u_camera.view_proj * world_position;
    out.tex_coords      = model.tex_coords0;
    out.world_position  = world_position.xyz;
    out.world_tangent   = world_tangent;
    out.world_bitangent = world_bitangent;
    out.world_normal    = world_normal;
    return out;
}

@fragment
fn s_fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.tex_coords * material::u_material.uv_tiling + material::u_material.uv_offset;

    let albedo = textureSample(material::t_diffuse, material::s_diffuse, uv)
        * material::u_material.base_colour;

    if material::u_material.alpha_mode == material::ALPHAMODE_MASK && albedo.a < material::u_material.alpha_cutoff {
        discard;
    }

    let world_N = normalize(in.world_normal);

    let world_T = normalize(in.world_tangent - dot(in.world_tangent, world_N) * world_N);
    let world_B_cross = cross(world_N, world_T);
    let world_B = world_B_cross * sign(dot(in.world_bitangent, world_B_cross));
    let tangent_matrix = transpose(mat3x3<f32>(world_T, world_B, world_N));

    var N: vec3<f32>;
    if material::u_material.has_normal_texture != 0u {
        let n_sample = textureSample(material::t_normal, material::s_normal, uv).xyz;
        N = normalize((n_sample * 2.0 - 1.0) * vec3<f32>(material::u_material.normal_scale, material::u_material.normal_scale, 1.0));
    } else {
        N = vec3<f32>(0.0, 0.0, 1.0);
    }

    // occlusion
    var occlusion = 1.0;
    if material::u_material.has_occlusion_texture != 0u {
        let occ_sample = textureSample(material::t_occlusion, material::s_occlusion, uv).r;
        occlusion = 1.0 + material::u_material.occlusion_strength * (occ_sample - 1.0);
    }

    let roughness = material::u_material.roughness;
    let shininess = max(2.0 / max(roughness * roughness, 0.001) - 2.0, 2.0);
    let spec_scale = max((1.0 - roughness) * (1.0 - roughness), 0.04);

    let tangent_pos      = tangent_matrix * in.world_position;
    let tangent_view_pos = tangent_matrix * u_camera.view_pos.xyz;

    let view_dir = normalize(tangent_view_pos - tangent_pos);
    let lit = light::calculate_lighting(
        tangent_pos,
        tangent_matrix,
        N,
        view_dir,
        albedo.rgb,
        shininess,
        spec_scale,
    ) * occlusion;

    var emissive = material::u_material.emissive_factor * material::u_material.emissive_strength;
    if material::u_material.has_emissive_texture != 0u {
        emissive *= textureSample(material::t_emissive, material::s_emissive, uv).rgb;
    }

    return vec4<f32>(lit + emissive, albedo.a);
}