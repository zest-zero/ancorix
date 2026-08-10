import ancorix;

struct Vertex {
    [[vk::location(0)]] float2 pos    : POSITION;
    [[vk::location(1)]] float4 color  : COLOR;
    [[vk::location(2)]] float2 local  : LOCAL;
    [[vk::location(3)]] float  radius : RADIUS;
};

struct Varyings {
    float4 clip   : SV_Position;
    float4 color  : COLOR;
    float2 local  : LOCAL;
    float  radius : RADIUS;
};

[shader("vertex")]
Varyings vertexMain(Vertex v) {
    Varyings o;
    o.clip = to_clip(v.pos);
    o.color = v.color;
    o.local = v.local;
    o.radius = v.radius;
    return o;
}

[shader("fragment")]
float4 fragmentMain(Varyings v) : SV_Target {
    float alpha = aa(sd_circle(v.local, v.radius));
    return float4(v.color.rgb, v.color.a * alpha);
}
