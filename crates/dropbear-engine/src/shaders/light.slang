#include "dropbear.slang"

[[vk::binding(0, 0)]]
ConstantBuffer<CameraUniform> camera;

[[vk::binding(0, 1)]]
ConstantBuffer<Light> light;

struct VertexInput {
    [[vk::location(0)]]
    float3 position : POSITION;
    
    [[vk::location(5)]]
    float4 model_matrix_0 : TEXCOORD5;
    
    [[vk::location(6)]]
    float4 model_matrix_1 : TEXCOORD6;
    
    [[vk::location(7)]]
    float4 model_matrix_2 : TEXCOORD7;
    
    [[vk::location(8)]]
    float4 model_matrix_3 : TEXCOORD8;
};

struct VertexOutput {
    float4 clip_position : SV_Position;
    
    [[vk::location(0)]]
    float3 color : COLOR0;
};

[shader("vertex")]
VertexOutput vs_main(VertexInput input) {
    float4x4 model_matrix = vs_input_matrix(
        input.model_matrix_0,
        input.model_matrix_1,
        input.model_matrix_2,
        input.model_matrix_3,
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