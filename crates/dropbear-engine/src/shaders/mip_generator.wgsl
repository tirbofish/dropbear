/// Generates mip level N from level N-1 of a cubemap stored as a D2Array texture.
/// The src binding is a regular texture view of exactly mip level N-1 (base_mip_level = N-1,
/// mip_level_count = 1).  Using textureLoad (not a ReadOnly storage texture) gives
/// reliable cross-backend support since ReadOnly storage images are poorly supported.
/// The dst binding is a WriteOnly storage view of mip level N.
/// All 6 cube faces (array_layer_count = 6) are processed in the Z dimension.

@group(0) @binding(0) var src: texture_2d_array<f32>;
@group(0) @binding(1) var dst: texture_storage_2d_array<rgba16float, write>;

@compute @workgroup_size(8, 8, 1)
fn generate_mip(@builtin(global_invocation_id) id: vec3<u32>) {
    let dst_size = textureDimensions(dst);
    if id.x >= dst_size.x || id.y >= dst_size.y || id.z >= 6u {
        return;
    }

    let src_x = i32(id.x) * 2;
    let src_y = i32(id.y) * 2;
    let layer = i32(id.z);

    // Box filter: average 2×2 block.
    // LOD 0 refers to the only mip level visible in the bound view (= the actual N-1 level).
    let c00 = textureLoad(src, vec2<i32>(src_x,     src_y    ), layer, 0);
    let c10 = textureLoad(src, vec2<i32>(src_x + 1, src_y    ), layer, 0);
    let c01 = textureLoad(src, vec2<i32>(src_x,     src_y + 1), layer, 0);
    let c11 = textureLoad(src, vec2<i32>(src_x + 1, src_y + 1), layer, 0);

    textureStore(dst, vec2<i32>(i32(id.x), i32(id.y)), layer, (c00 + c10 + c01 + c11) * 0.25);
}
