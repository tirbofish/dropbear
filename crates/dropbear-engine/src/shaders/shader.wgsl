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

fn blinn_phong(
    n: vec3<f32>,
    l: vec3<f32>,
    v: vec3<f32>,
    albedo: vec3<f32>,
    light_colour: vec3<f32>,
) -> vec3<f32> {
    // diffuse
    // 1.0 = light hits head-on
    // 0.0 = grazing
    // negative = behind
    let ndotl = max(dot(n, l), 0.0);
    let diffuse = albedo * light_colour * ndotl;

    // specular
    let h = normalize(l + v);
    let ndoth = max(dot(n, h), 0.0);
    let specular = light_colour * pow(ndoth, 32.0); // 32.0 = shininess

    return diffuse + specular;
}

// l will always be the same
fn directional_light(
    light: Light,
    n: vec3<f32>,
    v: vec3<f32>,
    albedo: vec3<f32>,
) -> vec3<f32> {
    // direction stored in light points towards light source
    let l = normalize(light.direction.xyz);
    let light_colour = light.color.rgb;
    return blinn_phong(n, l, v, albedo, light_colour);
}

// light comes from a position and falls off with distance
fn point_light(
    light: Light,
    n: vec3<f32>,
    v: vec3<f32>,
    albedo: vec3<f32>,
    world_pos: vec3<f32>,
) -> vec3<f32> {
    let to_light = light.position.xyz - world_pos;
    let l = normalize(to_light);
    let dist = length(to_light);

    let attenuation = 1.0 / (light.constant + light.lin * dist + light.quadratic * pow(dist, 2));
    
    let light_colour = light.color.rgb * attenuation;
    return blinn_phong(n, l, v, albedo, light_colour);
}

// same as point light, except in the shape of a cone and falls off with further distance
fn spot_light(
    light: Light,
    n: vec3<f32>,
    v: vec3<f32>,
    albedo: vec3<f32>,
    world_pos: vec3<f32>,
) -> vec3<f32> {
    let to_light = light.position.xyz - world_pos;
    let l = normalize(to_light);
    let dist = length(to_light);

    let attenuation = 1.0 / (light.constant + light.lin * dist + light.quadratic * pow(dist, 2));

    // how far off-center is the fragment from the spotlight's direction?
    let spot_dir = normalize(light.direction.xyz);
    let theta = dot(-l, spot_dir); // cosine from the angle of the center

    let inner = light.cutoff;
    let outer = light.direction.w;
    let cone_factor = smoothstep(outer, inner, theta);

    let light_colour = light.color.rgb * attenuation * cone_factor;
    return blinn_phong(n, l, v, albedo, light_colour);
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

    let t = normalize(in.world_tangent);
    let b = normalize(in.world_bitangent);
    let n = normalize(in.world_normal);
    let tbn = mat3x3<f32>(t, b, n);

    return normalize(tbn * tangent_normal);
}

@fragment
fn s_fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.tex_coords * u_material.uv_tiling;
    let albedo_sample = textureSample(t_diffuse, s_diffuse, uv);

    // transparency
    if albedo_sample.a < u_material.alpha_cutoff {
        discard;
    }

    let albedo = albedo_sample.rgb * u_material.base_colour.rgb;
    let v = normalize(u_camera.view_pos.xyz - in.world_position);
    let n = get_normal(in, uv);

    var total_light = vec3<f32>(0.0);

    let ambient = albedo * u_globals.ambient_strength;
    total_light += ambient;

    for (var i = 0u; i < u_globals.num_lights; i++) {
        let light = s_light_array[i];
        let light_type = u32(light.color.w);

        if light_type == 0u {
            total_light += directional_light(light, n, v, albedo);
        } else if light_type == 1u {
            total_light += point_light(light, n, v, albedo, in.world_position);
        } else if light_type == 2u {
            total_light += spot_light(light, n, v, albedo, in.world_position);
        }
    }

    return vec4<f32>(total_light, albedo_sample.a);
}