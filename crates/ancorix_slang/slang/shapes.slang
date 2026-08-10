// The built-in shapes. Distance functions follow
// iquilezles.org/articles/distfunctions2d.

import shape;

/// Exact distance to a circle centred on the origin.
public float sd_circle(float2 p, float radius) {
    return length(p) - radius;
}

/// Exact distance to a box with rounded corners, centred on the origin.
public float sd_rounded_box(float2 p, float2 half_size, float radius) {
    float2 q = abs(p) - half_size + radius;
    return min(max(q.x, q.y), 0.0) + length(max(q, 0.0)) - radius;
}

/// Exact distance to a line segment `width` pixels thick.
public float sd_segment(float2 p, float2 a, float2 b, float width) {
    let pa = p - a;
    let ba = b - a;
    let h = saturate(dot(pa, ba) / dot(ba, ba));
    return length(pa - ba * h) - width * 0.5;
}

/// A circle centred on the shape's own centre.
public struct Circle : IShape {
    public float radius;
    public __init(float radius) { this.radius = radius; }
    public float distance(float2 p) { return sd_circle(p, radius); }
}

/// A box, optionally with rounded corners, sized in pixels.
public struct Box : IShape {
    public float2 half_size;
    public float corner;
    public __init(float2 size, float corner = 0.0) {
        this.half_size = size * 0.5;
        this.corner = corner;
    }
    public float distance(float2 p) { return sd_rounded_box(p, half_size, corner); }
}

// The mask `chamfer` lays over a shape: a square standing on its corner, so
// that what they have in common is the shape with its corners taken off.
// Public only because the language needs the return type of a public method
// to be public - not part of the vocabulary.
public struct ChamferMask : IShape {
    public float reach;
    public __init(float2 size, float amount) {
        // the diamond's half-diagonal: pulling it in by `amount` on each axis
        // is what takes the corners off
        this.reach = (size.x + size.y) * 0.5 - amount;
    }
    public float distance(float2 p) {
        // |x| + |y| is a diamond's distance field, up to the constant that
        // keeps the result a true distance
        return (abs(p.x) + abs(p.y) - reach) * 0.70710678;
    }
}

/// A line segment between two points, `width` pixels thick.
public struct Segment : IShape {
    public float2 from;
    public float2 to;
    public float width;
    public __init(float2 from, float2 to, float width) {
        this.from = from;
        this.to = to;
        this.width = width;
    }
    public float distance(float2 p) { return sd_segment(p, from, to, width); }
}

// Lives here rather than in `shape.slang` because it needs `Diamond`, and a
// module cannot import the one that imports it.
public extension<S : IShape> S {
    /// The shape with its corners cut off at 45 degrees, `amount` pixels in
    /// from each one.
    ///
    /// Keeps only what a diamond covers, so a square becomes an octagon.
    public Intersect<S, ChamferMask> chamfer(float2 size, float amount) {
        return Intersect<S, ChamferMask>(this, ChamferMask(size, amount));
    }
}
