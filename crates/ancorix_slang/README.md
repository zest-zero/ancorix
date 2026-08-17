# ancorix_slang

Ancorix's shader library for [Slang](https://shader-slang.org). A shader is
one function; everything it needs is behind one import.

```slang
import ancorix;

float4 pixel(Surface s) {
    let ring = Circle(s.size.x * 0.4).subtract(Circle(s.size.x * 0.2));
    return ring.paint(s, s.color);
}
```

```rust
let donut = ctx.assets.shader(include_bytes!("../assets/shaders/donut.frag.spv"));
ctx.draw.shader(donut, Rect::from_center(center, v2!(320.0)), Rgba::CYAN);
```

The engine's vertex stage fills `Surface` and hands it to `pixel` - that
function is the only thing a shader writes.

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

Every shape gets these, including one you write yourself:

```slang
shape.coverage(p)         // 0..1, antialiased - the usual way to finish
shape.paint(s, color)     // color faded by coverage - what most shaders end with
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
struct: a shader pays for them per pixel, so `.at(v)` costs a subtract while
a combined transform would compute a sine for a shape that only moved.
Composing them makes the order explicit and gives a pivot for free:

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

The names are the same ones `Rgba` uses in Rust, so a colour reads the same
on both sides:

```slang
Color.cyan
Color.purple
Color.orange
Color.white
Color.black
Color.grey
Color.clear                // nothing at all
```

For a colour that isn't in that list, `rgb()` takes a hex literal the way
CSS writes one - `rgb(0x22d3ee)` is that same cyan:

```slang
rgb(hex)                   // e.g. rgb(0x22d3ee) for #22d3ee
rgba(hex)                  // same, with alpha - e.g. rgba(0x22d3ee80)
```

For plain r/g/b components, skip both and write `float3(r, g, b)` - that's
native Slang, no function needed.

```slang
color.fade(a)              // scale alpha - how coverage becomes transparency
color.lighten(t)           // towards white
color.darken(t)            // towards black
color.luminance()          // for choosing dark or light text on top
over(top, bottom)          // straight-alpha compositing
to_linear(c) / to_srgb(c)  // only when a calculation needs light-linear values
```

## Noise and easing

```slang
hash(p)                    // repeatable 0..1 per coordinate
value_noise(p)              // smooth
fbm(p)                      // several octaves - clouds, water, smoke
voronoi(p)                   // cellular noise - .f1/.f2/.id, cracks and cells
```

```slang
deg(degrees)                 // radians, so an angle reads the way it's written
bezier(t, x1, y1, x2, y2)    // the CSS four-number curve - one function instead of a table of named eases
bias(t, k)                   // cheap shaping: k above 1 pushes late, below 1 early
gain(t, k)                   // cheap S-curve: k above 1 steepens the middle
pulse(t)                     // 0 -> 1 -> 0
ping_pong(t)                 // bounces instead of wrapping, for `s.time`
```

## Recipes

**A ring.** Cut a smaller circle out of a bigger one:

```slang
let ring = Circle(s.size.x * 0.4).subtract(Circle(s.size.x * 0.2));
return ring.paint(s, s.color);
```

**A badge with an outline.** Draw the body, then the edge on top:

```slang
let body = Circle(s.size.x * 0.35);
return over(Color.orange.fade(body.outline(4.0).coverage(s.local)),
            body.paint(s, s.color));
```

**A pulsing highlight.** `s.time` drives it, `ping_pong` keeps it from
jumping:

```slang
let beat = ping_pong(s.time * 0.5);
return Circle(s.size.x * 0.5).paint(s, s.color.lighten(beat * 0.4));
```

**A vertical gradient:**

```slang
return float4(lerp(Color.cyan.rgb, Color.purple.rgb, s.uv.y), 1.0);
```

**Clouds:**

```slang
let n = fbm(s.uv * 4.0 + s.time * 0.05);
return float4(lerp(Color.black.rgb, Color.white.rgb, n), 1.0);
```

**Cracked stone.** `f1` shades each cell, `f2 - f1` is the distance to its
border - zero on it, so `smoothstep` draws mortar there and nowhere else:

```slang
let v = voronoi(s.uv * 6.0);
let stone = lerp(Color.grey.rgb, Color.white.rgb, hash(v.id));
let mortar = smoothstep(0.0, 0.06, v.f2 - v.f1);
return float4(lerp(Color.black.rgb, stone, mortar), 1.0);
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
| `ease` | `bezier`, `bias`, `gain`, `pulse`, `ping_pong` |
| `noise` | `hash`, `value_noise`, `fbm`, `voronoi` |
| `gradient` | `linear_gradient`, `radial_gradient`, `angular_gradient` |
| `pattern` | `checker`, `stripes`, `grid`, `dots` |
| `distort` | `wave`, `ripple`, `swirl`, `turbulence` |
