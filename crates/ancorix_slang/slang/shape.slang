// The shape system: one interface, and every operation for free on anything
// that implements it - including shapes you write yourself.

/// Coverage of a pixel by a shape whose signed distance is `dist`.
///
/// `fwidth` is how much `dist` changes between neighbouring pixels, so the
/// edge is smoothed over roughly one pixel whatever the shape's size. This is
/// the one place the derivative trick appears; no shader should need it.
public float aa(float dist) {
    let width = fwidth(dist) * 0.5;
    return 1.0 - smoothstep(-width, width, dist);
}

/// A 2D shape, described by how far any point is from its edge: negative
/// inside, zero on it, positive outside.
///
/// Implement this and every operation below works on your shape too.
public interface IShape {
    float distance(float2 p);
}

/// The edge of `inner`, `width` pixels thick.
public struct Outline<S : IShape> : IShape {
    public S inner;
    public float width;
    public float distance(float2 p) { return abs(inner.distance(p)) - width * 0.5; }
}

/// `inner` with `hole` cut out of it.
public struct Subtract<A : IShape, B : IShape> : IShape {
    public A inner;
    public B hole;
    public float distance(float2 p) { return max(inner.distance(p), -hole.distance(p)); }
}

/// Both shapes at once.
public struct Union<A : IShape, B : IShape> : IShape {
    public A first;
    public B second;
    public float distance(float2 p) { return min(first.distance(p), second.distance(p)); }
}

/// Only where both overlap.
public struct Intersect<A : IShape, B : IShape> : IShape {
    public A first;
    public B second;
    public float distance(float2 p) { return max(first.distance(p), second.distance(p)); }
}

/// `inner` with its edge pushed out by `amount` pixels - negative shrinks it.
public struct Grow<S : IShape> : IShape {
    public S inner;
    public float amount;
    public float distance(float2 p) { return inner.distance(p) - amount; }
}

/// `inner` moved by `offset` pixels.
public struct At<S : IShape> : IShape {
    public S inner;
    public float2 offset;
    public float distance(float2 p) { return inner.distance(p - offset); }
}

/// `inner` turned by `angle` radians about its own centre.
public struct Rotate<S : IShape> : IShape {
    public S inner;
    public float angle;
    public float distance(float2 p) {
        // the shape turns one way by asking the point to turn the other
        let c = cos(-angle);
        let s = sin(-angle);
        return inner.distance(float2(p.x * c - p.y * s, p.x * s + p.y * c));
    }
}

/// `inner` scaled about its own centre.
///
/// The distance is divided by the scale so that the result stays a true
/// distance - otherwise `coverage` would smooth the edge over the wrong
/// number of pixels.
public struct Scale<S : IShape> : IShape {
    public S inner;
    public float factor;
    public float distance(float2 p) { return inner.distance(p / factor) * factor; }
}

public extension<S : IShape> S {
    /// How much of the pixel at `p` this shape covers, 0 to 1, with the edge
    /// smoothed over roughly one pixel whatever the shape's size.
    public float coverage(float2 p) { return aa(distance(p)); }

    /// Like [`coverage`], but with the edge spread over `radius` pixels
    /// instead of one: 1 well inside, 0 well outside, a ramp between.
    ///
    /// A halo where it sits, and a drop shadow once moved:
    ///
    ///     shape.soften(s.local, 30.0)                 // glow
    ///     shape.at(float2(8.0, 8.0)).soften(p, 12.0)  // shadow
    public float soften(float2 p, float radius) {
        return 1.0 - smoothstep(-radius, radius, distance(p));
    }

    /// The shape with `hole` taken out of it.
    public Subtract<S, H> subtract<H : IShape>(H hole) { return Subtract<S, H>(this, hole); }

    /// The shape's edge as a shape of its own, `width` pixels thick.
    public Outline<S> outline(float width) { return Outline<S>(this, width); }

    /// This shape and `other` together.
    public Union<S, O> union<O : IShape>(O other) { return Union<S, O>(this, other); }

    /// Only where this shape and `other` overlap.
    public Intersect<S, O> intersect<O : IShape>(O other) { return Intersect<S, O>(this, other); }

    /// This shape with its edge pushed out by `amount` pixels.
    public Grow<S> grow(float amount) { return Grow<S>(this, amount); }

    /// This shape moved by `offset` pixels - the shader's `Transform2D`.
    public At<S> at(float2 offset) { return At<S>(this, offset); }

    /// This shape turned by `angle` radians about its own centre.
    public Rotate<S> rotate(float angle) { return Rotate<S>(this, angle); }

    /// This shape scaled about its own centre.
    public Scale<S> scale(float factor) { return Scale<S>(this, factor); }
}
