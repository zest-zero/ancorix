use ancorix_ash::Texture;
use ancorix_asset::{Assets, Handle};
use ancorix_color::Rgba;
use ancorix_draw::{Circle, DrawCmd, Line, Rect, RoundedRect, Sprite, Transform2D, Triangle};
use ancorix_math::Vector2;

#[cfg(feature = "unstable_shaders")]
use crate::vertex::ShadedVertex;
use crate::vertex::{RoundedRectInstance, SdfVertex, SpriteVertex, Vertex};

/// Which pipeline, and which texture for the one that samples, a [`DrawRun`]
/// is drawn with.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RunKind {
    Solid,
    /// Solid geometry drawn with a user pipeline instead of the built-in one
    /// (see `ancorix_draw::Draw::with_shader`).
    #[cfg(feature = "unstable_shaders")]
    SolidShaded(ancorix_draw::ShaderSlot),
    Sdf,
    RoundedRect,
    Sprite(Handle<Texture>),
}

/// A contiguous range of one pipeline's output, drawn as a single batch.
///
/// `first`/`count` index `solid_indices`, `sdf_indices` or `sprite_indices`
/// for those kinds, and `rounded_rect_instances` for
/// [`RunKind::RoundedRect`], which is drawn instanced.
// One run per *consecutive* sequence of same-kind commands, so queue order
// (and with it alpha stacking) is exactly what the caller asked for. Drawing
// a rect, a circle, then a rect again gives three runs, never two: merging
// the two rects would move the second one on top of the circle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DrawRun {
    pub kind: RunKind,
    pub first: u32,
    pub count: u32,
    /// The `with_clip` region this run was queued inside, in pixels, or the
    /// whole framebuffer if `None`.
    pub clip: Option<Rect>,
}

/// One frame's triangulated geometry: a vertex/index pair per pipeline, plus
/// the runs that put them back into the order the commands were queued in.
///
/// Kept by the renderer and reused every frame, so the vectors hold on to
/// their capacity instead of reallocating.
#[derive(Default)]
pub struct Geometry {
    pub solid_vertices: Vec<Vertex>,
    pub solid_indices: Vec<u32>,
    pub sdf_vertices: Vec<SdfVertex>,
    pub sdf_indices: Vec<u32>,
    pub rounded_rect_instances: Vec<RoundedRectInstance>,
    pub sprite_vertices: Vec<SpriteVertex>,
    pub sprite_indices: Vec<u32>,
    #[cfg(feature = "unstable_shaders")]
    pub shaded_vertices: Vec<ShadedVertex>,
    #[cfg(feature = "unstable_shaders")]
    pub shaded_indices: Vec<u32>,
    pub runs: Vec<DrawRun>,
}

impl Geometry {
    /// Returns empty geometry, with room for a typical frame already
    /// reserved (see `INITIAL_VERTEX_CAPACITY` in `buffers`).
    pub fn new() -> Self {
        use crate::buffers::{INITIAL_INDEX_CAPACITY, INITIAL_VERTEX_CAPACITY};

        Self {
            solid_vertices: Vec::with_capacity(INITIAL_VERTEX_CAPACITY),
            solid_indices: Vec::with_capacity(INITIAL_INDEX_CAPACITY),
            sdf_vertices: Vec::with_capacity(INITIAL_VERTEX_CAPACITY),
            sdf_indices: Vec::with_capacity(INITIAL_INDEX_CAPACITY),
            rounded_rect_instances: Vec::with_capacity(INITIAL_VERTEX_CAPACITY),
            sprite_vertices: Vec::with_capacity(INITIAL_VERTEX_CAPACITY),
            sprite_indices: Vec::with_capacity(INITIAL_INDEX_CAPACITY),
            #[cfg(feature = "unstable_shaders")]
            shaded_vertices: Vec::new(),
            #[cfg(feature = "unstable_shaders")]
            shaded_indices: Vec::new(),
            runs: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.solid_vertices.clear();
        self.solid_indices.clear();
        self.sdf_vertices.clear();
        self.sdf_indices.clear();
        self.rounded_rect_instances.clear();
        self.sprite_vertices.clear();
        self.sprite_indices.clear();
        #[cfg(feature = "unstable_shaders")]
        {
            self.shaded_vertices.clear();
            self.shaded_indices.clear();
        }
        self.runs.clear();
    }
}

/// Batches `cmds` into `out`, clearing it first.
///
/// `textures` is read only for each sprite's pixel size, which turns a
/// `Sprite::source` rect (authored in texture pixels) into normalized UVs.
///
/// # Panics
///
/// Panics if a sprite's `Handle<Texture>` isn't in `textures`.
pub fn triangulate(
    cmds: &[DrawCmd],
    viewport: Vector2,
    // only the shaded path carries it into a vertex; without that feature
    // there is nothing to hand it to
    #[cfg_attr(not(feature = "unstable_shaders"), allow(unused_variables))] time: f32,
    textures: &Assets<Texture>,
    out: &mut Geometry,
) {
    out.clear();

    // set by `DrawCmd::SetClip`, already intersected with any enclosing
    // region by `Draw::with_clip`
    let mut clip: Option<Rect> = None;

    // set by `DrawCmd::SetShader`, cleared at the end of the region
    #[cfg(feature = "unstable_shaders")]
    let mut shader: Option<ancorix_draw::ShaderSlot> = None;

    for &cmd in cmds {
        if let DrawCmd::SetClip(rect) = cmd {
            clip = rect;
            continue;
        }

        #[cfg(feature = "unstable_shaders")]
        if let DrawCmd::SetShader(handle) = cmd {
            shader = handle;
            continue;
        }

        // Inside a shader region the same shapes go to a different pipeline
        // *and* a different vertex format, since a user's `pixel` function
        // needs to know the shape it is inside.
        #[cfg(feature = "unstable_shaders")]
        let (kind, first, count) = match shader {
            Some(slot) => push_shaded(out, slot, cmd, viewport, time),
            None => push_plain(out, cmd, viewport, textures),
        };
        #[cfg(not(feature = "unstable_shaders"))]
        let (kind, first, count) = push_plain(out, cmd, viewport, textures);

        // a different clip needs its own scissor, so it breaks the run just
        // like a different pipeline does
        match out.runs.last_mut() {
            Some(run) if run.kind == kind && run.clip == clip => run.count += count,
            _ => out.runs.push(DrawRun {
                kind,
                first,
                count,
                clip,
            }),
        }
    }
}

// each push reports how much it appended, so the run's length never has to
// be measured off the buffer afterwards
fn push_plain(
    out: &mut Geometry,
    cmd: DrawCmd,
    viewport: Vector2,
    textures: &Assets<Texture>,
) -> (RunKind, u32, u32) {
    match cmd {
        DrawCmd::Clear(color) => {
            let first = out.solid_indices.len() as u32;
            let count = push_rect(
                &mut out.solid_vertices,
                &mut out.solid_indices,
                Rect::new(Vector2::ZERO, viewport),
                Transform2D::IDENTITY,
                color,
            );
            (RunKind::Solid, first, count)
        }
        DrawCmd::Rect {
            shape,
            transform,
            color,
        } => {
            let first = out.solid_indices.len() as u32;
            let count = push_rect(
                &mut out.solid_vertices,
                &mut out.solid_indices,
                shape,
                transform,
                color,
            );
            (RunKind::Solid, first, count)
        }
        DrawCmd::Triangle {
            shape,
            transform,
            color,
        } => {
            let first = out.solid_indices.len() as u32;
            let count = push_triangle(
                &mut out.solid_vertices,
                &mut out.solid_indices,
                shape,
                transform,
                color,
            );
            (RunKind::Solid, first, count)
        }
        DrawCmd::Line { shape, color } => {
            let first = out.solid_indices.len() as u32;
            let count = push_line(
                &mut out.solid_vertices,
                &mut out.solid_indices,
                shape,
                color,
            );
            (RunKind::Solid, first, count)
        }
        DrawCmd::Circle {
            shape,
            transform,
            color,
        } => {
            let first = out.sdf_indices.len() as u32;
            let count = push_circle(
                &mut out.sdf_vertices,
                &mut out.sdf_indices,
                shape,
                transform,
                color,
            );
            (RunKind::Sdf, first, count)
        }
        DrawCmd::RoundedRect {
            shape,
            transform,
            color,
        } => {
            let first = out.rounded_rect_instances.len() as u32;
            let count = push_rounded_rect(&mut out.rounded_rect_instances, shape, transform, color);
            (RunKind::RoundedRect, first, count)
        }
        DrawCmd::Sprite {
            shape,
            handle,
            transform,
            tint,
        } => {
            let (width, height) = textures
                .get(handle)
                .expect("Handle<Texture> doesn't belong to this Assets registry")
                .size();
            let texture_size = Vector2::new(width as f32, height as f32);

            let first = out.sprite_indices.len() as u32;
            let count = push_sprite(
                &mut out.sprite_vertices,
                &mut out.sprite_indices,
                shape,
                transform,
                tint,
                texture_size,
            );
            (RunKind::Sprite(handle), first, count)
        }
        // both handled by the `if let`s in `triangulate`, which skip to the
        // next command before this is reached
        DrawCmd::SetClip(_) => unreachable!(),
        #[cfg(feature = "unstable_shaders")]
        DrawCmd::SetShader(_) => unreachable!(),
    }
}

// `Draw::with_shader` rejects the shapes that have their own pipelines, so
// only the solid ones can reach this.
#[cfg(feature = "unstable_shaders")]
fn push_shaded(
    out: &mut Geometry,
    slot: ancorix_draw::ShaderSlot,
    cmd: DrawCmd,
    viewport: Vector2,
    time: f32,
) -> (RunKind, u32, u32) {
    let first = out.shaded_indices.len() as u32;

    let count = match cmd {
        DrawCmd::Clear(color) => push_rect_shaded(
            &mut out.shaded_vertices,
            &mut out.shaded_indices,
            Rect::new(Vector2::ZERO, viewport),
            Transform2D::IDENTITY,
            color,
            time,
        ),
        DrawCmd::Rect {
            shape,
            transform,
            color,
        } => push_rect_shaded(
            &mut out.shaded_vertices,
            &mut out.shaded_indices,
            shape,
            transform,
            color,
            time,
        ),
        DrawCmd::Triangle {
            shape,
            transform,
            color,
        } => push_triangle_shaded(
            &mut out.shaded_vertices,
            &mut out.shaded_indices,
            shape,
            transform,
            color,
            time,
        ),
        DrawCmd::Line { shape, color } => push_line_shaded(
            &mut out.shaded_vertices,
            &mut out.shaded_indices,
            shape,
            color,
            time,
        ),
        other => unreachable!("{other:?} can't be queued inside a shader region"),
    };

    (RunKind::SolidShaded(slot), first, count)
}

#[inline]
fn vertex_at(p: Vector2, color: Rgba) -> Vertex {
    Vertex {
        pos: [p.x, p.y],
        color: [color.r, color.g, color.b, color.a],
    }
}

fn push_rect(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    shape: Rect,
    transform: Transform2D,
    color: Rgba,
) -> u32 {
    let base = vertices.len() as u32;
    let corners = [
        shape.pos,
        shape.pos + Vector2::new(shape.size.x, 0.0),
        shape.pos + shape.size,
        shape.pos + Vector2::new(0.0, shape.size.y),
    ];

    for corner in corners {
        let p = transform.apply(corner, shape.pos, shape.size);
        vertices.push(vertex_at(p, color));
    }

    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    6
}

fn push_triangle(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    shape: Triangle,
    transform: Transform2D,
    color: Rgba,
) -> u32 {
    let bounds_min = shape.a.min(shape.b).min(shape.c);
    let bounds_max = shape.a.max(shape.b).max(shape.c);
    let bounds_size = bounds_max - bounds_min;
    let base = vertices.len() as u32;

    for corner in [shape.a, shape.b, shape.c] {
        let p = transform.apply(corner, bounds_min, bounds_size);
        vertices.push(vertex_at(p, color));
    }

    indices.extend_from_slice(&[base, base + 1, base + 2]);
    3
}

// One bounding quad, 4 vertices whatever the radius - `sdf2d.frag` finds the
// edge analytically.
//
// `local` stays *untransformed* and is still correct once `transform` skews
// the quad on screen: `transform` is affine, affine maps commute with the
// barycentric interpolation the GPU does across a triangle, so interpolating
// untransformed `local` over the transformed quad gives exactly what
// `Transform2D::invert` would compute at that point.
fn push_circle(
    vertices: &mut Vec<SdfVertex>,
    indices: &mut Vec<u32>,
    shape: Circle,
    transform: Transform2D,
    color: Rgba,
) -> u32 {
    let bounds_min = shape.pos - Vector2::splat(shape.radius);
    let bounds_size = Vector2::splat(shape.radius * 2.0);
    let base = vertices.len() as u32;

    let locals = [
        Vector2::new(-shape.radius, -shape.radius),
        Vector2::new(shape.radius, -shape.radius),
        Vector2::new(shape.radius, shape.radius),
        Vector2::new(-shape.radius, shape.radius),
    ];

    for local in locals {
        let p = transform.apply(shape.pos + local, bounds_min, bounds_size);
        vertices.push(SdfVertex {
            pos: [p.x, p.y],
            color: [color.r, color.g, color.b, color.a],
            local: [local.x, local.y],
            radius: shape.radius,
        });
    }

    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    6
}

// `roundedrect2d.vert` builds the six corners from `gl_VertexIndex`, so
// nothing is stored per vertex. The transform travels with the instance
// rather than being applied here: the shader runs the same pivot maths for
// free, saving the 108 bytes per shape four full vertices used to take.
fn push_rounded_rect(
    instances: &mut Vec<RoundedRectInstance>,
    shape: RoundedRect,
    transform: Transform2D,
    color: Rgba,
) -> u32 {
    instances.push(RoundedRectInstance {
        pos: [shape.pos.x, shape.pos.y],
        size: [shape.size.x, shape.size.y],
        origin: [transform.origin.x, transform.origin.y],
        scale: [transform.scale.x, transform.scale.y],
        radius: shape.radius,
        rotation: transform.rotation,
        color: [color.r, color.g, color.b, color.a],
    });
    1
}

// one quad, UV corners matching the same TL/TR/BR/BL corner order as
// push_rect. `shape.source` is in texture pixels, so it's divided by
// `texture_size` here - the shader samples in normalized UV, and neither
// the vertex format nor the sampler ever learns the texture's dimensions.
fn push_sprite(
    vertices: &mut Vec<SpriteVertex>,
    indices: &mut Vec<u32>,
    shape: Sprite,
    transform: Transform2D,
    tint: Rgba,
    texture_size: Vector2,
) -> u32 {
    let (uv_min, uv_max) = match shape.source {
        Some(source) => (
            source.pos / texture_size,
            (source.pos + source.size) / texture_size,
        ),
        None => (Vector2::ZERO, Vector2::ONE),
    };

    let base = vertices.len() as u32;
    let corners = [
        (shape.pos, [uv_min.x, uv_min.y]),
        (
            shape.pos + Vector2::new(shape.size.x, 0.0),
            [uv_max.x, uv_min.y],
        ),
        (shape.pos + shape.size, [uv_max.x, uv_max.y]),
        (
            shape.pos + Vector2::new(0.0, shape.size.y),
            [uv_min.x, uv_max.y],
        ),
    ];

    for (corner, uv) in corners {
        let p = transform.apply(corner, shape.pos, shape.size);
        vertices.push(SpriteVertex {
            pos: [p.x, p.y],
            uv,
            color: [tint.r, tint.g, tint.b, tint.a],
        });
    }

    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    6
}

fn push_line(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, shape: Line, color: Rgba) -> u32 {
    let dir = (shape.to - shape.from)
        .try_normalize()
        .unwrap_or(Vector2::X);
    let offset = dir.perp() * (shape.thickness * 0.5);
    let base = vertices.len() as u32;
    let corners = [
        shape.from - offset,
        shape.from + offset,
        shape.to + offset,
        shape.to - offset,
    ];

    for corner in corners {
        vertices.push(vertex_at(corner, color));
    }

    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    6
}

// The shaded pushes mirror the plain ones, but also record where each vertex
// sits inside its own shape - `uv` and `size` are what `Surface` is built
// from, and what lets one shader work for a shape of any size anywhere on
// screen.
#[cfg(feature = "unstable_shaders")]
fn push_rect_shaded(
    vertices: &mut Vec<ShadedVertex>,
    indices: &mut Vec<u32>,
    shape: Rect,
    transform: Transform2D,
    color: Rgba,
    time: f32,
) -> u32 {
    let base = vertices.len() as u32;
    let corners = [
        (shape.pos, [0.0, 0.0]),
        (shape.pos + Vector2::new(shape.size.x, 0.0), [1.0, 0.0]),
        (shape.pos + shape.size, [1.0, 1.0]),
        (shape.pos + Vector2::new(0.0, shape.size.y), [0.0, 1.0]),
    ];

    for (corner, uv) in corners {
        let p = transform.apply(corner, shape.pos, shape.size);
        vertices.push(ShadedVertex {
            pos: [p.x, p.y],
            color: [color.r, color.g, color.b, color.a],
            uv,
            size: [shape.size.x, shape.size.y],
            time,
        });
    }

    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    6
}

#[cfg(feature = "unstable_shaders")]
fn push_triangle_shaded(
    vertices: &mut Vec<ShadedVertex>,
    indices: &mut Vec<u32>,
    shape: Triangle,
    transform: Transform2D,
    color: Rgba,
    time: f32,
) -> u32 {
    // uv spans the triangle's bounding box, so a shader sees the same 0..1
    // square it would for a rect
    let bounds_min = shape.a.min(shape.b).min(shape.c);
    let bounds_size = shape.a.max(shape.b).max(shape.c) - bounds_min;
    let base = vertices.len() as u32;

    for corner in [shape.a, shape.b, shape.c] {
        let uv = (corner - bounds_min) / bounds_size;
        let p = transform.apply(corner, bounds_min, bounds_size);
        vertices.push(ShadedVertex {
            pos: [p.x, p.y],
            color: [color.r, color.g, color.b, color.a],
            uv: [uv.x, uv.y],
            size: [bounds_size.x, bounds_size.y],
            time,
        });
    }

    indices.extend_from_slice(&[base, base + 1, base + 2]);
    3
}

#[cfg(feature = "unstable_shaders")]
fn push_line_shaded(
    vertices: &mut Vec<ShadedVertex>,
    indices: &mut Vec<u32>,
    shape: Line,
    color: Rgba,
    time: f32,
) -> u32 {
    let dir = (shape.to - shape.from)
        .try_normalize()
        .unwrap_or(Vector2::X);
    let offset = dir.perp() * (shape.thickness * 0.5);
    let base = vertices.len() as u32;
    // uv runs along the line, not along the screen: x is progress from end to
    // end, y crosses the thickness
    let corners = [
        (shape.from - offset, [0.0, 0.0]),
        (shape.from + offset, [0.0, 1.0]),
        (shape.to + offset, [1.0, 1.0]),
        (shape.to - offset, [1.0, 0.0]),
    ];
    let size = [(shape.to - shape.from).length(), shape.thickness];

    for (corner, uv) in corners {
        vertices.push(ShadedVertex {
            pos: [corner.x, corner.y],
            color: [color.r, color.g, color.b, color.a],
            uv,
            size,
            time,
        });
    }

    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    6
}

#[cfg(test)]
mod tests {
    use super::*;

    // Sprites need a real `Texture`, which needs a GPU - every scene here is
    // built from the shape kinds that can be triangulated on the CPU alone,
    // which is enough to exercise every pipeline switch but one.
    fn run(cmds: &[DrawCmd]) -> Geometry {
        let mut out = Geometry::new();
        triangulate(
            cmds,
            Vector2::new(100.0, 100.0),
            0.0,
            &Assets::new(),
            &mut out,
        );
        out
    }

    fn rect() -> DrawCmd {
        DrawCmd::Rect {
            shape: Rect::new(Vector2::ZERO, Vector2::new(10.0, 10.0)),
            transform: Transform2D::IDENTITY,
            color: Rgba::WHITE,
        }
    }

    fn circle() -> DrawCmd {
        DrawCmd::Circle {
            shape: Circle::new(Vector2::ZERO, 5.0),
            transform: Transform2D::IDENTITY,
            color: Rgba::WHITE,
        }
    }

    fn rounded_rect() -> DrawCmd {
        DrawCmd::RoundedRect {
            shape: RoundedRect::new(Vector2::ZERO, Vector2::new(10.0, 10.0), 2.0),
            transform: Transform2D::IDENTITY,
            color: Rgba::WHITE,
        }
    }

    #[test]
    fn runs_follow_queue_order_not_pipeline() {
        let out = run(&[rect(), circle(), rect()]);

        assert_eq!(
            out.runs,
            [
                DrawRun {
                    kind: RunKind::Solid,
                    first: 0,
                    count: 6,
                    clip: None,
                },
                DrawRun {
                    kind: RunKind::Sdf,
                    first: 0,
                    count: 6,
                    clip: None,
                },
                // the second rect draws *after* the circle, so it must be its
                // own run - merging it into the first would put it underneath
                DrawRun {
                    kind: RunKind::Solid,
                    first: 6,
                    count: 6,
                    clip: None,
                },
            ]
        );
    }

    #[test]
    fn consecutive_same_kind_commands_merge() {
        let out = run(&[rect(), rect(), rect()]);

        assert_eq!(
            out.runs,
            [DrawRun {
                kind: RunKind::Solid,
                first: 0,
                count: 18,
                clip: None,
            }]
        );
    }

    #[test]
    fn rounded_rect_runs_count_instances_not_indices() {
        let out = run(&[rounded_rect(), circle(), rounded_rect()]);

        assert_eq!(out.rounded_rect_instances.len(), 2);
        assert_eq!(
            out.runs,
            [
                DrawRun {
                    kind: RunKind::RoundedRect,
                    first: 0,
                    count: 1,
                    clip: None,
                },
                DrawRun {
                    kind: RunKind::Sdf,
                    first: 0,
                    count: 6,
                    clip: None,
                },
                DrawRun {
                    kind: RunKind::RoundedRect,
                    first: 1,
                    count: 1,
                    clip: None,
                },
            ]
        );
    }

    #[test]
    fn clear_is_a_solid_run_wherever_it_is_queued() {
        let out = run(&[circle(), DrawCmd::Clear(Rgba::BLACK)]);

        // queued last, so it covers the circle - the whole point of the fix
        assert_eq!(
            out.runs,
            [
                DrawRun {
                    kind: RunKind::Sdf,
                    first: 0,
                    count: 6,
                    clip: None,
                },
                DrawRun {
                    kind: RunKind::Solid,
                    first: 0,
                    count: 6,
                    clip: None,
                },
            ]
        );
    }

    #[test]
    fn every_run_range_stays_inside_its_buffer() {
        let out = run(&[rect(), circle(), rounded_rect(), rect(), circle()]);

        for run in &out.runs {
            let len = match run.kind {
                RunKind::Solid => out.solid_indices.len(),
                #[cfg(feature = "unstable_shaders")]
                RunKind::SolidShaded(_) => out.solid_indices.len(),
                RunKind::Sdf => out.sdf_indices.len(),
                RunKind::RoundedRect => out.rounded_rect_instances.len(),
                RunKind::Sprite(_) => out.sprite_indices.len(),
            };
            assert!(
                (run.first + run.count) as usize <= len,
                "{run:?} runs past its buffer of {len}"
            );
        }
    }

    #[test]
    fn clip_breaks_the_run_and_travels_with_it() {
        let panel = Rect::new(Vector2::new(10.0, 10.0), Vector2::new(50.0, 50.0));
        let out = run(&[
            rect(),
            DrawCmd::SetClip(Some(panel)),
            rect(),
            DrawCmd::SetClip(None),
            rect(),
        ]);

        // same pipeline all three times, but the middle one needs its own
        // scissor, so it cannot share a batch with its neighbours
        assert_eq!(
            out.runs,
            [
                DrawRun {
                    kind: RunKind::Solid,
                    first: 0,
                    count: 6,
                    clip: None,
                },
                DrawRun {
                    kind: RunKind::Solid,
                    first: 6,
                    count: 6,
                    clip: Some(panel),
                },
                DrawRun {
                    kind: RunKind::Solid,
                    first: 12,
                    count: 6,
                    clip: None,
                },
            ]
        );
    }

    #[test]
    fn same_clip_keeps_commands_in_one_run() {
        let panel = Rect::new(Vector2::ZERO, Vector2::new(50.0, 50.0));
        let out = run(&[DrawCmd::SetClip(Some(panel)), rect(), rect()]);

        assert_eq!(out.runs.len(), 1);
        assert_eq!(out.runs[0].count, 12);
        assert_eq!(out.runs[0].clip, Some(panel));
    }

    #[test]
    fn empty_frame_produces_no_runs() {
        assert!(run(&[]).runs.is_empty());
    }
}
