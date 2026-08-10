// Colours: building them, fading them, and the sRGB conversions that are easy
// to get silently wrong.

/// A colour from a hex literal, the way CSS writes one: `rgb(0x22d3ee)`.
public float3 rgb(uint hex) {
    return float3((hex >> 16) & 0xFF, (hex >> 8) & 0xFF, hex & 0xFF) / 255.0;
}

/// The same names `ancorix_color::Rgba` uses, so a colour reads the same on
/// both sides: `Rgba::CYAN` in Rust, `Color.cyan` in a shader.
///
/// Grouped in a struct rather than loose constants, so that a shader of your
/// own can have a `red` without colliding with this one.
public struct Color {
    public static const float4 red     = float4(1.0, 0.0, 0.0, 1.0);
    public static const float4 green   = float4(0.0, 1.0, 0.0, 1.0);
    public static const float4 blue    = float4(0.0, 0.0, 1.0, 1.0);
    public static const float4 white   = float4(1.0, 1.0, 1.0, 1.0);
    public static const float4 black   = float4(0.0, 0.0, 0.0, 1.0);
    public static const float4 yellow  = float4(1.0, 1.0, 0.0, 1.0);
    public static const float4 cyan    = float4(0.0, 1.0, 1.0, 1.0);
    public static const float4 magenta = float4(1.0, 0.0, 1.0, 1.0);
    public static const float4 purple  = float4(0.502, 0.0, 0.502, 1.0);
    public static const float4 pink    = float4(1.0, 0.753, 0.796, 1.0);
    public static const float4 orange  = float4(1.0, 0.647, 0.0, 1.0);
    public static const float4 grey    = float4(0.502, 0.502, 0.502, 1.0);
    /// Nothing at all - the usual "leave this pixel alone".
    public static const float4 clear   = float4(0.0, 0.0, 0.0, 0.0);
};

/// A colour from a hex literal with alpha: `rgba(0x22d3ee80)`.
public float4 rgba(uint hex) {
    return float4(rgb(hex >> 8), float(hex & 0xFF) / 255.0);
}

/// Straight alpha "over" compositing: `top` drawn on top of `bottom`.
public float4 over(float4 top, float4 bottom) {
    let alpha = top.a + bottom.a * (1.0 - top.a);
    if (alpha <= 0.0) { return float4(0.0, 0.0, 0.0, 0.0); }

    let rgb = (top.rgb * top.a + bottom.rgb * bottom.a * (1.0 - top.a)) / alpha;
    return float4(rgb, alpha);
}

public extension float4 {
    /// This colour with its alpha scaled - how a shape's coverage becomes
    /// transparency.
    public float4 fade(float alpha) { return float4(this.rgb, this.a * alpha); }

    /// Moved `amount` of the way towards white, alpha untouched.
    public float4 lighten(float amount) {
        return float4(lerp(this.rgb, float3(1.0), saturate(amount)), this.a);
    }

    /// Moved `amount` of the way towards black, alpha untouched.
    public float4 darken(float amount) {
        return float4(lerp(this.rgb, float3(0.0), saturate(amount)), this.a);
    }

    /// Perceived brightness, for deciding whether to put dark or light text
    /// on top of this colour.
    public float luminance() {
        return dot(this.rgb, float3(0.2126, 0.7152, 0.0722));
    }
};

/// sRGB to linear, per channel.
///
/// Ancorix hands shaders colours exactly as they were authored, and blends in
/// that space. Convert only when a calculation genuinely needs light-linear
/// values (physical blur, additive lighting), and convert back afterwards.
public float3 to_linear(float3 srgb) {
    return select(srgb <= 0.04045, srgb / 12.92, pow((srgb + 0.055) / 1.055, 2.4));
}

/// Linear back to sRGB, per channel.
public float3 to_srgb(float3 linear) {
    return select(linear <= 0.0031308, linear * 12.92, 1.055 * pow(linear, 1.0 / 2.4) - 0.055);
}
