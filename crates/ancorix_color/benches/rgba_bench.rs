//! Colour construction.
//!
//! `from_hex` is a `const fn`, so a literal like `rgba!("#1e1e1e")` costs
//! nothing at runtime - the interesting number is what it costs when the
//! string only becomes known at runtime, which is what a config file or a
//! colour picker produces. `from_hsv` is not const and runs per shape in
//! scenes that cycle hue.

use ancorix_color::Rgba;
use criterion::{Criterion, criterion_group, criterion_main};

fn bench_construct(c: &mut Criterion) {
    let mut group = c.benchmark_group("rgba_construct");

    // the floor: four bytes into four fields
    group.bench_function("new", |b| {
        b.iter(|| std::hint::black_box(Rgba::new(std::hint::black_box(30), 30, 30, 255)));
    });

    group.bench_function("from_hex_rgb", |b| {
        b.iter(|| std::hint::black_box(Rgba::from_hex(std::hint::black_box("#1e1e1e"))));
    });

    group.bench_function("from_hex_rgba", |b| {
        b.iter(|| std::hint::black_box(Rgba::from_hex(std::hint::black_box("#1e1e1e80"))));
    });

    group.bench_function("from_hsv", |b| {
        b.iter(|| std::hint::black_box(Rgba::from_hsv(std::hint::black_box(210.0), 0.8, 0.9)));
    });

    group.finish();
}

fn bench_adjust(c: &mut Criterion) {
    let mut group = c.benchmark_group("rgba_adjust");

    let color = Rgba::new(90, 140, 200, 255);

    group.bench_function("lighten", |b| {
        b.iter(|| std::hint::black_box(std::hint::black_box(color).lighten(0.2)));
    });
    group.bench_function("darken", |b| {
        b.iter(|| std::hint::black_box(std::hint::black_box(color).darken(0.2)));
    });
    group.bench_function("with_alpha", |b| {
        b.iter(|| std::hint::black_box(std::hint::black_box(color).with_alpha(128)));
    });

    group.finish();
}

criterion_group!(benches, bench_construct, bench_adjust);
criterion_main!(benches);
