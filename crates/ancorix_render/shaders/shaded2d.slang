import ancorix;

// The vertex stage the engine pairs with a user's fragment shader, so that
// writing one means writing `float4 pixel(Surface s)` and nothing else.
struct Vertex {
    [[vk::location(0)]] float2 pos   : POSITION;
    [[vk::location(1)]] float4 color : COLOR;
    [[vk::location(2)]] float2 uv    : UV;
    [[vk::location(3)]] float2 size  : SIZE;
};

[shader("vertex")]
Surface vertexMain(Vertex v) {
    Surface s;
    s.clip = to_clip(v.pos);
    s.color = v.color;
    s.uv = v.uv;
    s.size = v.size;
    s.time = globals.time;
    return s;
}
