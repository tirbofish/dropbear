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
    float4x4 view_proj;
};

[[vk::binding(0, 0)]]
ConstantBuffer<CameraUniform> camera;

[[vk::binding(1, 0)]]
ConstantBuffer<Light> light;

struct VertexInput {
    float3 position : POSITION;
    float4 model_matrix_0 : TEXCOORD5;
    float4 model_matrix_1 : TEXCOORD6;
    float4 model_matrix_2 : TEXCOORD7;
    float4 model_matrix_3 : TEXCOORD8;
};

struct VertexOutput {
    float4 clip_position : SV_Position;
    float3 color : COLOR0;
};


[shader("vertex")]
VertexOutput vs_main(VertexInput input) {
    float4x4 model_matrix = float4x4(
        input.model_matrix_0,
        input.model_matrix_1,
        input.model_matrix_2,
        input.model_matrix_3
    );

    VertexOutput output;
    output.clip_position = mul(camera.view_proj, mul(model_matrix, float4(input.position, 1.0)));
    output.color = light.color.xyz;
    return output;
}

[shader("fragment")]
float4 fs_main(VertexOutput input) : SV_Target0 {
    return float4(input.color, 1.0);
}