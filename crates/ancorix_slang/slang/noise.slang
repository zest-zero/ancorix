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

/// The result of [`voronoi`]: how far `p` is from the nearest cell's seed
/// and the second-nearest, and which cell it fell in.
public struct Voronoi {
    /// Distance to the nearest cell's seed point.
    public float f1;
    /// Distance to the second-nearest cell's seed point.
    public float f2;
    /// The winning cell's own integer coordinate.
    public float2 id;

    public __init(float f1, float f2, float2 id) {
        this.f1 = f1;
        this.f2 = f2;
        this.id = id;
    }
}

/// Cellular (Worley) noise: scatters one random point per unit cell and
/// measures `p` against the nearest two.
public Voronoi voronoi(float2 p) {
    let cell = floor(p);
    let local = frac(p);

    var f1 = 1e9;
    var f2 = 1e9;
    var id = float2(0.0, 0.0);

    for (int y = -1; y <= 1; y++) {
        for (int x = -1; x <= 1; x++) {
            let neighbour = float2(float(x), float(y));
            let corner = cell + neighbour;
            let seed = neighbour + float2(hash(corner), hash(corner + float2(41.3, 289.1)));
            let d = length(seed - local);

            if (d < f1) {
                f2 = f1;
                f1 = d;
                id = corner;
            } else if (d < f2) {
                f2 = d;
            }
        }
    }

    return Voronoi(f1, f2, id);
}
