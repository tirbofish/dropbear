struct MaterialUniform {
    base_colour:       vec4<f32>,
    emissive:          vec3<f32>,
    emissive_strength: f32,
    metallic:          f32,
    roughness:         f32,
    normal_scale:      f32,
    occlusion_strength: f32,
    alpha_cutoff:      f32,
    uv_tiling:         vec2<f32>,

    has_normal_texture:   u32,
    has_emissive_texture: u32,
    has_metallic_texture: u32,
    has_occlusion_texture: u32,
}

@group(1) @binding(0)  var<uniform>  u_material:  MaterialUniform;
@group(1) @binding(1)  var           t_diffuse:   texture_2d<f32>;
@group(1) @binding(2)  var           s_diffuse:   sampler;
@group(1) @binding(3)  var           t_normal:    texture_2d<f32>;
@group(1) @binding(4)  var           s_normal:    sampler;
@group(1) @binding(5)  var           t_emissive:  texture_2d<f32>;
@group(1) @binding(6)  var           s_emissive:  sampler;
@group(1) @binding(7)  var           t_metallic:  texture_2d<f32>;
@group(1) @binding(8)  var           s_metallic:  sampler;
@group(1) @binding(9)  var           t_occlusion: texture_2d<f32>;
@group(1) @binding(10) var           s_occlusion: sampler;