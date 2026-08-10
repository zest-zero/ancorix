//! What the immediate-mode queue costs per command, and what hit-testing
//! costs per call.
//!
//! Every number here is paired with a cheaper reference measured in the
//! same run - absolute timings drift between runs on a loaded machine,
//! ratios inside one run don't.

use ancorix_color::Rgba;
use ancorix_draw::{Circle, Draw, Rect, RoundedRect, Transform2D, Triangle};
use ancorix_math::{Vector2, v2};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

// Small deterministic LCG, so the scene is spread out but reproducible and
// no `rand` dependency is pulled in for a benchmark.
struct Rng(u64);

impl Rng {
    fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.0 >> 33) as u32) as f32 / u32::MAX as f32
    }
}

fn bench_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("push_command");

    // one queue reused across iterations, cleared each time - measuring the
    // push, not the allocator warming up
    let mut draw = Draw::new();

    group.bench_function("rect", |b| {
        b.iter(|| {
            draw.rect(
                std::hint::black_box(Rect::new(v2!(10.0, 20.0), v2!(30.0, 40.0))),
                Rgba::WHITE,
            );
            draw.flush();
        });
    });

    group.bench_function("rect_ex", |b| {
        b.iter(|| {
            draw.rect_ex(
                std::hint::black_box(Rect::new(v2!(10.0, 20.0), v2!(30.0, 40.0))),
                Transform2D::rotated(0.5),
                Rgba::WHITE,
            );
            draw.flush();
        });
    });

    group.bench_function("circle", |b| {
        b.iter(|| {
            draw.circle(
                std::hint::black_box(Circle::new(v2!(10.0, 20.0), 15.0)),
                Rgba::WHITE,
            );
            draw.flush();
        });
    });

    group.bench_function("rounded_rect", |b| {
        b.iter(|| {
            draw.rounded_rect(
                std::hint::black_box(RoundedRect::new(v2!(10.0, 20.0), v2!(30.0, 40.0), 6.0)),
                Rgba::WHITE,
            );
            draw.flush();
        });
    });

    group.finish();
}

/// A whole frame's worth of queueing, to see whether cost stays linear in
/// command count or the queue's growth starts showing.
fn bench_queue_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("queue_frame");

    for &count in &[100usize, 1000, 5000] {
        let mut rng = Rng(42);
        let shapes: Vec<Rect> = (0..count)
            .map(|_| {
                Rect::new(
                    Vector2::new(rng.next_f32() * 1920.0, rng.next_f32() * 1080.0),
                    v2!(16.0, 16.0),
                )
            })
            .collect();

        let mut draw = Draw::new();

        group.bench_with_input(BenchmarkId::from_parameter(count), &shapes, |b, shapes| {
            b.iter(|| {
                draw.clear(Rgba::BLACK);
                for &shape in shapes {
                    draw.rect(shape, Rgba::WHITE);
                }
                std::hint::black_box(draw.commands().len());
                draw.flush();
            });
        });
    }

    group.finish();
}

/// Hit-testing. `contains_ex` inverts the point through the transform and
/// then calls plain `contains`, so the pair shows exactly what the
/// transform costs on top.
fn bench_contains(c: &mut Criterion) {
    let mut group = c.benchmark_group("contains");

    let rect = Rect::new(v2!(10.0, 10.0), v2!(100.0, 60.0));
    let circle = Circle::new(v2!(60.0, 40.0), 30.0);
    let rounded = RoundedRect::new(v2!(10.0, 10.0), v2!(100.0, 60.0), 12.0);
    let triangle = Triangle::new(v2!(10.0, 10.0), v2!(110.0, 10.0), v2!(60.0, 70.0));
    let transform = Transform2D::rotated(0.7);
    let point = v2!(55.0, 35.0);

    group.bench_function("rect", |b| {
        b.iter(|| std::hint::black_box(rect.contains(std::hint::black_box(point))));
    });
    group.bench_function("rect_ex", |b| {
        b.iter(|| std::hint::black_box(rect.contains_ex(std::hint::black_box(point), transform)));
    });
    group.bench_function("circle", |b| {
        b.iter(|| std::hint::black_box(circle.contains(std::hint::black_box(point))));
    });
    group.bench_function("rounded_rect", |b| {
        b.iter(|| std::hint::black_box(rounded.contains(std::hint::black_box(point))));
    });
    group.bench_function("triangle", |b| {
        b.iter(|| std::hint::black_box(triangle.contains(std::hint::black_box(point))));
    });

    group.finish();
}

/// `Transform2D::apply` runs per vertex during triangulation, so its cost
/// is multiplied by four for every quad on screen.
fn bench_transform(c: &mut Criterion) {
    let mut group = c.benchmark_group("transform2d");

    let transform = Transform2D::rotated_around(0.7, v2!(0.5, 0.5));
    let identity = Transform2D::IDENTITY;
    let point = v2!(55.0, 35.0);
    let (min, size) = (v2!(10.0, 10.0), v2!(100.0, 60.0));

    // The transform is black-boxed too, not just the point: it is a loop
    // invariant here, and without the barrier the compiler is free to hoist
    // the sin/cos out of the loop and measure something that never happens
    // in a real frame, where the rotation differs per shape.
    group.bench_function("apply_identity", |b| {
        b.iter(|| {
            std::hint::black_box(std::hint::black_box(identity).apply(
                std::hint::black_box(point),
                min,
                size,
            ))
        });
    });
    group.bench_function("apply_rotated", |b| {
        b.iter(|| {
            std::hint::black_box(std::hint::black_box(transform).apply(
                std::hint::black_box(point),
                min,
                size,
            ))
        });
    });
    group.bench_function("invert_rotated", |b| {
        b.iter(|| {
            std::hint::black_box(std::hint::black_box(transform).invert(
                std::hint::black_box(point),
                min,
                size,
            ))
        });
    });

    // The pre-short-circuit implementation, reconstructed here only as a
    // baseline so both live in one run - the machine is loaded enough that
    // comparing across runs is meaningless. Never compiled into the crate.
    fn apply_unconditional(t: Transform2D, p: Vector2, min: Vector2, size: Vector2) -> Vector2 {
        let pivot = min + t.origin.component_mul(size);
        let local = (p - pivot).component_mul(t.scale).rotated(t.rotation);
        pivot + local
    }

    group.bench_function("apply_identity_no_shortcircuit", |b| {
        b.iter(|| {
            std::hint::black_box(apply_unconditional(
                std::hint::black_box(identity),
                std::hint::black_box(point),
                min,
                size,
            ))
        });
    });
    group.bench_function("apply_rotated_no_shortcircuit", |b| {
        b.iter(|| {
            std::hint::black_box(apply_unconditional(
                std::hint::black_box(transform),
                std::hint::black_box(point),
                min,
                size,
            ))
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_push,
    bench_queue_frame,
    bench_contains,
    bench_transform
);
criterion_main!(benches);
