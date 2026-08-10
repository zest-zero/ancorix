// Hash-based noise. No textures and no tables - everything is computed from
// the coordinate, so it costs nothing to set up.

/// A repeatable pseudo-random value in 0..1 for each integer coordinate.
public float hash(float2 p) {
    let h = dot(p, float2(127.1, 311.7));
    return frac(sin(h) * 43758.5453123);
}

/// Smoothly interpolated `hash` - the classic value noise.
public float value_noise(float2 p) {
    let cell = floor(p);
    let f = frac(p);
    // smoothstep between cells, so the result has no visible grid lines
    let w = f * f * (3.0 - 2.0 * f);

    let a = hash(cell);
    let b = hash(cell + float2(1.0, 0.0));
    let c = hash(cell + float2(0.0, 1.0));
    let d = hash(cell + float2(1.0, 1.0));

    return lerp(lerp(a, b, w.x), lerp(c, d, w.x), w.y);
}

/// Several octaves of `value_noise` stacked, each half the amplitude and
/// twice the frequency - what most "clouds", "water" and "smoke" are made of.
public float fbm(float2 p, int octaves = 4) {
    var sum = 0.0;
    var amplitude = 0.5;
    var point = p;

    for (int i = 0; i < octaves; i++) {
        sum += value_noise(point) * amplitude;
        point *= 2.0;
        amplitude *= 0.5;
    }

    return sum;
}
