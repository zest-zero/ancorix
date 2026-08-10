import ancorix;

struct Vertex {
    [[vk::location(0)]] float2 pos   : POSITION;
    [[vk::location(1)]] float4 color : COLOR;
};

struct Varyings {
    float4 clip  : SV_Position;
    float4 color : COLOR;
};

[shader("vertex")]
Varyings vertexMain(Vertex v) {
    Varyings o;
    o.clip = to_clip(v.pos);
    o.color = v.color;
    return o;
}

[shader("fragment")]
float4 fragmentMain(Varyings v) : SV_Target {
    return v.color;
}
