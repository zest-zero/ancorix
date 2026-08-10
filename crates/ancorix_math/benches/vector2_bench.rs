//! `Vector2` against `glam::Vec2`, the most common alternative.
//!
//! Both are scalar, not SIMD - `glam`'s `Vec2` does not use `__m128` (its
//! source confirms it), so this is a fair comparison of two scalar
//! implementations rather than scalar against vectorized.
//!
//! Inputs are black-boxed so nothing gets folded at compile time: every
//! value here is a literal the optimizer could otherwise evaluate away.

use ancorix_math::Vector2;
use criterion::{Criterion, criterion_group, criterion_main};

fn bench_normalize(c: &mut Criterion) {
    let mut group = c.benchmark_group("normalize");

    group.bench_function("ancorix", |b| {
        b.iter(|| std::hint::black_box(std::hint::black_box(Vector2::new(3.0, 4.0)).normalize()));
    });
    group.bench_function("glam", |b| {
        b.iter(|| {
            std::hint::black_box(std::hint::black_box(glam::Vec2::new(3.0, 4.0)).normalize())
        });
    });

    // the degenerate input: ancorix returns early on a squared-length
    // epsilon, glam checks `is_finite` on the result
    group.bench_function("ancorix_zero", |b| {
        b.iter(|| std::hint::black_box(std::hint::black_box(Vector2::ZERO).try_normalize()));
    });
    group.bench_function("glam_zero", |b| {
        b.iter(|| std::hint::black_box(std::hint::black_box(glam::Vec2::ZERO).try_normalize()));
    });

    group.finish();
}

fn bench_length(c: &mut Criterion) {
    let mut group = c.benchmark_group("length");

    group.bench_function("ancorix", |b| {
        b.iter(|| std::hint::black_box(std::hint::black_box(Vector2::new(3.0, 4.0)).length()));
    });
    group.bench_function("glam", |b| {
        b.iter(|| std::hint::black_box(std::hint::black_box(glam::Vec2::new(3.0, 4.0)).length()));
    });

    // no square root - what movement code should reach for when comparing
    group.bench_function("ancorix_squared", |b| {
        b.iter(|| {
            std::hint::black_box(std::hint::black_box(Vector2::new(3.0, 4.0)).length_squared())
        });
    });

    group.finish();
}

fn bench_ops(c: &mut Criterion) {
    let mut group = c.benchmark_group("ops");

    let a = Vector2::new(1.5, -2.5);
    let b_vec = Vector2::new(0.25, 4.0);
    let ga = glam::Vec2::new(1.5, -2.5);
    let gb = glam::Vec2::new(0.25, 4.0);

    group.bench_function("ancorix_dot", |b| {
        b.iter(|| std::hint::black_box(std::hint::black_box(a).dot(std::hint::black_box(b_vec))));
    });
    group.bench_function("glam_dot", |b| {
        b.iter(|| std::hint::black_box(std::hint::black_box(ga).dot(std::hint::black_box(gb))));
    });

    group.bench_function("ancorix_distance", |b| {
        b.iter(|| {
            std::hint::black_box(std::hint::black_box(a).distance(std::hint::black_box(b_vec)))
        });
    });
    group.bench_function("glam_distance", |b| {
        b.iter(|| {
            std::hint::black_box(std::hint::black_box(ga).distance(std::hint::black_box(gb)))
        });
    });

    group.bench_function("ancorix_lerp", |b| {
        b.iter(|| {
            std::hint::black_box(std::hint::black_box(a).lerp(std::hint::black_box(b_vec), 0.35))
        });
    });
    group.bench_function("glam_lerp", |b| {
        b.iter(|| {
            std::hint::black_box(std::hint::black_box(ga).lerp(std::hint::black_box(gb), 0.35))
        });
    });

    group.finish();
}

criterion_group!(benches, bench_normalize, bench_length, bench_ops);
criterion_main!(benches);
