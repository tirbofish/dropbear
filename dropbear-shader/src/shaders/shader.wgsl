// Main shader for standard objects.

const MAX_LIGHTS: u32 = 8;

struct CameraUniform {
    view_pos: vec4<f32>,
    view_proj: mat4x4<f32>,
};

struct Light {
    position: vec4<f32>,
    direction: vec4<f32>, // x, y, z, outer_cutoff_angle
    color: vec4<f32>, // r, g, b, light_type (0, 1, 2)
    constant: f32,
    lin: f32,
    quadratic: f32,
    cutoff: f32,
    shadow_index: i32,
    proj: mat4x4<f32>,
}

struct LightArray {
    _lights: array<Light, MAX_LIGHTS>,
    light_count: u32,
    ambient_strength: f32,
}

struct MaterialUniform {
    // for stuff like tinting
    colour: vec4<f32>,

    // scales incoming UVs before sampling
    uv_tiling: vec2<f32>,
    _pad: vec2<f32>,
}

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;
@group(0) @binding(1)
var s_diffuse: sampler;
@group(0) @binding(2)
var t_normal: texture_2d<f32>;
@group(0) @binding(3)
var s_normal: sampler;

@group(3) @binding(0)
var<uniform> u_material: MaterialUniform;

@group(1) @binding(0)
var<uniform> camera: CameraUniform;

@group(2) @binding(0)
var<uniform> light_array: LightArray;
@group(2) @binding(1)
var t_shadow: texture_depth_2d_array;
@group(2) @binding(2)
var s_shadow: sampler_comparison;

struct InstanceInput {
    @location(5) model_matrix_0: vec4<f32>,
    @location(6) model_matrix_1: vec4<f32>,
    @location(7) model_matrix_2: vec4<f32>,
    @location(8) model_matrix_3: vec4<f32>,

    @location(9) normal_matrix_0: vec3<f32>,
    @location(10) normal_matrix_1: vec3<f32>,
    @location(11) normal_matrix_2: vec3<f32>,
};

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) tex_coords: vec2<f32>,
    @location(2) normal: vec3<f32>,
    @location(3) tangent: vec3<f32>,
    @location(4) bitangent: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) world_position: vec3<f32>,
    @location(3) world_tangent: vec3<f32>,
    @location(4) world_bitangent: vec3<f32>,
};

@vertex
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
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

    let world_normal = normalize(normal_matrix * model.normal);
    let world_tangent = normalize(normal_matrix * model.tangent);
    let world_bitangent = normalize(normal_matrix * model.bitangent);
    let world_position = model_matrix * vec4<f32>(model.position, 1.0);

    var out: VertexOutput;
    out.clip_position = camera.view_proj * world_position;
    out.tex_coords = model.tex_coords;
    out.world_normal = world_normal;
    out.world_position = world_position.xyz;
    out.world_tangent = world_tangent;
    out.world_bitangent = world_bitangent;
    return out;
}

fn calculate_shadow(light: Light, world_pos: vec3<f32>, normal: vec3<f32>, light_dir: vec3<f32>) -> f32 {
    // if -1, it means it doesnt cast shadows
    if (light.shadow_index < 0) {
        return 1.0;
    }

    let light_space_pos = light.proj * vec4<f32>(world_pos, 1.0);

    if (light_space_pos.w <= 0.0) {
        return 1.0;
    }
    let proj_correction = 1.0 / light_space_pos.w;

    let flip_correction = vec2<f32>(0.5, -0.5);
    let light_local = light_space_pos.xy * flip_correction * proj_correction + vec2<f32>(0.5, 0.5);

    let current_depth = light_space_pos.z * proj_correction;

    if (light_local.x < 0.0 || light_local.x > 1.0 || light_local.y < 0.0 || light_local.y > 1.0 || current_depth > 1.0) {
        return 1.0;
    }

    return textureSampleCompare(
        t_shadow,
        s_shadow,
        light_local,
        light.shadow_index,
        current_depth - 0.002
    );
}

fn directional_light(
    light: Light,
    world_normal: vec3<f32>,
    view_dir: vec3<f32>,
    tex_color: vec3<f32>,
    world_pos: vec3<f32>
) -> vec3<f32> {
    let light_dir = normalize(-light.direction.xyz);

    let ambient = light.color.xyz * light_array.ambient_strength * tex_color;

    let shadow = calculate_shadow(light, world_pos, world_normal, light_dir);

    let diff = max(dot(world_normal, light_dir), 0.0);
    let diffuse = light.color.xyz * diff * tex_color;

    let reflect_dir = reflect(-light_dir, world_normal);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    let specular = light.color.xyz * spec * tex_color;

    return ambient + (shadow * (diffuse + specular));
}

fn point_light(
    light: Light,
    world_pos: vec3<f32>,
    world_normal: vec3<f32>,
    view_dir: vec3<f32>,
    tex_color: vec3<f32>
) -> vec3<f32> {
    let norm = normalize(world_normal);
    let light_dir = normalize(light.position.xyz - world_pos);

    let shadow = calculate_shadow(light, world_pos, world_normal, light_dir);

    let diff = max(dot(norm, light_dir), 0.0);
    let diffuse = light.color.xyz * diff * tex_color;

    let shininess = 32.0;
    let reflect_dir = reflect(-light_dir, norm);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), shininess);
    let specular = light.color.xyz * spec * tex_color;

    let distance = length(light.position.xyz - world_pos);
    let attenuation = 1.0 / (light.constant + (light.lin * distance) + (light.quadratic * (distance * distance)));

    return (shadow * (diffuse + specular)) * attenuation;
}

fn spot_light(
    light: Light,
    world_pos: vec3<f32>,
    world_normal: vec3<f32>,
    view_dir: vec3<f32>,
    tex_color: vec3<f32>
) -> vec3<f32> {
    let outer_cutoff = light.direction.w;
    let ambient = light.color.xyz * light_array.ambient_strength * tex_color;

    let norm = normalize(world_normal);
    let light_dir = normalize(light.position.xyz - world_pos);

    let shadow = calculate_shadow(light, world_pos, world_normal, light_dir);

    let diff = max(dot(norm, light_dir), 0.0);
    var diffuse = light.color.xyz * diff * tex_color;

    let shininess = 32.0;
    let reflect_dir = reflect(-light_dir, norm);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), shininess);
    var specular = light.color.xyz * spec * tex_color;

    let theta = dot(light_dir, normalize(-light.direction.xyz));
    let epsilon = light.cutoff - outer_cutoff;
    let intensity = clamp((theta - outer_cutoff) / epsilon, 0.0, 1.0);

    diffuse *= intensity;
    specular *= intensity;

    let distance = length(light.position.xyz - world_pos);
    let attenuation = 1.0 / (light.constant + (light.lin * distance) + (light.quadratic * (distance * distance)));

    let ambient_attenuated = ambient * attenuation;
    let diffuse_attenuated = diffuse * attenuation;
    let specular_attenuated = specular * attenuation;

    return ambient_attenuated + (shadow * (diffuse_attenuated + specular_attenuated));
}

fn apply_normal_map(
    world_normal_in: vec3<f32>,
    world_tangent_in: vec3<f32>,
    world_bitangent_in: vec3<f32>,
    normal_sample_rgb: vec3<f32>,
) -> vec3<f32> {
    // Tangent-space normal in [-1, 1].
    let normal_ts = normalize(normal_sample_rgb * 2.0 - vec3<f32>(1.0));

    let n = normalize(world_normal_in);
    var t = normalize(world_tangent_in);
    t = normalize(t - n * dot(n, t));

    let b_in = normalize(world_bitangent_in);
    let handedness = select(-1.0, 1.0, dot(cross(n, t), b_in) >= 0.0);
    let b = cross(n, t) * handedness;

    let tbn = mat3x3<f32>(t, b, n);
    return normalize(tbn * normal_ts);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.tex_coords * u_material.uv_tiling;
    var tex_color = textureSample(t_diffuse, s_diffuse, uv);
    var object_normal = textureSample(t_normal, s_normal, uv);

    let base_colour = tex_color * u_material.colour;

    if (base_colour.a < 0.1) {
        discard;
    }

    let view_dir = normalize(camera.view_pos.xyz - in.world_position);
    let world_normal = apply_normal_map(
        in.world_normal,
        in.world_tangent,
        in.world_bitangent,
        object_normal.xyz,
    );

    var final_color = vec3<f32>(0.0);

    var total_ambient = vec3<f32>(0.0);
    for (var i: u32 = 0u; i < light_array.light_count; i = i + 1u) {
        let light = light_array._lights[i];
        total_ambient += light.color.xyz * light_array.ambient_strength;
    }

    for (var i: u32 = 0u; i < light_array.light_count; i = i + 1u) {
        let light = light_array._lights[i];

        // light type is color.w
        if light.color.w == 0.0 {
            // dir
            final_color += directional_light(light, world_normal, view_dir, base_colour.xyz, in.world_position);
        } else if light.color.w == 1.0 {
            // point
            final_color += point_light(light, in.world_position, world_normal, view_dir, base_colour.xyz);
        } else if light.color.w == 2.0 {
            // spot
            final_color += spot_light(light, in.world_position, world_normal, view_dir, base_colour.xyz);
        }
    }



//    final_color = (total_ambient * base_colour.xyz) + final_color;

    return vec4<f32>(final_color, base_colour.a);
}