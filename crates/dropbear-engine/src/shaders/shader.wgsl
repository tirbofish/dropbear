const PI: f32 = 3.14159265358979;

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
    out.clip_position       = u_camera.view_proj * world_position;
    out.tex_coords          = model.tex_coords0;
    out.world_normal        = world_normal;
    out.world_position      = world_position.xyz;
    out.world_tangent       = world_tangent;
    out.world_bitangent     = world_bitangent;
    out.world_view_position = u_camera.view_pos.xyz;
    return out;
}

fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a  = roughness * roughness;
    let a2 = a * a;
    let d  = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / (PI * d * d);
}

fn geometry_schlick_ggx(n_dot_x: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return n_dot_x / (n_dot_x * (1.0 - k) + k);
}

fn geometry_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    return geometry_schlick_ggx(n_dot_v, roughness)
         * geometry_schlick_ggx(n_dot_l, roughness);
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (1.0 - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn fresnel_schlick_roughness(cos_theta: f32, f0: vec3<f32>, roughness: f32) -> vec3<f32> {
    let r1 = vec3<f32>(1.0 - roughness);
    return f0 + (max(r1, f0) - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

fn pbr_direct(
    n:         vec3<f32>,
    v:         vec3<f32>,
    l:         vec3<f32>,
    albedo:    vec3<f32>,
    f0:        vec3<f32>,
    roughness: f32,
    metallic:  f32,
) -> vec3<f32> {
    let h = normalize(v + l);

    let n_dot_v = max(dot(n, v), 0.0001);
    let n_dot_l = max(dot(n, l), 0.0);
    let n_dot_h = max(dot(n, h), 0.0);
    let h_dot_v = max(dot(h, v), 0.0);

    let D = distribution_ggx(n_dot_h, roughness);
    let G = geometry_smith(n_dot_v, n_dot_l, roughness);
    let F = fresnel_schlick(h_dot_v, f0);

    let numerator   = D * G * F;
    let denominator = 4.0 * n_dot_v * n_dot_l + 0.0001;
    let specular    = numerator / denominator;

    let k_s = F;
    let k_d = (1.0 - k_s) * (1.0 - metallic);

    return (k_d * albedo / PI + specular) * n_dot_l;
}

fn directional_light_pbr(
    light:     Light,
    n:         vec3<f32>,
    v:         vec3<f32>,
    albedo:    vec3<f32>,
    f0:        vec3<f32>,
    roughness: f32,
    metallic:  f32,
) -> vec3<f32> {
    let l = normalize(light.direction.xyz);
    return pbr_direct(n, v, l, albedo, f0, roughness, metallic) * light.color.rgb;
}

fn point_light_pbr(
    light:     Light,
    n:         vec3<f32>,
    v:         vec3<f32>,
    albedo:    vec3<f32>,
    f0:        vec3<f32>,
    roughness: f32,
    metallic:  f32,
    world_pos: vec3<f32>,
) -> vec3<f32> {
    let to_light = light.position.xyz - world_pos;
    let dist     = length(to_light);
    let l        = to_light / dist;
    let atten    = 1.0 / (light.constant + light.lin * dist + light.quadratic * dist * dist);
    return pbr_direct(n, v, l, albedo, f0, roughness, metallic) * light.color.rgb * atten;
}

fn spot_light_pbr(
    light:     Light,
    n:         vec3<f32>,
    v:         vec3<f32>,
    albedo:    vec3<f32>,
    f0:        vec3<f32>,
    roughness: f32,
    metallic:  f32,
    world_pos: vec3<f32>,
) -> vec3<f32> {
    let to_light = light.position.xyz - world_pos;
    let dist     = length(to_light);
    let l        = to_light / dist;
    let atten    = 1.0 / (light.constant + light.lin * dist + light.quadratic * dist * dist);

    let spot_dir   = normalize(light.direction.xyz);
    let theta      = dot(-l, spot_dir);
    let cone_factor = smoothstep(light.direction.w, light.cutoff, theta);

    return pbr_direct(n, v, l, albedo, f0, roughness, metallic)
         * light.color.rgb * atten * cone_factor;
}

fn ibl(
    n:            vec3<f32>,
    v:            vec3<f32>,
    albedo:       vec3<f32>,
    f0:           vec3<f32>,
    roughness:    f32,
    metallic:     f32,
) -> vec3<f32> {
    let n_dot_v = max(dot(n, v), 0.0001);
    let num_mips = f32(textureNumLevels(env_map));

    // fresnel for smooth roughness fade
    let F = fresnel_schlick_roughness(n_dot_v, f0, roughness);

    let k_s = F;
    let k_d = (1.0 - k_s) * (1.0 - metallic);

    // diffuse irradiance: sample the most-blurred mip to approximate
    // hemisphere-integrated radiance
    let irradiance  = textureSampleLevel(env_map, env_sampler, n, num_mips - 1.0).rgb;
    let diffuse_ibl = k_d * albedo * irradiance;

    // specular radiance: rougher materials sample higher (blurrier) mip levels
    let r              = reflect(-v, n);
    let specular_mip   = roughness * roughness * (num_mips - 1.0);
    let prefiltered    = textureSampleLevel(env_map, env_sampler, r, specular_mip).rgb;

    // Analytic BRDF integration approximation (no LUT needed)
    let env_brdf_x = exp(-6.9 * roughness * roughness * n_dot_v);
    let env_brdf   = F * (1.0 - env_brdf_x) + f0 * env_brdf_x;
    let specular_ibl = prefiltered * env_brdf;

    return diffuse_ibl + specular_ibl;
}

fn get_normal(in: VertexOutput, uv: vec2<f32>) -> vec3<f32> {
    if u_material.has_normal_texture == 0u {
        return normalize(in.world_normal);
    }

    let raw = textureSample(t_normal, s_normal, uv).rgb;
    var tangent_normal = raw * 2.0 - 1.0;

    tangent_normal = vec3<f32>(
        tangent_normal.xy * u_material.normal_scale,
        tangent_normal.z,
    );

    let t   = normalize(in.world_tangent);
    let b   = normalize(in.world_bitangent);
    let nm  = normalize(in.world_normal);
    let tbn = mat3x3<f32>(t, b, nm);

    return normalize(tbn * tangent_normal);
}

@fragment
fn s_fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.tex_coords * u_material.uv_tiling;

    // albedo + alpha
    let albedo_sample = textureSample(t_diffuse, s_diffuse, uv);
    if albedo_sample.a < u_material.alpha_cutoff {
        discard;
    }

    let albedo = albedo_sample.rgb * u_material.base_colour.rgb;

    // metallic / roughness
    // glTF convention: B = metallic, G = roughness (R = occlusion when packed).
    var metallic  = u_material.metallic;
    var roughness = u_material.roughness;
    if u_material.has_metallic_texture != 0u {
        let mr_sample = textureSample(t_metallic, s_metallic, uv);
        metallic  *= mr_sample.b;
        roughness *= mr_sample.g;
    }
    roughness = clamp(roughness, 0.04, 1.0);

    // occlusion
    var occlusion = 1.0;
    if u_material.has_occlusion_texture != 0u {
        let occ_sample = textureSample(t_occlusion, s_occlusion, uv);
        occlusion = mix(1.0, occ_sample.r, u_material.occlusion_strength);
    }

    let n   = get_normal(in, uv);
    let v   = normalize(u_camera.view_pos.xyz - in.world_position);

    // F0: dielectrics use 0.04, metals use their albedo colour
    let f0 = mix(vec3<f32>(0.04), albedo, metallic);

    // cook-torence
    var lo = vec3<f32>(0.0);
    for (var i = 0u; i < u_globals.num_lights; i++) {
        let light      = s_light_array[i];
        let light_type = u32(light.color.w);

        if light_type == 0u {
            lo += directional_light_pbr(light, n, v, albedo, f0, roughness, metallic);
        } else if light_type == 1u {
            lo += point_light_pbr(light, n, v, albedo, f0, roughness, metallic, in.world_position);
        } else if light_type == 2u {
            lo += spot_light_pbr(light, n, v, albedo, f0, roughness, metallic, in.world_position);
        }
    }

    // image-based lighting (env map)
    let ambient_ibl = ibl(n, v, albedo, f0, roughness, metallic)
                    * occlusion
                    * u_globals.ambient_strength;

    // emissive
    var emissive = u_material.emissive * u_material.emissive_strength;
    if u_material.has_emissive_texture != 0u {
        emissive *= textureSample(t_emissive, s_emissive, uv).rgb;
    }

    // combine
    let colour = lo + ambient_ibl + emissive;
    
    return vec4<f32>(colour, albedo_sample.a);
}