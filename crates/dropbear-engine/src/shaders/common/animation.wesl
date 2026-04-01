struct MorphTargetInfo {
    num_vertices:  u32,
    num_targets:   u32,
    base_offset:   u32,
    weight_offset: u32,
    uses_morph:    u32,
}

@group(2) @binding(0) var<storage, read> s_skinning:     array<mat4x4<f32>>;
@group(2) @binding(1) var<storage, read> s_morph_deltas: array<f32>;
@group(2) @binding(2) var<storage, read> s_morph_weights: array<f32>;
@group(2) @binding(3) var<uniform>       u_morph_info:   MorphTargetInfo;

fn apply_morph(base_pos: vec3<f32>, vertex_id: u32) -> vec3<f32> {
    var result = base_pos;
    for (var t = 0u; t < u_morph_info.num_targets; t++) {
        let weight = s_morph_weights[u_morph_info.weight_offset + t];
        let idx    = u_morph_info.base_offset + (t * u_morph_info.num_vertices + vertex_id) * 3u;
        let delta  = vec3<f32>(
            s_morph_deltas[idx],
            s_morph_deltas[idx + 1u],
            s_morph_deltas[idx + 2u],
        );
        result += delta * weight;
    }
    return result;
}

struct AnimatedVertex {
    position: vec3<f32>,
    normal:   vec3<f32>,
    tangent:  vec3<f32>,
}

fn apply_animation(
    position:  vec3<f32>,
    normal:    vec3<f32>,
    tangent:   vec3<f32>,
    joints:    vec4<u32>,
    weights:   vec4<f32>,
    vertex_id: u32,
) -> AnimatedVertex {
    // Morph targets are applied first (in bind-pose space, per glTF spec)
    let morphed_pos = apply_morph(position, vertex_id);

    let skin_matrix =
        weights.x * s_skinning[joints.x] +
        weights.y * s_skinning[joints.y] +
        weights.z * s_skinning[joints.z] +
        weights.w * s_skinning[joints.w];

    let skinned_pos     = (skin_matrix * vec4<f32>(morphed_pos, 1.0)).xyz;
    let skinned_normal  = normalize((skin_matrix * vec4<f32>(normal,  0.0)).xyz);
    let skinned_tangent = normalize((skin_matrix * vec4<f32>(tangent, 0.0)).xyz);

    return AnimatedVertex(skinned_pos, skinned_normal, skinned_tangent);
}