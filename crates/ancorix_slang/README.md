# ancorix_slang

Ancorix's shader library for [Slang](https://shader-slang.org). A shader is
one function; everything it needs is behind one import.

```slang
import ancorix;

float4 pixel(Surface s) {
    let ring = Circle(s.size.x * 0.4).subtract(Circle(s.size.x * 0.2));
    return s.color.fade(ring.coverage(s.local));
}
```

```rust
let donut = ctx.assets.shader(include_bytes!("../assets/shaders/donut.frag.spv"));
ctx.draw.shader(donut, Rect::from_center(center, v2!(320.0)), Rgba::CYAN);
```

No vertex stage, no varyings, no push constants, no `fwidth`. The rect is the
canvas the shader paints on; what it paints is up to the shader.

## What a pixel knows

`Surface` is the only input:

| field | meaning |
|---|---|
| `s.uv` | 0..1 across the shape, from its top-left corner |
| `s.local` | pixels from the shape's centre |
| `s.size` | the shape's size in pixels |
| `s.color` | the colour passed to `draw.shader(...)` |
| `s.time` | seconds since the window opened |
| `s.screen` | this pixel's position on screen, in pixels |

Everything is in pixels or in 0..1 - never in clip space, and never scaled by
the window's aspect, so a circle stays a circle.

## Shapes

A shape answers one question: how far is this point from my edge? Negative
inside, zero on it, positive outside.

```slang
Circle(radius)
Box(size)                 // size in pixels
Box(size, corner)         // rounded corners
Segment(from, to, width)
```

Every shape gets these for free, including one you write yourself:

```slang
shape.coverage(p)         // 0..1, antialiased - the usual way to finish
shape.glow(p, radius)     // soft halo reaching `radius` pixels out
shape.outline(width)      // the edge, as a shape
shape.grow(amount)        // edge pushed out (negative shrinks)
shape.subtract(hole)      // this, with `hole` cut out
shape.union(other)        // both
shape.intersect(other)    // only the overlap
shape.at(offset)          // moved, in pixels
shape.rotate(angle)       // turned about its own centre
shape.scale(factor)       // scaled about its own centre
```

Transforms are separate operations rather than one `Transform2D`-shaped
struct, because a shader pays for them per pixel: `.at(v)` costs a subtract,
while a combined transform would compute a sine for a shape that only moved.
Composing them also makes the order explicit, and gives a pivot for free:

```slang
shape.at(-pivot).rotate(angle).at(pivot)
```

Your own shape needs one function:

```slang
struct Star : IShape {
    float radius;
    float distance(float2 p) { /* ... */ }
}
```

and `Star(40).outline(3).coverage(s.local)` works immediately.

## Colour

```slang
Color.cyan                // the names `Rgba` uses in Rust
Color.clear               // nothing at all
rgb(0x22d3ee)             // as CSS writes it
rgba(0x22d3ee80)          // with alpha
color.fade(a)             // scale alpha - how coverage becomes transparency
color.lighten(t)          // towards white
color.darken(t)           // towards black
color.luminance()         // for choosing dark or light text on top
over(top, bottom)         // straight-alpha compositing
to_linear(c) / to_srgb(c) // only when a calculation needs light-linear values
```

## Noise and easing

```slang
hash(p)                   // repeatable 0..1 per coordinate
value_noise(p)            // smooth
fbm(p)                    // several octaves - clouds, water, smoke
```

```slang
ease_in_quad / ease_out_quad / ease_in_out_quad
ease_in_cubic / ease_out_cubic / ease_in_out_cubic
ease_out_back             // overshoots, then settles
pulse(t)                  // 0 -> 1 -> 0
ping_pong(t)              // bounces instead of wrapping, for `s.time`
```

## Recipes

**A ring.** Cut a smaller circle out of a bigger one:

```slang
let ring = Circle(s.size.x * 0.4).subtract(Circle(s.size.x * 0.2));
return s.color.fade(ring.coverage(s.local));
```

**A badge with an outline.** Draw the body, then the edge on top:

```slang
let body = Circle(s.size.x * 0.35);
return over(rgb(0xf59e0b).fade(body.outline(4.0).coverage(s.local)),
            s.color.fade(body.coverage(s.local)));
```

**A pulsing highlight.** `s.time` drives it, `ping_pong` keeps it from
jumping:

```slang
let beat = ping_pong(s.time * 0.5);
return s.color.lighten(beat * 0.4).fade(Circle(s.size.x * 0.5).coverage(s.local));
```

**A vertical gradient:**

```slang
return float4(lerp(rgb(0x22d3ee), rgb(0xa855f7), s.uv.y), 1.0);
```

**Clouds:**

```slang
let n = fbm(s.uv * 4.0 + s.time * 0.05);
return float4(lerp(rgb(0x1e293b), rgb(0xe2e8f0), n), 1.0);
```

## Building a shader

Slang has no package manager, so the compiler has to be told where these
modules are. `include_path()` is that directory:

```rust
// build.rs
Command::new("slangc")
    .arg("-target").arg("spirv")
    .arg("-I").arg(ancorix_slang::include_path())
    .arg("-entry").arg("pixel").arg("-stage").arg("fragment")
    .arg("-o").arg(out.join("donut.frag.spv"))
    .arg("shaders/donut.slang")
    .status()?;
```

## Tests

`slangc` compiles the same modules to a host executable, so the maths is
tested on the CPU, without a GPU or a screenshot:

```
cargo test -p ancorix_slang
```

Anything that depends on screen derivatives (`aa`, `coverage`) can't be
checked that way - a pixel has no neighbours on the CPU.

## Modules

`import ancorix;` re-exports all of them; import one directly if you want
only part.

| module | holds |
|---|---|
| `surface` | `Surface`, `Globals`, `to_clip` |
| `shape` | `IShape`, `aa`, and the operations every shape gets |
| `shapes` | `Circle`, `Box`, `Segment`, and their distance functions |
| `color` | `rgb`, `over`, fades, sRGB conversions |
| `ease` | easing curves |
| `noise` | `hash`, `value_noise`, `fbm` |
