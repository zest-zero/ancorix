// What a pixel knows about the shape it is being drawn in, and the handful of
// values the engine pushes for every draw.

/// Push constants every Ancorix pipeline agrees on.
public struct Globals {
    public float2 screen_size;
    public float  time;
};

[[vk::push_constant]]
public ConstantBuffer<Globals> globals;

/// Pixel space (origin top-left, y down) maps straight onto Vulkan's clip
/// space (also y down) - no flip, unlike OpenGL.
public float4 to_clip(float2 pixel) {
    return float4((pixel / globals.screen_size) * 2.0 - 1.0, 0.0, 1.0);
}

/// Everything a pixel knows about the shape it is being drawn in.
///
/// The engine's vertex stage fills this, so a user shader only ever writes
/// the `pixel` function that receives it.
public struct Surface {
    public float4 clip  : SV_Position;
    public float4 color : COLOR;
    /// 0..1 across the shape, from its top-left corner.
    public float2 uv    : UV;
    /// The shape's size in pixels.
    public float2 size  : SIZE;
    /// Seconds since the window opened.
    public float  time  : TIME;
};

public extension Surface {
    /// Pixels from the shape's centre. Derived rather than interpolated - it
    /// follows from `uv` and `size`, so it costs no varying.
    public property float2 local {
        get { return (uv - 0.5) * size; }
    }

    /// Where this pixel is on screen, in pixels.
    public property float2 screen {
        get { return clip.xy; }
    }
};

/// The push constants a shader with parameters of its own declares.
///
/// Vulkan gives a shader stage one push constant block, so a shader cannot
/// declare its parameters beside the engine's - they would both land at
/// offset zero and overlap. Wrapping them puts the engine's bytes first,
/// where the engine writes them, and the shader's own after:
///
///     struct Params { float progress; float4 fill; };
///     [[vk::push_constant]] uniform Uniforms<Params> pc;
///
///     float4 pixel(Surface s) { ... pc.params.progress ... }
///
/// A shader with no parameters declares nothing and reads `s.time`.
public struct Uniforms<T> {
    /// What the engine writes. Present to reserve those bytes; a fragment
    /// shader reads the same values off [`Surface`] instead.
    public Globals globals;
    public T params;
};
