//! Benchmarks for `ancorix_render::geometry`.
//!
//! Four questions, matching what the code actually changed recently:
//! (1) `triangulate()` throughput on a realistic mixed scene - a baseline
//!     for the CPU cost of the batching step itself.
//! (2) how much cheaper the SDF `push_circle` (one quad, 4 vertices,
//!     regardless of radius) is than the triangle-fan approach it
//!     replaced. The fan code was deleted from the library entirely (see
//!     CLAUDE.md open question 24) - `fan_push_circle` below is a
//!     bench-only reconstruction of it, kept solely as a comparison
//!     baseline, never compiled into the actual crate.
//! (3) whether circle should stay on its own dedicated `SdfVertex` (20
//!     bytes/vertex, current) or merge into the rounded-rect pipeline's
//!     `RoundedRectVertex` (28 bytes/vertex) as a degenerate rounded rect
//!     (`half_size == (radius, radius)` makes `sdRoundedBox` reduce
//!     exactly to `sdCircle`) - `unified_push_circle` below is the
//!     bench-only reconstruction of that merged path, never wired in.
//! (5) whether the renderer's per-`Handle` caches are worth keying with
//!     `FxHashMap` instead of the std one - the lookup happens once per run
//!     per frame, and the key is a `u32`, not a hostile string.
//! (4) what making draw order follow queue order cost, now that a frame is
//!     cut into runs instead of one batch per pipeline - `grouped_batching`
//!     below is the pre-fix reconstruction, `ordered_batching` the current
//!     behaviour, both built from the same push helpers so only the run
//!     bookkeeping differs.

use ancorix_ash::Texture;
use ancorix_asset::Assets;
use ancorix_color::Rgba;
use ancorix_draw::{Circle, DrawCmd, Line, Rect, RoundedRect, Transform2D, Triangle};
use ancorix_math::Vector2;
use ancorix_render::geometry::triangulate;
use ancorix_render::vertex::{RoundedRectInstance, SdfVertex, Vertex};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

/// The merged vertex format question (3) measures - 28 bytes against
/// `SdfVertex`'s 20. Defined here rather than in the crate: the rounded-rect
/// pipeline is instanced (`RoundedRectInstance`) and never had a per-vertex
/// format, so this is a reconstruction of a path the library doesn't have,
/// same as `fan_push_circle` below.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct RoundedRectVertex {
    pos: [f32; 2],
    color: [u8; 4],
    local: [f32; 2],
    half_size: [f32; 2],
    radius: f32,
}

// Small deterministic LCG - just enough spread to avoid every shape
// landing on identical geometry, without pulling in a `rand` dependency
// for a benchmark-only scene generator.
struct Rng(u64);

impl Rng {
    fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.0 >> 33) as u32) as f32 / u32::MAX as f32
    }
}

fn mixed_scene(shapes_per_kind: usize) -> Vec<DrawCmd> {
    let mut rng = Rng(42);
    let mut cmds = Vec::with_capacity(shapes_per_kind * 4 + 1);
    cmds.push(DrawCmd::Clear(Rgba::from_hex("#1e1e1e")));

    for _ in 0..shapes_per_kind {
        let pos = Vector2::new(rng.next_f32() * 1920.0, rng.next_f32() * 1080.0);

        cmds.push(DrawCmd::Rect {
            shape: Rect::new(
                pos,
                Vector2::new(20.0 + rng.next_f32() * 80.0, 20.0 + rng.next_f32() * 80.0),
            ),
            transform: Transform2D::IDENTITY,
            color: Rgba::from_hex("#31a6ff"),
        });
        cmds.push(DrawCmd::Circle {
            shape: Circle::new(pos, 10.0 + rng.next_f32() * 40.0),
            transform: Transform2D::IDENTITY,
            color: Rgba::from_hex("#ff6b6b"),
        });
        cmds.push(DrawCmd::Triangle {
            shape: Triangle::new(
                pos,
                pos + Vector2::new(40.0, 0.0),
                pos + Vector2::new(20.0, 40.0),
            ),
            transform: Transform2D::IDENTITY,
            color: Rgba::from_hex("#ffd166"),
        });
        cmds.push(DrawCmd::Line {
            shape: Line::new(pos, pos + Vector2::new(60.0, 30.0), 4.0),
            color: Rgba::from_hex("#06d6a0"),
        });
    }

    cmds
}

fn bench_triangulate_mixed_scene(c: &mut Criterion) {
    let mut group = c.benchmark_group("triangulate_mixed_scene");

    for &shapes_per_kind in &[25usize, 100, 500] {
        let cmds = mixed_scene(shapes_per_kind);
        let viewport = Vector2::new(1920.0, 1080.0);

        let mut geometry = ancorix_render::geometry::Geometry::new();
        // this scene queues no sprites, so no texture is ever looked up.
        let textures = Assets::<Texture>::new();

        group.bench_with_input(
            BenchmarkId::from_parameter(shapes_per_kind * 4),
            &cmds,
            |b, cmds| {
                b.iter(|| {
                    triangulate(
                        std::hint::black_box(cmds),
                        viewport,
                        0.0,
                        &textures,
                        &mut geometry,
                    );
                });
            },
        );
    }

    group.finish();
}

/// Reconstruction of the pre-SDF triangle-fan circle triangulation, kept
/// only for benchmark comparison - not part of the library. Matches the
/// old `CIRCLE_SEGMENTS = 32` fixed segment count.
const FAN_CIRCLE_SEGMENTS: u32 = 32;

fn fan_push_circle(vertices: &mut Vec<Vertex>, indices: &mut Vec<u32>, shape: Circle, color: Rgba) {
    let base = vertices.len() as u32;
    let raw = [shape.pos.x, shape.pos.y];
    vertices.push(Vertex {
        pos: raw,
        color: [color.r, color.g, color.b, color.a],
    });

    for i in 0..=FAN_CIRCLE_SEGMENTS {
        let angle = (i as f32 / FAN_CIRCLE_SEGMENTS as f32) * std::f32::consts::TAU;
        let p = shape.pos + Vector2::new(angle.cos(), angle.sin()) * shape.radius;
        vertices.push(Vertex {
            pos: [p.x, p.y],
            color: [color.r, color.g, color.b, color.a],
        });
    }

    for i in 0..FAN_CIRCLE_SEGMENTS {
        indices.extend_from_slice(&[base, base + 1 + i, base + 2 + i]);
    }
}

fn sdf_push_circle(
    vertices: &mut Vec<SdfVertex>,
    indices: &mut Vec<u32>,
    shape: Circle,
    color: Rgba,
) -> u32 {
    let base = vertices.len() as u32;
    let locals = [
        Vector2::new(-shape.radius, -shape.radius),
        Vector2::new(shape.radius, -shape.radius),
        Vector2::new(shape.radius, shape.radius),
        Vector2::new(-shape.radius, shape.radius),
    ];

    for local in locals {
        let p = shape.pos + local;
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

/// Pushes a circle as a degenerate rounded rect (`half_size ==
/// (radius, radius)`, which makes `sdRoundedBox` reduce exactly to
/// `sdCircle` - see `crates/ancorix_render/shaders/roundedrect2d.frag`) using
/// `RoundedRectVertex` - a bench-only reconstruction of what merging
/// circle into the rounded-rect pipeline would cost, never wired into the
/// actual crate. Compared against `sdf_push_circle` (the real, currently
/// used dedicated `SdfVertex` path) to decide whether that merge is worth
/// doing - same method this file already used to decide fan vs SDF for
/// circles in the first place.
fn unified_push_circle(
    vertices: &mut Vec<RoundedRectVertex>,
    indices: &mut Vec<u32>,
    shape: Circle,
    color: Rgba,
) {
    let base = vertices.len() as u32;
    let locals = [
        Vector2::new(-shape.radius, -shape.radius),
        Vector2::new(shape.radius, -shape.radius),
        Vector2::new(shape.radius, shape.radius),
        Vector2::new(-shape.radius, shape.radius),
    ];

    for local in locals {
        let p = shape.pos + local;
        vertices.push(RoundedRectVertex {
            pos: [p.x, p.y],
            color: [color.r, color.g, color.b, color.a],
            local: [local.x, local.y],
            half_size: [shape.radius, shape.radius],
            radius: shape.radius,
        });
    }

    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

fn bench_circle_dedicated_vs_unified_vertex(c: &mut Criterion) {
    let mut group = c.benchmark_group("circle_dedicated_vs_unified_vertex");

    for &count in &[10usize, 100, 1000] {
        let mut rng = Rng(7);
        let circles: Vec<Circle> = (0..count)
            .map(|_| {
                Circle::new(
                    Vector2::new(rng.next_f32() * 1920.0, rng.next_f32() * 1080.0),
                    10.0 + rng.next_f32() * 40.0,
                )
            })
            .collect();
        let color = Rgba::from_hex("#ff6b6b");

        group.bench_with_input(
            BenchmarkId::new("dedicated_sdfvertex", count),
            &circles,
            |b, circles| {
                let mut vertices = Vec::new();
                let mut indices = Vec::new();
                b.iter(|| {
                    vertices.clear();
                    indices.clear();
                    for &circle in std::hint::black_box(circles) {
                        sdf_push_circle(&mut vertices, &mut indices, circle, color);
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("unified_roundedrectvertex", count),
            &circles,
            |b, circles| {
                let mut vertices = Vec::new();
                let mut indices = Vec::new();
                b.iter(|| {
                    vertices.clear();
                    indices.clear();
                    for &circle in std::hint::black_box(circles) {
                        unified_push_circle(&mut vertices, &mut indices, circle, color);
                    }
                });
            },
        );
    }

    group.finish();
}

fn bench_circles_fan_vs_sdf(c: &mut Criterion) {
    let mut group = c.benchmark_group("circles_fan_vs_sdf");

    for &count in &[10usize, 100, 1000] {
        let mut rng = Rng(7);
        let circles: Vec<Circle> = (0..count)
            .map(|_| {
                Circle::new(
                    Vector2::new(rng.next_f32() * 1920.0, rng.next_f32() * 1080.0),
                    10.0 + rng.next_f32() * 40.0,
                )
            })
            .collect();
        let color = Rgba::from_hex("#ff6b6b");

        group.bench_with_input(BenchmarkId::new("fan", count), &circles, |b, circles| {
            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            b.iter(|| {
                vertices.clear();
                indices.clear();
                for &circle in std::hint::black_box(circles) {
                    fan_push_circle(&mut vertices, &mut indices, circle, color);
                }
            });
        });

        group.bench_with_input(BenchmarkId::new("sdf", count), &circles, |b, circles| {
            let mut vertices = Vec::new();
            let mut indices = Vec::new();
            b.iter(|| {
                vertices.clear();
                indices.clear();
                for &circle in std::hint::black_box(circles) {
                    sdf_push_circle(&mut vertices, &mut indices, circle, color);
                }
            });
        });
    }

    group.finish();
}

/// Rounded rects on their own - the shape with the worst ratio of bytes
/// written to information carried (`half_size`, `radius` and `color` are
/// properties of the shape, yet stored once per vertex). Measured alone so
/// a change to that pipeline shows up undiluted by the other three.
fn bench_triangulate_rounded_rects(c: &mut Criterion) {
    let mut group = c.benchmark_group("triangulate_rounded_rects");

    for &count in &[100usize, 1000, 5000] {
        let mut rng = Rng(7);
        let cmds: Vec<DrawCmd> = (0..count)
            .map(|_| DrawCmd::RoundedRect {
                shape: RoundedRect::new(
                    Vector2::new(rng.next_f32() * 1920.0, rng.next_f32() * 1080.0),
                    Vector2::new(40.0 + rng.next_f32() * 80.0, 20.0 + rng.next_f32() * 40.0),
                    8.0,
                ),
                transform: Transform2D::IDENTITY,
                color: Rgba::from_hex("#31a6ff"),
            })
            .collect();

        let viewport = Vector2::new(1920.0, 1080.0);
        let textures = Assets::<Texture>::new();

        let mut geometry = ancorix_render::geometry::Geometry::new();

        group.bench_with_input(BenchmarkId::from_parameter(count), &cmds, |b, cmds| {
            b.iter(|| {
                triangulate(
                    std::hint::black_box(cmds),
                    viewport,
                    0.0,
                    &textures,
                    &mut geometry,
                );
            });
        });
    }

    group.finish();
}

// --- batching order: grouped (pre-fix) vs ordered (current) ---
//
// Question (4): the fix that made draw order follow queue order instead of
// pipeline groups. Both functions below build byte-identical geometry from
// the same local push helpers - the *only* difference is that `ordered_`
// also records the runs that put the output back in queue order. Comparing
// them measures exactly the bookkeeping the fix added, with nothing else
// moving. `grouped_batching` is the pre-fix reconstruction, never wired in.
//
// The other half of the fix's cost is on the GPU (one `vkCmdBindPipeline`
// per run instead of a fixed three), which a CPU benchmark cannot see -
// each scene prints its run count so that half is at least visible.

#[derive(Debug, Clone, Copy, PartialEq)]
enum BenchKind {
    Solid,
    Sdf,
    RoundedRect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct BenchRun {
    kind: BenchKind,
    first: u32,
    count: u32,
}

fn bench_push_rect(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u32>,
    shape: Rect,
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
        vertices.push(Vertex {
            pos: [corner.x, corner.y],
            color: [color.r, color.g, color.b, color.a],
        });
    }

    indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    6
}

fn bench_push_rounded_rect(
    instances: &mut Vec<RoundedRectInstance>,
    shape: RoundedRect,
    color: Rgba,
) -> u32 {
    instances.push(RoundedRectInstance {
        pos: [shape.pos.x, shape.pos.y],
        size: [shape.size.x, shape.size.y],
        origin: [0.0, 0.0],
        scale: [1.0, 1.0],
        radius: shape.radius,
        rotation: 0.0,
        color: [color.r, color.g, color.b, color.a],
    });
    1
}

struct BatchOutput {
    solid_vertices: Vec<Vertex>,
    solid_indices: Vec<u32>,
    sdf_vertices: Vec<SdfVertex>,
    sdf_indices: Vec<u32>,
    rounded_rect_instances: Vec<RoundedRectInstance>,
    runs: Vec<BenchRun>,
}

impl BatchOutput {
    fn new() -> Self {
        Self {
            solid_vertices: Vec::new(),
            solid_indices: Vec::new(),
            sdf_vertices: Vec::new(),
            sdf_indices: Vec::new(),
            rounded_rect_instances: Vec::new(),
            runs: Vec::new(),
        }
    }

    fn clear(&mut self) {
        self.solid_vertices.clear();
        self.solid_indices.clear();
        self.sdf_vertices.clear();
        self.sdf_indices.clear();
        self.rounded_rect_instances.clear();
        self.runs.clear();
    }
}

fn grouped_batching(cmds: &[DrawCmd], out: &mut BatchOutput) {
    out.clear();

    for &cmd in cmds {
        match cmd {
            DrawCmd::Rect { shape, color, .. } => {
                bench_push_rect(
                    &mut out.solid_vertices,
                    &mut out.solid_indices,
                    shape,
                    color,
                );
            }
            DrawCmd::Circle { shape, color, .. } => {
                sdf_push_circle(&mut out.sdf_vertices, &mut out.sdf_indices, shape, color);
            }
            DrawCmd::RoundedRect { shape, color, .. } => {
                bench_push_rounded_rect(&mut out.rounded_rect_instances, shape, color);
            }
            _ => unreachable!("scene only contains rects, circles and rounded rects"),
        }
    }
}

fn ordered_batching(cmds: &[DrawCmd], out: &mut BatchOutput) {
    out.clear();

    for &cmd in cmds {
        let (kind, first) = match cmd {
            DrawCmd::Rect { shape, color, .. } => {
                let first = out.solid_indices.len() as u32;
                bench_push_rect(
                    &mut out.solid_vertices,
                    &mut out.solid_indices,
                    shape,
                    color,
                );
                (BenchKind::Solid, first)
            }
            DrawCmd::Circle { shape, color, .. } => {
                let first = out.sdf_indices.len() as u32;
                sdf_push_circle(&mut out.sdf_vertices, &mut out.sdf_indices, shape, color);
                (BenchKind::Sdf, first)
            }
            DrawCmd::RoundedRect { shape, color, .. } => {
                let first = out.rounded_rect_instances.len() as u32;
                bench_push_rounded_rect(&mut out.rounded_rect_instances, shape, color);
                (BenchKind::RoundedRect, first)
            }
            _ => unreachable!("scene only contains rects, circles and rounded rects"),
        };

        let end = match kind {
            BenchKind::Solid => out.solid_indices.len(),
            BenchKind::Sdf => out.sdf_indices.len(),
            BenchKind::RoundedRect => out.rounded_rect_instances.len(),
        } as u32;

        match out.runs.last_mut() {
            Some(run) if run.kind == kind => run.count += end - first,
            _ => out.runs.push(BenchRun {
                kind,
                first,
                count: end - first,
            }),
        }
    }
}

// The same ordering, with the bookkeeping moved into registers: counts come
// from the push helpers instead of being measured off the buffers, buffer
// positions are cursors, and the open run only reaches the vector when the
// pipeline actually changes. This is what the library does now.
fn ordered_fast_batching(cmds: &[DrawCmd], out: &mut BatchOutput) {
    out.clear();

    let (mut solid_at, mut sdf_at, mut rounded_rect_at) = (0u32, 0u32, 0u32);
    let mut open: Option<BenchRun> = None;

    for &cmd in cmds {
        let (kind, first, count) = match cmd {
            DrawCmd::Rect { shape, color, .. } => {
                let first = solid_at;
                bench_push_rect(
                    &mut out.solid_vertices,
                    &mut out.solid_indices,
                    shape,
                    color,
                );
                solid_at += 6;
                (BenchKind::Solid, first, 6)
            }
            DrawCmd::Circle { shape, color, .. } => {
                let first = sdf_at;
                sdf_push_circle(&mut out.sdf_vertices, &mut out.sdf_indices, shape, color);
                sdf_at += 6;
                (BenchKind::Sdf, first, 6)
            }
            DrawCmd::RoundedRect { shape, color, .. } => {
                let first = rounded_rect_at;
                bench_push_rounded_rect(&mut out.rounded_rect_instances, shape, color);
                rounded_rect_at += 1;
                (BenchKind::RoundedRect, first, 1)
            }
            _ => unreachable!("scene only contains rects, circles and rounded rects"),
        };

        match &mut open {
            Some(run) if run.kind == kind => run.count += count,
            Some(run) => {
                out.runs.push(*run);
                *run = BenchRun { kind, first, count };
            }
            None => open = Some(BenchRun { kind, first, count }),
        }
    }

    if let Some(run) = open {
        out.runs.push(run);
    }
}

// Only the part of `ordered_fast_batching` that turned out to be worth
// keeping: the count comes back from the push instead of being measured off
// the buffer afterwards, which drops a whole `match` and one length read per
// command. Everything else stays exactly as `ordered_batching` had it.
fn ordered_hybrid_batching(cmds: &[DrawCmd], out: &mut BatchOutput) {
    out.clear();

    for &cmd in cmds {
        let (kind, first, count) = match cmd {
            DrawCmd::Rect { shape, color, .. } => {
                let first = out.solid_indices.len() as u32;
                let count = bench_push_rect(
                    &mut out.solid_vertices,
                    &mut out.solid_indices,
                    shape,
                    color,
                );
                (BenchKind::Solid, first, count)
            }
            DrawCmd::Circle { shape, color, .. } => {
                let first = out.sdf_indices.len() as u32;
                let count =
                    sdf_push_circle(&mut out.sdf_vertices, &mut out.sdf_indices, shape, color);
                (BenchKind::Sdf, first, count)
            }
            DrawCmd::RoundedRect { shape, color, .. } => {
                let first = out.rounded_rect_instances.len() as u32;
                let count = bench_push_rounded_rect(&mut out.rounded_rect_instances, shape, color);
                (BenchKind::RoundedRect, first, count)
            }
            _ => unreachable!("scene only contains rects, circles and rounded rects"),
        };

        match out.runs.last_mut() {
            Some(run) if run.kind == kind => run.count += count,
            _ => out.runs.push(BenchRun { kind, first, count }),
        }
    }
}

fn shape_cmds(count: usize, interleaved: bool) -> Vec<DrawCmd> {
    let mut rng = Rng(11);
    let mut kinds: Vec<usize> = (0..count).map(|i| i % 3).collect();
    if !interleaved {
        // every rect, then every circle, then every rounded rect: the layered
        // scene a real frame usually is (background, shapes, panels)
        kinds.sort_unstable();
    }

    kinds
        .into_iter()
        .map(|kind| {
            let pos = Vector2::new(rng.next_f32() * 1920.0, rng.next_f32() * 1080.0);
            let size = Vector2::new(20.0 + rng.next_f32() * 80.0, 20.0 + rng.next_f32() * 80.0);

            match kind {
                0 => DrawCmd::Rect {
                    shape: Rect::new(pos, size),
                    transform: Transform2D::IDENTITY,
                    color: Rgba::from_hex("#31a6ff"),
                },
                1 => DrawCmd::Circle {
                    shape: Circle::new(pos, 10.0 + rng.next_f32() * 40.0),
                    transform: Transform2D::IDENTITY,
                    color: Rgba::from_hex("#ff6b6b"),
                },
                _ => DrawCmd::RoundedRect {
                    shape: RoundedRect::new(pos, size, 8.0),
                    transform: Transform2D::IDENTITY,
                    color: Rgba::from_hex("#ffd166"),
                },
            }
        })
        .collect()
}

fn bench_grouped_vs_ordered_batching(c: &mut Criterion) {
    let mut group = c.benchmark_group("grouped_vs_ordered_batching");

    for &interleaved in &[false, true] {
        let layout = if interleaved {
            "interleaved"
        } else {
            "layered"
        };

        for &count in &[100usize, 1000] {
            let cmds = shape_cmds(count, interleaved);
            let mut out = BatchOutput::new();

            ordered_batching(&cmds, &mut out);
            eprintln!(
                "{layout}/{count}: {} pipeline binds after the fix, 3 before",
                out.runs.len()
            );

            group.bench_with_input(
                BenchmarkId::new(format!("grouped_{layout}"), count),
                &cmds,
                |b, cmds| b.iter(|| grouped_batching(std::hint::black_box(cmds), &mut out)),
            );
            group.bench_with_input(
                BenchmarkId::new(format!("ordered_{layout}"), count),
                &cmds,
                |b, cmds| b.iter(|| ordered_batching(std::hint::black_box(cmds), &mut out)),
            );
            group.bench_with_input(
                BenchmarkId::new(format!("ordered_hybrid_{layout}"), count),
                &cmds,
                |b, cmds| b.iter(|| ordered_hybrid_batching(std::hint::black_box(cmds), &mut out)),
            );
            group.bench_with_input(
                BenchmarkId::new(format!("ordered_fast_{layout}"), count),
                &cmds,
                |b, cmds| b.iter(|| ordered_fast_batching(std::hint::black_box(cmds), &mut out)),
            );
        }
    }

    group.finish();
}

fn bench_cache_lookup(c: &mut Criterion) {
    use ancorix_asset::Assets as Registry;
    use rustc_hash::FxHashMap;
    use std::collections::HashMap;

    let mut group = c.benchmark_group("descriptor_cache_lookup");

    // handles as the renderer really gets them: dense indices out of a
    // registry. The payload type is irrelevant - `Handle<T>` hashes its u32
    // index, and a real `Texture` would need a GPU.
    for &textures in &[4usize, 32] {
        let mut registry = Registry::<u32>::new();
        let handles: Vec<_> = (0..textures as u32).map(|i| registry.insert(i)).collect();

        let std_map: HashMap<_, u32> = handles.iter().map(|&h| (h, 1u32)).collect();
        let fx_map: FxHashMap<_, u32> = handles.iter().map(|&h| (h, 1u32)).collect();

        group.bench_with_input(
            BenchmarkId::new("std_hashmap", textures),
            &handles,
            |b, handles| {
                b.iter(|| {
                    let mut sum = 0u32;
                    for h in std::hint::black_box(handles) {
                        sum += std_map[h];
                    }
                    sum
                });
            },
        );
        group.bench_with_input(
            BenchmarkId::new("fxhashmap", textures),
            &handles,
            |b, handles| {
                b.iter(|| {
                    let mut sum = 0u32;
                    for h in std::hint::black_box(handles) {
                        sum += fx_map[h];
                    }
                    sum
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_triangulate_mixed_scene,
    bench_triangulate_rounded_rects,
    bench_circles_fan_vs_sdf,
    bench_circle_dedicated_vs_unified_vertex,
    bench_grouped_vs_ordered_batching,
    bench_cache_lookup
);
criterion_main!(benches);
