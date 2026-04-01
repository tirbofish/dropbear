import super::super::shader::u_globals;

@group(0) @binding(2) var<storage, read> s_light_array: array<Light>;

struct Light {
    position:  vec4<f32>,
    direction: vec4<f32>, // x, y, z, outer_cutoff_angle
    color:     vec4<f32>, // r, g, b, light_type (0=directional, 1=point, 2=spot)
    constant:  f32,
    lin:       f32,
    quadratic: f32,
    cutoff:    f32,   // inner cutoff (cos of angle)
}

const LIGHT_DIRECTIONAL: u32 = 0u;
const LIGHT_POINT: u32       = 1u;
const LIGHT_SPOT: u32        = 2u;

fn diffuse(
    light_dir:    vec3<f32>,
    normal:       vec3<f32>,
    light_color:  vec3<f32>,
    object_color: vec3<f32>,
) -> vec3<f32> {
    let diff = max(dot(normal, light_dir), 0.0);
    return diff * light_color * object_color;
}

// Blinn-Phong specular
fn specular(
    light_dir:   vec3<f32>,
    normal:      vec3<f32>,
    view_dir:    vec3<f32>,
    light_color: vec3<f32>,
    shininess:   f32,
) -> vec3<f32> {
    let halfway  = normalize(light_dir + view_dir);
    let spec     = pow(max(dot(normal, halfway), 0.0), shininess);
    return light_color * spec;
}

fn attenuation(light: Light, world_pos: vec3<f32>) -> f32 {
    let d = length(light.position.xyz - world_pos);
    return 1.0 / (light.constant + light.lin * d + light.quadratic * d * d);
}

fn calculate_light(
    light:        Light,
    world_pos:    vec3<f32>,
    normal:       vec3<f32>,   // normalised world-space surface normal
    view_dir:     vec3<f32>,   // normalised direction from fragment to camera
    object_color: vec3<f32>,
    shininess:    f32,
    spec_scale:   f32,
) -> vec3<f32> {
    let light_type  = u32(light.color.w);
    let light_color = light.color.xyz;

    var light_dir: vec3<f32>;
    var atten:     f32 = 1.0;
    var intensity: f32 = 1.0;

    if light_type == LIGHT_DIRECTIONAL {
        light_dir = normalize(-light.direction.xyz);
    } else {
        // point and spot included
        light_dir = normalize(light.position.xyz - world_pos);
        atten     = attenuation(light, world_pos);

        if light_type == LIGHT_SPOT {
            // smooth edge
            let theta        = dot(light_dir, normalize(-light.direction.xyz));
            let inner_cutoff = light.cutoff;
            let outer_cutoff = light.direction.w;
            let epsilon      = inner_cutoff - outer_cutoff;
            intensity        = clamp((theta - outer_cutoff) / epsilon, 0.0, 1.0);
        }
    }

    let d = diffuse(light_dir, normal, light_color, object_color) * intensity;
    let s = specular(light_dir, normal, view_dir, light_color, shininess) * intensity * spec_scale;

    return (d + s) * atten;
}


fn calculate_lighting(
    world_pos:    vec3<f32>,
    normal:       vec3<f32>,
    view_dir:     vec3<f32>,
    object_color: vec3<f32>,
    shininess:    f32,
    spec_scale:   f32,
) -> vec3<f32> {
    var result = u_globals.ambient_strength * object_color;
    let num_lights = u_globals.num_lights;
    for (var i = 0u; i < num_lights; i++) {
        result += calculate_light(
            s_light_array[i],
            world_pos,
            normal,
            view_dir,
            object_color,
            shininess,
            spec_scale,
        );
    }
    return result;
}