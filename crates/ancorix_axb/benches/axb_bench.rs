//! Writing and reading `.axb` sections.
//!
//! This is startup cost, not per-frame cost: `ancorix_window::Runner`
//! compiles the project file's sections once, before the window exists. The
//! number that matters is therefore whether it is visible next to the rest
//! of startup (window creation, Vulkan device, font atlases), not whether
//! it fits a frame budget.

use ancorix_axb::{Section, SectionType, Writer, read};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

const ACTION_NAMES: [&str; 8] = [
    "move_left",
    "move_right",
    "move_up",
    "move_down",
    "jump",
    "interact",
    "pause",
    "quick_save",
];

fn input_section(actions: usize) -> Vec<u8> {
    let mut w = Writer::new(SectionType::Input, 1);
    w.u16(actions as u16);

    for index in 0..actions {
        w.str(ACTION_NAMES[index % ACTION_NAMES.len()]);
        w.u8(2);
        w.str("Space");
        w.str("Enter");
    }

    w.finish()
}

fn bench_write(c: &mut Criterion) {
    let mut group = c.benchmark_group("axb_write");

    for &actions in &[8usize, 64] {
        group.bench_with_input(BenchmarkId::from_parameter(actions), &actions, |b, &n| {
            b.iter(|| std::hint::black_box(input_section(std::hint::black_box(n))));
        });
    }

    group.finish();
}

fn bench_read(c: &mut Criterion) {
    let mut group = c.benchmark_group("axb_read");

    for &actions in &[8usize, 64] {
        let bytes = input_section(actions);

        group.bench_with_input(BenchmarkId::from_parameter(actions), &bytes, |b, bytes| {
            b.iter(|| {
                let sections = read(std::hint::black_box(bytes));
                // touch the result so the parse can't be optimized away
                let count = sections
                    .iter()
                    .filter(|section| matches!(section, Section::Input(_)))
                    .count();
                std::hint::black_box(count)
            });
        });
    }

    group.finish();
}

fn bench_round_trip(c: &mut Criterion) {
    // what actually happens at startup: compile the section, then hand the
    // bytes straight to the reader, all in memory
    c.bench_function("axb_round_trip_8", |b| {
        b.iter(|| {
            let bytes = input_section(std::hint::black_box(8));
            std::hint::black_box(read(&bytes).len())
        });
    });
}

criterion_group!(benches, bench_write, bench_read, bench_round_trip);
criterion_main!(benches);
