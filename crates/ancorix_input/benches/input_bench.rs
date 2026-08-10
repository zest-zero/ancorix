//! What input actually costs per frame.
//!
//! The suspect is action lookup: actions live in a `HashMap<Box<str>, _>`,
//! so every `action_pressed("jump")` hashes the string again. A game polls
//! several actions per frame - `action_vector` alone is four lookups - so
//! this is measured against `is_pressed`, a plain array index, to see how
//! much the string layer costs over the floor.

use ancorix_input::{Binding, Input, Key};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rustc_hash::FxHashMap;

// Enough distinct names that the map isn't a degenerate one-bucket case,
// and long enough to hash realistically - "jump", "crouch" and friends.
const ACTION_NAMES: [&str; 16] = [
    "move_left",
    "move_right",
    "move_up",
    "move_down",
    "jump",
    "crouch",
    "sprint",
    "interact",
    "attack_primary",
    "attack_secondary",
    "reload",
    "inventory",
    "map",
    "pause",
    "quick_save",
    "quick_load",
];

const KEYS: [Key; 16] = [
    Key::A,
    Key::D,
    Key::W,
    Key::S,
    Key::Space,
    Key::C,
    Key::ShiftLeft,
    Key::E,
    Key::Q,
    Key::F,
    Key::R,
    Key::I,
    Key::M,
    Key::Escape,
    Key::F5,
    Key::F9,
];

fn input_with(action_count: usize) -> Input {
    let mut input = Input::new();

    for index in 0..action_count {
        // more actions than names: suffix the extras so every key is unique
        let name = if index < ACTION_NAMES.len() {
            ACTION_NAMES[index].to_string()
        } else {
            format!("{}_{index}", ACTION_NAMES[index % ACTION_NAMES.len()])
        };
        input.bind_keys(name, &[KEYS[index % KEYS.len()]]);
    }

    // a realistic frame: a few keys down, not none and not all
    input.press_key(Key::W);
    input.press_key(Key::D);
    input.press_key(Key::ShiftLeft);

    input
}

fn bench_action_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("action_pressed");

    for &count in &[4usize, 16, 64] {
        let input = input_with(count);

        group.bench_with_input(BenchmarkId::from_parameter(count), &count, |b, _| {
            b.iter(|| {
                // a miss and a hit - a real frame does both
                std::hint::black_box(input.action_pressed(std::hint::black_box("move_up")));
                std::hint::black_box(input.action_pressed(std::hint::black_box("jump")));
            });
        });
    }

    group.finish();
}

fn bench_lookup_vs_floor(c: &mut Criterion) {
    let input = input_with(16);
    let mut group = c.benchmark_group("lookup_vs_floor");

    // the floor: no string, no hash, just an array index
    group.bench_function("is_pressed", |b| {
        b.iter(|| std::hint::black_box(input.is_pressed(std::hint::black_box(Key::W))));
    });

    group.bench_function("action_pressed", |b| {
        b.iter(|| std::hint::black_box(input.action_pressed(std::hint::black_box("jump"))));
    });

    // what movement costs every single frame
    group.bench_function("action_vector", |b| {
        b.iter(|| {
            std::hint::black_box(input.action_vector(
                std::hint::black_box("move_left"),
                "move_right",
                "move_up",
                "move_down",
            ))
        });
    });

    group.finish();
}

fn bench_begin_frame(c: &mut Criterion) {
    let mut input = input_with(16);

    // called once per frame no matter what the app does
    c.bench_function("begin_frame", |b| {
        b.iter(|| {
            input.begin_frame();
            std::hint::black_box(&input);
        });
    });
}

/// Three ways to get from an action to its bindings, over identical data
/// and doing identical work once found, so the only difference measured is
/// the lookup itself. Compared in one run on purpose - absolute timings on
/// a loaded laptop drift between runs, ratios within a run don't.
fn bench_lookup_strategies(c: &mut Criterion) {
    use std::collections::HashMap;

    let names: Vec<Box<str>> = (0..16)
        .map(|index| ACTION_NAMES[index].to_string().into_boxed_str())
        .collect();
    let bindings: Vec<Vec<Binding>> = (0..16)
        .map(|index| {
            vec![
                Binding::Key(KEYS[index]),
                Binding::Key(KEYS[(index + 8) % 16]),
            ]
        })
        .collect();

    let std_map: HashMap<Box<str>, Vec<Binding>> = names
        .iter()
        .cloned()
        .zip(bindings.iter().cloned())
        .collect();
    let fx_map: FxHashMap<Box<str>, Vec<Binding>> = names
        .iter()
        .cloned()
        .zip(bindings.iter().cloned())
        .collect();

    // what the engine holds today is `keys_cur`; mirror it so the "is any
    // binding down" work is the same in every arm
    let mut keys = [false; Key::COUNT];
    keys[Key::W as usize] = true;

    let pressed = |list: Option<&Vec<Binding>>| {
        list.map(|list| {
            list.iter()
                .any(|binding| matches!(binding, Binding::Key(key) if keys[*key as usize]))
        })
        .unwrap_or(false)
    };

    let mut group = c.benchmark_group("lookup_strategy");

    group.bench_function("std_hashmap", |b| {
        b.iter(|| std::hint::black_box(pressed(std_map.get(std::hint::black_box("jump")))));
    });

    group.bench_function("fxhashmap", |b| {
        b.iter(|| std::hint::black_box(pressed(fx_map.get(std::hint::black_box("jump")))));
    });

    // the ideal: the name was resolved to an index once, at startup
    group.bench_function("interned_index", |b| {
        b.iter(|| std::hint::black_box(pressed(bindings.get(std::hint::black_box(4usize)))));
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_action_lookup,
    bench_lookup_vs_floor,
    bench_begin_frame,
    bench_lookup_strategies
);
criterion_main!(benches);
