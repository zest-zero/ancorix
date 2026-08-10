import ancorix;

// One instance per shape: the six corners of the quad come from the vertex
// index, so nothing about the shape is stored more than once.
struct Instance {
    [[vk::location(0)]] float2 pos      : POSITION;
    [[vk::location(1)]] float2 size     : SIZE;
    [[vk::location(2)]] float2 origin   : ORIGIN;
    [[vk::location(3)]] float2 scale    : SCALE;
    [[vk::location(4)]] float  radius   : RADIUS;
    [[vk::location(5)]] float  rotation : ROTATION;
    [[vk::location(6)]] float4 color    : COLOR;
};

struct Varyings {
    float4 clip      : SV_Position;
    float4 color     : COLOR;
    float2 local     : LOCAL;
    float2 half_size : HALF_SIZE;
    float  radius    : RADIUS;
};

static const float2 CORNERS[6] = {
    float2(0.0, 0.0), float2(1.0, 0.0), float2(1.0, 1.0),
    float2(0.0, 0.0), float2(1.0, 1.0), float2(0.0, 1.0),
};

[shader("vertex")]
Varyings vertexMain(Instance inst, uint vertex_id : SV_VertexID) {
    let half_size = inst.size * 0.5;
    let local = (CORNERS[vertex_id] - 0.5) * inst.size;
    let world = inst.pos + half_size + local;

    // the same pivot maths `Transform2D::apply` runs on the CPU for the
    // other pipelines - identity leaves `world` untouched
    let pivot = inst.pos + inst.origin * inst.size;
    let rel = (world - pivot) * inst.scale;
    let s = sin(inst.rotation);
    let c = cos(inst.rotation);
    let placed = pivot + float2(rel.x * c - rel.y * s, rel.x * s + rel.y * c);

    Varyings o;
    o.clip = to_clip(placed);
    o.color = inst.color;
    // untransformed: the SDF is evaluated in the shape's own space, and an
    // affine transform commutes with the interpolation
    o.local = local;
    o.half_size = half_size;
    o.radius = inst.radius;
    return o;
}

[shader("fragment")]
float4 fragmentMain(Varyings v) : SV_Target {
    float alpha = aa(sd_rounded_box(v.local, v.half_size, v.radius));
    return float4(v.color.rgb, v.color.a * alpha);
}
