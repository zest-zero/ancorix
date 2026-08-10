import ancorix;

[[vk::binding(0, 0)]]
Sampler2D tex;

struct Vertex {
    [[vk::location(0)]] float2 pos   : POSITION;
    [[vk::location(1)]] float2 uv    : UV;
    [[vk::location(2)]] float4 color : COLOR;
};

struct Varyings {
    float4 clip  : SV_Position;
    float2 uv    : UV;
    float4 color : COLOR;
};

[shader("vertex")]
Varyings vertexMain(Vertex v) {
    Varyings o;
    o.clip = to_clip(v.pos);
    o.uv = v.uv;
    o.color = v.color;
    return o;
}

// `color` tints the sampled texel rather than replacing it
[shader("fragment")]
float4 fragmentMain(Varyings v) : SV_Target {
    return tex.Sample(v.uv) * v.color;
}
