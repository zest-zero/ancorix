// Gradients. Each returns a 0..1 ramp for a coordinate, so the colours stay
// the caller's business: `lerp(a, b, linear_gradient(s.uv, 0.0))`.

/// A ramp across the shape at `angle` radians: 0 at one edge, 1 at the other.
///
/// Takes `uv` (0..1), so it fits any size without scaling.
public float linear_gradient(float2 uv, float angle) {
    let dir = float2(cos(angle), sin(angle));
    // project onto the direction, then map the -0.5..0.5 range onto 0..1
    return saturate(dot(uv - 0.5, dir) + 0.5);
}

/// A ramp outward from `centre`, reaching 1 at `radius`.
///
/// In pixels, so it stays circular in a rectangle of any proportion - pass
/// `s.local` and a radius in pixels.
public float radial_gradient(float2 p, float2 centre, float radius) {
    return saturate(length(p - centre) / radius);
}

/// A ramp around `centre`, going once around from 0 back to 0 - the sweep a
/// cooldown timer or a pie chart draws.
public float angular_gradient(float2 p, float2 centre, float start) {
    const float TAU = 6.28318530718;
    let a = atan2(p.y - centre.y, p.x - centre.x) - start;
    return frac(a / TAU + 1.0);
}
