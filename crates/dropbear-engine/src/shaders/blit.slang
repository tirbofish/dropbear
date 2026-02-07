#include "dropbear.slang"

struct VertexOutput {
    float4 clip_position : SV_Position;

    [[vk_location(0)]]
    float2 uv;
};

[[vk_binding(0, 0)]]
Texture2D tex;

[[vk_binding(1, 0)]]
SamplerState tex_sampler;

[shader("vertex")]
VertexOutput vs_main(
    uint in_vertex_index: SV_VulkanVertexID
) {
    VertexOutput output;

    float x = float((in_vertex_index << 1u) & 2u);
    float y = float(in_vertex_index & 2u);

    output.clip_position = float4(x * 2.0 - 1.0, y * 2.0 - 1.0, 0.0, 1.0);
    output.uv = float2(x, 1.0 - y);

    return output;
}

[shader("fragment")]
float4 fs_main(VertexOutput input) : SV_Target0 {
    float4 s = tex.Sample(tex_sampler, input.uv);
    return s;
}