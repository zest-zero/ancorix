// Repeating fills. Each takes a coordinate and returns 0..1, so it drops
// straight into `lerp` between two colours or into a shape's alpha.

/// A checkerboard of `size`-pixel squares: 0 or 1, no blending.
public float checker(float2 p, float size) {
    let cell = floor(p / size);
    return abs(fmod(cell.x + cell.y, 2.0));
}

/// Stripes running along y, `size` pixels from one edge to the next.
///
/// `softness` in pixels smooths the transition; 0 keeps it hard.
public float stripes(float2 p, float size, float softness = 0.0) {
    let phase = abs(fmod(p.x, size * 2.0) - size);
    if (softness <= 0.0) { return step(size * 0.5, phase); }

    return smoothstep(size * 0.5 - softness, size * 0.5 + softness, phase);
}

/// Grid lines `thickness` pixels wide, every `size` pixels: 1 on a line.
public float grid(float2 p, float size, float thickness) {
    // distance to the nearest line in each axis, antialiased with `fwidth`
    // so thin lines don't shimmer
    let d = abs(fmod(p, size) - size * 0.5);
    let edge = size * 0.5 - thickness * 0.5;
    let w = fwidth(p);

    let x = 1.0 - smoothstep(edge - w.x, edge + w.x, d.x);
    let y = 1.0 - smoothstep(edge - w.y, edge + w.y, d.y);
    return 1.0 - x * y;
}

/// Dots of `radius` pixels on a `size`-pixel grid.
public float dots(float2 p, float size, float radius) {
    let centre = abs(fmod(p, size) - size * 0.5);
    let d = length(centre) - radius;
    let w = fwidth(d) * 0.5;

    return 1.0 - smoothstep(-w, w, d);
}
