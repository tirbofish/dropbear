struct Globals {
    num_lights: u32,
    ambient_strength: f32,
}

struct CameraUniform {
    view_pos: vec4<f32>,
    view: mat4x4<f32>,
    view_proj: mat4x4<f32>,
    inv_proj: mat4x4<f32>,
    inv_view: mat4x4<f32>,
}

struct Light {
    position: vec4<f32>,
    direction: vec4<f32>, // x, y, z, outer_cutoff_angle
    color: vec4<f32>,     // r, g, b, light_type (0=directional, 1=point, 2=spot)
    constant: f32,
    lin: f32,
    quadratic: f32,
    cutoff: f32,
}

struct MaterialUniform {
    base_colour: vec4<f32>,
    emissive: vec3<f32>,
    emissive_strength: f32,
    metallic: f32,
    roughness: f32,
    normal_scale: f32,
    occlusion_strength: f32,
    alpha_cutoff: f32,
    uv_tiling: vec2<f32>,

    has_normal_texture: u32,
    has_emissive_texture: u32,
    has_metallic_texture: u32,
    has_occlusion_texture: u32,
}

struct MorphTargetInfo {
    num_vertices: u32,
    num_targets: u32,
    base_offset: u32,
    weight_offset: u32,
    uses_morph: u32,
}

// per-frame
@group(0) @binding(0)
var<uniform> u_globals: Globals;
@group(0) @binding(1)
var<uniform> u_camera: CameraUniform;
@group(0) @binding(2)
var<storage, read> s_light_array: array<Light>;

// per-material
@group(1) @binding(0)
var<uniform> u_material: MaterialUniform;
@group(1) @binding(1)
var t_diffuse: texture_2d<f32>;
@group(1) @binding(2)
var s_diffuse: sampler;
@group(1) @binding(3)
var t_normal: texture_2d<f32>;
@group(1) @binding(4)
var s_normal: sampler;
@group(1) @binding(5)
var t_emissive: texture_2d<f32>;
@group(1) @binding(6)
var s_emissive: sampler;
@group(1) @binding(7)
var t_metallic: texture_2d<f32>;
@group(1) @binding(8)
var s_metallic: sampler;
@group(1) @binding(9)
var t_occlusion: texture_2d<f32>;
@group(1) @binding(10)
var s_occlusion: sampler;

// animation
@group(2) @binding(0)
var<storage, read> s_skinning: array<mat4x4<f32>>;
@group(2) @binding(1)
var<storage, read> s_morph_deltas: array<f32>;
@group(2) @binding(2)
var<storage, read> s_morph_weights: array<f32>;
@group(2) @binding(3)
var<uniform> u_morph_info: MorphTargetInfo;

// environment
@group(3) @binding(0)
var env_map: texture_cube<f32>;
@group(3) @binding(1)
var env_sampler: sampler;

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
    @builtin(vertex_index) vertex_id: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) tex_coords0: vec2<f32>,
    @location(4) tex_coords1: vec2<f32>,
    @location(5) colour0: vec4<f32>,
    @location(6) joints: vec4<u32>,
    @location(7) weights: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) world_tangent: vec3<f32>,
    @location(4) world_bitangent: vec3<f32>,
    @location(5) world_view_position: vec3<f32>,
};

fn apply_morph(base_pos: vec3<f32>, vertex_id: u32) -> vec3<f32> {
    var result = base_pos;
    for (var t = 0u; t < u_morph_info.num_targets; t++) {
        let weight = s_morph_weights[u_morph_info.weight_offset + t];
        let idx = u_morph_info.base_offset + (t * u_morph_info.num_vertices + vertex_id) * 3u;
        let delta = vec3<f32>(
            s_morph_deltas[idx],
            s_morph_deltas[idx + 1u],
            s_morph_deltas[idx + 2u],
        );
        result += delta * weight;
    }
    return result;
}

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

    var skin_matrix = mat4x4<f32>(
        vec4<f32>(1.0, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 1.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 1.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );
    if (dot(model.weights, vec4<f32>(1.0)) > 0.0) {
        let j = model.joints;
        let w = model.weights;
        skin_matrix =
            s_skinning[j.x] * w.x +
            s_skinning[j.y] * w.y +
            s_skinning[j.z] * w.z +
            s_skinning[j.w] * w.w;
    }

    let morphed_pos = select(
        model.position,
        apply_morph(model.position, model.vertex_id),
        u_morph_info.uses_morph != 0u,
    );
    let world_position = model_matrix * skin_matrix * vec4<f32>(morphed_pos, 1.0);

    let skin_normal  = (skin_matrix * vec4<f32>(model.normal,       0.0)).xyz;
    let skin_tangent = (skin_matrix * vec4<f32>(model.tangent.xyz,  0.0)).xyz;

    let world_normal    = normalize(normal_matrix * skin_normal);
    let world_tangent   = normalize(normal_matrix * skin_tangent);
    let world_bitangent = normalize(cross(world_normal, world_tangent) * model.tangent.w);

    var out: VertexOutput;
    out.clip_position      = u_camera.view_proj * world_position;
    out.tex_coords         = model.tex_coords0;
    out.world_normal       = world_normal;
    out.world_position     = world_position.xyz;
    out.world_tangent      = world_tangent;
    out.world_bitangent    = world_bitangent;
    out.world_view_position = u_camera.view_pos.xyz;
    return out;
}


fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn apply_normal_map(
    world_normal_in:    vec3<f32>,
    world_tangent_in:   vec3<f32>,
    world_bitangent_in: vec3<f32>,
    normal_sample_raw:  vec3<f32>,
    normal_scale:       f32,
) -> vec3<f32> {
    let unpacked = normalize((normal_sample_raw * 2.0 - 1.0) * vec3<f32>(normal_scale, normal_scale, 1.0));

    let n = normalize(world_normal_in);
    var t = normalize(world_tangent_in);
    // Gram-Schmidt re-orthogonalise
    t = normalize(t - n * dot(n, t));

    let b_in     = normalize(world_bitangent_in);
    let handedness = select(-1.0, 1.0, dot(cross(n, t), b_in) >= 0.0);
    let b        = cross(n, t) * handedness;

    let tbn = mat3x3<f32>(t, b, n);
    return normalize(tbn * unpacked);
}

fn eval_brdf(
    light_dir:   vec3<f32>,
    world_normal: vec3<f32>,
    view_dir:    vec3<f32>,
    light_color: vec3<f32>,
    tex_color:   vec3<f32>,
    metallic:    f32,
    roughness:   f32,
) -> vec3<f32> {
    let r = clamp(roughness, 0.05, 1.0);

    let n_dot_l  = max(dot(world_normal, light_dir), 0.0);
    let diffuse  = light_color * n_dot_l * tex_color * (1.0 - metallic);

    let half_dir = normalize(light_dir + view_dir);
    let n_dot_h  = max(dot(world_normal, half_dir), 0.0);
    let shininess = pow(2.0, (1.0 - r) * 10.0); // maps [0,1] roughness → [2, 1024]
    let spec_val  = pow(n_dot_h, shininess);

    let f0       = mix(vec3<f32>(0.04), tex_color, metallic);
    let n_dot_v  = max(dot(world_normal, view_dir), 0.0);
    let fresnel  = fresnel_schlick(max(dot(half_dir, view_dir), 0.0), f0);
    let specular = light_color * spec_val * fresnel;

    return diffuse + specular;
}

fn directional_light(
    light:        Light,
    world_normal: vec3<f32>,
    view_dir:     vec3<f32>,
    tex_color:    vec3<f32>,
    metallic:     f32,
    roughness:    f32,
) -> vec3<f32> {
    let light_dir = normalize(-light.direction.xyz);
    return eval_brdf(light_dir, world_normal, view_dir, light.color.xyz, tex_color, metallic, roughness);
}

fn point_light(
    light:        Light,
    world_pos:    vec3<f32>,
    world_normal: vec3<f32>,
    view_dir:     vec3<f32>,
    tex_color:    vec3<f32>,
    metallic:     f32,
    roughness:    f32,
) -> vec3<f32> {
    let to_light  = light.position.xyz - world_pos;
    let distance  = length(to_light);
    let light_dir = normalize(to_light);

    let attenuation = 1.0 / (light.constant + light.lin * distance + light.quadratic * (distance * distance));

    return eval_brdf(light_dir, world_normal, view_dir, light.color.xyz, tex_color, metallic, roughness)
           * attenuation;
}

fn spot_light(
    light:        Light,
    world_pos:    vec3<f32>,
    world_normal: vec3<f32>,
    view_dir:     vec3<f32>,
    tex_color:    vec3<f32>,
    metallic:     f32,
    roughness:    f32,
) -> vec3<f32> {
    let to_light     = light.position.xyz - world_pos;
    let distance     = length(to_light);
    let light_dir    = normalize(to_light);

    let theta        = dot(light_dir, normalize(-light.direction.xyz));
    let outer_cutoff = light.direction.w;
    let epsilon      = light.cutoff - outer_cutoff;
    let intensity    = clamp((theta - outer_cutoff) / epsilon, 0.0, 1.0);

    let attenuation  = 1.0 / (light.constant + light.lin * distance + light.quadratic * (distance * distance));

    return eval_brdf(light_dir, world_normal, view_dir, light.color.xyz, tex_color, metallic, roughness)
           * attenuation * intensity;
}

@fragment
fn s_fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.tex_coords * u_material.uv_tiling;

    // base colour
    let tex_color  = textureSample(t_diffuse, s_diffuse, uv);
    let base_colour = tex_color * u_material.base_colour;
    if (base_colour.a < u_material.alpha_cutoff) {
        discard;
    }

    // normal
    var world_normal: vec3<f32>;
    if (u_material.has_normal_texture != 0u) {
        let normal_sample_raw = textureSample(t_normal, s_normal, uv).xyz;
        world_normal = apply_normal_map(
            in.world_normal,
            in.world_tangent,
            in.world_bitangent,
            normal_sample_raw,
            u_material.normal_scale,
        );
    } else {
        world_normal = normalize(in.world_normal);
    }

    // metallic and roughness
    var metallic  = u_material.metallic;
    var roughness = u_material.roughness;
    if (u_material.has_metallic_texture != 0u) {
        let mr    = textureSample(t_metallic, s_metallic, uv);
        metallic  *= mr.b;
        roughness *= mr.g;
    }
    metallic  = clamp(metallic,  0.0, 1.0);
    roughness = clamp(roughness, 0.05, 1.0);

    // occlusion
    var occlusion = 1.0;
    if (u_material.has_occlusion_texture != 0u) {
        let occ   = textureSample(t_occlusion, s_occlusion, uv).r;
        occlusion = 1.0 + u_material.occlusion_strength * (occ - 1.0);
    }

    // emissive
    var emissive = u_material.emissive * u_material.emissive_strength;
    if (u_material.has_emissive_texture != 0u) {
        let emissive_tex = textureSample(t_emissive, s_emissive, uv).rgb;
        emissive *= emissive_tex;
    }

    let view_dir = normalize(u_camera.view_pos.xyz - in.world_position);

    // ambient
    let ambient = u_globals.ambient_strength * base_colour.rgb * occlusion;
    var final_color = ambient;

    for (var i = 0u; i < u_globals.num_lights; i += 1u) {
        let light      = s_light_array[i];
        let light_type = i32(light.color.w + 0.1);

        if (light_type == 0) {
            final_color += directional_light(light, world_normal, view_dir, base_colour.rgb, metallic, roughness);
        } else if (light_type == 1) {
            final_color += point_light(light, in.world_position, world_normal, view_dir, base_colour.rgb, metallic, roughness);
        } else if (light_type == 2) {
            final_color += spot_light(light, in.world_position, world_normal, view_dir, base_colour.rgb, metallic, roughness);
        }
    }

    let world_reflect      = reflect(-view_dir, world_normal);
    let reflection         = textureSample(env_map, env_sampler, world_reflect).rgb;
    let n_dot_v            = max(dot(world_normal, view_dir), 0.0);
    let f0                 = mix(vec3<f32>(0.04), base_colour.rgb, metallic);
    let fresnel            = fresnel_schlick(n_dot_v, f0);
    let reflection_weight  = fresnel * pow(1.0 - roughness, 2.0) * mix(vec3<f32>(1.0), base_colour.rgb, metallic);
    final_color           += reflection * reflection_weight;

    final_color += emissive;

    return vec4<f32>(final_color, base_colour.a);
}