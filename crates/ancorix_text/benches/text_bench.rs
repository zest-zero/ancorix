//! Font atlas construction and glyph lookup.
//!
//! The lookup arm exists to check a choice made without measuring: glyphs
//! are kept in a `Vec` sorted by `char` and found by binary search, on the
//! assumption that beats hashing for ~100 entries looked up once per
//! character of every string drawn every frame. Both are measured here in
//! one run, along with a direct-index table as the floor.

#[cfg(feature = "ttf")]
use ancorix_text::charset;
use ancorix_text::{Glyph, builtin};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rustc_hash::FxHashMap;

/// Building the atlas happens once per font, at load - worth knowing so a
/// game can tell a hitch at startup from one mid-frame.
fn bench_atlas_build(c: &mut Criterion) {
    let mut group = c.benchmark_group("atlas_build");

    for &scale in &[1u32, 2, 4] {
        group.bench_with_input(BenchmarkId::new("builtin", scale), &scale, |b, &scale| {
            b.iter(|| std::hint::black_box(builtin(std::hint::black_box(scale))));
        });
    }

    #[cfg(feature = "ttf")]
    {
        const FONT: &[u8] = include_bytes!("../../../assets/fonts/JetBrainsMono-Regular.otf");

        for &px in &[16.0f32, 32.0] {
            group.bench_with_input(BenchmarkId::new("ttf_ascii", px as u32), &px, |b, &px| {
                b.iter(|| {
                    std::hint::black_box(ancorix_text::rasterize(
                        FONT,
                        std::hint::black_box(px),
                        charset::DEFAULT,
                    ))
                });
            });
        }

        // Same call, one glyph instead of 161. If parsing the font file
        // dominates, this lands near the full-charset number; if
        // rasterizing does, it collapses.
        group.bench_function("ttf_single_glyph", |b| {
            b.iter(|| {
                std::hint::black_box(ancorix_text::rasterize(
                    FONT,
                    std::hint::black_box(32.0),
                    "A",
                ))
            });
        });
    }

    group.finish();
}

/// One lookup per character of every string drawn - the hottest thing in
/// the text path after the draw queue itself.
fn bench_glyph_lookup(c: &mut Criterion) {
    let rasterized = builtin(2);
    let sorted = rasterized.glyphs.clone();

    let map: FxHashMap<char, Glyph> = sorted.iter().copied().collect();

    // the floor: printable ASCII is contiguous, so the index is arithmetic
    let mut table = [Glyph::default(); 128];
    for &(ch, glyph) in &sorted {
        table[ch as usize] = glyph;
    }

    // a mid-alphabet character, so binary search does its full depth rather
    // than getting lucky on the first probe
    let needle = 'q';

    let mut group = c.benchmark_group("glyph_lookup");

    group.bench_function("binary_search", |b| {
        b.iter(|| {
            let needle = std::hint::black_box(needle);
            std::hint::black_box(
                sorted
                    .binary_search_by_key(&needle, |(key, _)| *key)
                    .ok()
                    .map(|index| sorted[index].1),
            )
        });
    });

    group.bench_function("fxhashmap", |b| {
        b.iter(|| std::hint::black_box(map.get(&std::hint::black_box(needle)).copied()));
    });

    group.bench_function("direct_index", |b| {
        b.iter(|| std::hint::black_box(table[std::hint::black_box(needle) as usize]));
    });

    group.finish();
}

/// Looking up every character of a realistic string, which is what a frame
/// actually does - the per-lookup difference above multiplied by length.
fn bench_lookup_string(c: &mut Criterion) {
    let rasterized = builtin(2);
    let sorted = rasterized.glyphs.clone();
    let map: FxHashMap<char, Glyph> = sorted.iter().copied().collect();

    let text = "fps 144  dt 6.94 ms  pos 1280, 720  draws 37";

    let mut group = c.benchmark_group("lookup_string");

    group.bench_function("binary_search", |b| {
        b.iter(|| {
            let mut advance = 0.0f32;
            for ch in std::hint::black_box(text).chars() {
                if let Ok(index) = sorted.binary_search_by_key(&ch, |(key, _)| *key) {
                    advance += sorted[index].1.advance;
                }
            }
            std::hint::black_box(advance)
        });
    });

    group.bench_function("fxhashmap", |b| {
        b.iter(|| {
            let mut advance = 0.0f32;
            for ch in std::hint::black_box(text).chars() {
                if let Some(glyph) = map.get(&ch) {
                    advance += glyph.advance;
                }
            }
            std::hint::black_box(advance)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_atlas_build,
    bench_glyph_lookup,
    bench_lookup_string
);
criterion_main!(benches);
