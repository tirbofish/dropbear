struct Globals {
    uint num_lights;
    float ambient_strength;
};

/// A standard light descriptor struct. 
struct Light {
    float4 position;
    float4 direction; // x, y, z, outer_cutoff_angle
    float4 color; // r, g, b, light_type (0, 1, 2)

    float constant;
    float lin;
    float quadratic;

    float cutoff;
};

struct CameraUniform {
    float4 view_pos;
    float4x4 view;
    float4x4 view_proj;
    float4x4 inv_proj;
    float4x4 inv_view;
};

struct MaterialUniform {
    float4 base_colour;
    float3 emissive;
    float emissive_strength;
    float metallic;
    float roughness;
    float normal_scale;
    float occlusion_strength;
    float alpha_cutoff;
    float2 uv_tiling;
};

/// Converts a wgpu matrix (in the form of 4 float4) into a matrix usable for Slang. 
float4x4 vs_input_matrix(
    float4 mat1, 
    float4 mat2, 
    float4 mat3, 
    float4 mat4
) {
    return transpose(float4x4(
        mat1,
        mat2,
        mat3,
        mat4
    ));
}