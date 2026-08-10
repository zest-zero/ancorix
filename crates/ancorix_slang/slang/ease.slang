// Curves that shape a 0..1 value: how an animation accelerates, how a
// gradient bends, how a highlight fades.

/// Radians from degrees, so an angle can be written the way it is thought:
/// `shape.rotate(deg(45.0))`.
public float deg(float degrees) {
    return degrees * 0.01745329252;
}

/// The cubic Bezier curve through (0,0), (x1,y1), (x2,y2) and (1,1) - the
/// same four numbers CSS takes.
///
/// One curve replaces a table of named easings: `bezier(t, 0.42, 0, 0.58, 1)`
/// is ease-in-out, `bezier(t, 0.25, 0.1, 0.25, 1)` is ease, and control
/// points above 1 overshoot.
///
/// Costs a handful of iterations, because the curve gives x and y from a
/// parameter and the answer wanted is y from x. Fine per frame or per shape;
/// think twice before putting it in a loop over pixels.
public float bezier(float t, float x1, float y1, float x2, float y2) {
    let clamped = saturate(t);

    // Newton's method on x(u) - t. Six steps is plenty for control points in
    // the usual range, and a fixed count keeps the shader branch-free.
    var u = clamped;
    for (int i = 0; i < 6; i++) {
        let slope = bezier_slope(u, x1, x2);
        if (abs(slope) < 0.00001) { break; }
        u -= (bezier_axis(u, x1, x2) - clamped) / slope;
    }

    return bezier_axis(saturate(u), y1, y2);
}

/// One axis of the curve at parameter `u`, with the ends pinned at 0 and 1.
public float bezier_axis(float u, float a, float b) {
    let v = 1.0 - u;
    return 3.0 * v * v * u * a + 3.0 * v * u * u * b + u * u * u;
}

/// That axis's slope, which Newton's method needs.
public float bezier_slope(float u, float a, float b) {
    let v = 1.0 - u;
    return 3.0 * v * v * a + 6.0 * v * u * (b - a) + 3.0 * u * u * (1.0 - b);
}

/// Bends `t` towards one end without solving anything: `k` above 1 pushes it
/// late, below 1 pushes it early.
///
/// The cheap shaping function, for when a curve is needed per pixel.
public float bias(float t, float k) {
    return pow(saturate(t), k);
}

/// Steepens or flattens the middle of `t` while keeping both ends: `k` above
/// 1 gives an S-curve, below 1 flattens it.
///
/// Also cheap - no iteration.
public float gain(float t, float k) {
    let x = saturate(t);
    return x < 0.5 ? pow(2.0 * x, k) * 0.5 : 1.0 - pow(2.0 - 2.0 * x, k) * 0.5;
}

/// A single smooth pulse: 0 at both ends, 1 in the middle.
public float pulse(float t) {
    return sin(saturate(t) * 3.14159265);
}

/// `t` bounced back and forth in 0..1 instead of wrapping, so an animation
/// driven by `s.time` reverses instead of jumping.
public float ping_pong(float t) {
    return 1.0 - abs(frac(t * 0.5) * 2.0 - 1.0);
}
