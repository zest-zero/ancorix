# Contributing

Thanks for looking. The project is young enough that an issue is often worth
more than a patch: if something is missing, awkward or slower than it should
be, say so before writing code for it, because the answer may be that the
abstraction has not earned its place yet.

## Before a pull request

```sh
cargo test --workspace --all-features
cargo clippy --workspace --all-features -- -D warnings
cargo fmt --all --check
```

```sh
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps
```

All four have to pass, and CI runs the same four. Doctests count - they are
compiled and run, so an example in a doc comment that no longer builds is a
failing test, which is the point of writing them. The doc build is there for
intra-doc links, which nothing else notices when they break.

To have all of it run before every push:

```sh
git config core.hooksPath .githooks
```

The minimum supported Rust version is 1.90, checked by its own CI job.

## Where things live

| crate | holds |
|---|---|
| `ancorix_ctx` | `Ctx`: input, time, window, drawing, assets |
| `ancorix_draw` | the draw queue and 2D primitives |
| `ancorix_render` | batching draw commands into Vulkan draw calls |
| `ancorix_ash` | the Vulkan layer |
| `ancorix_window` | window creation and the frame loop |
| `ancorix_input` | keys, mouse, named actions - knows nothing of windowing |
| `ancorix_winit` | the one bridge from winit events into that |
| `ancorix_math` | vectors and matrices |
| `ancorix_color` | `Rgba` |
| `ancorix_text` | fonts, glyph atlases, layout |
| `ancorix_image` | PNG and JPEG, pure Rust |
| `ancorix_asset` | handles and registries |
| `ancorix_shader` | compiled SPIR-V |
| `ancorix_slang` | the shader library, and where to find it |
| `ancorix_axb` | the binary asset format |
| `ancorix_build` | compiling project files into it |

Dependencies point one way. `ancorix_input` must not learn about winit,
`ancorix_ash` must not learn about anything but raw window handles, and no
windowing type may appear in the public API. A change that blurs one of those
lines needs to argue for itself.

One file is one unit of functionality. `lib.rs` holds `mod` and `pub use`,
never logic.

## Conventions

**Documentation.** Doc comments belong on the public API and nowhere else -
not on private items, not on `pub(crate)`, not in `build.rs`. A public item
says what it does or returns, then `# Examples`, then `# Panics` if it
panics, then `# See also` if it belongs to a family of methods. Internal
mechanics and the reasoning behind a decision go in `//` comments: rustdoc
goes stale silently, and a comment next to the code does not.

Crates the user never calls directly - `ancorix_ash`, `ancorix_render` -
document only what a function returns and when it panics. No examples there.

**Performance is not a later pass.** `const fn` wherever stable Rust allows.
`#[inline]` on thin wrappers, not on functions with logic in them. No
allocation in the frame loop.

**Measure, do not argue.** A contested decision is settled with a criterion
benchmark, and the version that lost stays in the bench file next to the one
that won, so the next person does not repeat the experiment. Benchmark
dependencies go in `dev-dependencies`, never anywhere else.

**Abstractions are extracted, not designed.** When a pattern has repeated -
usually the third time - it becomes a type. Not before.

**`unsafe` only where it is unavoidable**, which in practice means Vulkan and
FFI, and always with a comment saying why. Deserialisation uses
`from_le_bytes`, never a pointer cast.

**Versions of external crates live in the root `[workspace.dependencies]`**
and are inherited from there.

Code, comments and commit messages in English.

## Commits and pull requests

One commit is one change. If the subject line needs an "and", it is two
commits; if a rename is buried inside a bug fix, the fix cannot be read.

Write the subject in the imperative, under about fifty characters, with no
trailing full stop - `Fix scissor clamp on resize`, not `fixed some stuff`.
Leave a blank line, then use the body for **why**: what the code does is
already in the diff, what you were thinking is not. Say what you measured, or
what broke, or which of three approaches you threw away and on what evidence.

Rebase onto `main` rather than merging it in, so the history stays a line.
Squash the "oops, formatting" commits away before you open the pull request -
they are notes to yourself, not to the next reader.

A pull request needs a description that a stranger can act on: what changes,
why it is worth changing, and how you know it works. Link the issue if there
is one. If it touches performance, bring numbers from a real run and say which
machine produced them.

## Using AI

You may use a model. You may not submit what it wrote without understanding
it: whatever the patch says, the name on it is yours, and you will be asked to
defend the reasoning, not to relay it.

This project has already been burned by the alternative. A 2x2 matrix arrived
with fifty-six methods, among them `distance()` between two matrices and
`floor()` of a rotation, none of which mean anything geometrically, none of
which had a caller, all of them documented at length. Alongside it were a
method whose entire body called its neighbour under a different name, doc
comments on private items no user can reach, and `# See also` links pointing
at functions that were never written. Three thousand lines were deleted. That
is the shape of the problem: not wrong code, but plausible code, in volume,
that no one had a reason to write.

So the bar is the same as for anything else, and these are the questions the
review will actually ask:

- **Who calls this?** An API with no caller is a guess about the future. It
  gets written when the need arrives, not before.
- **Have you run it?** Not "does it compile" - run the example, look at the
  window, watch the thing you claim to have fixed stop happening.
- **Did you measure, or did the model?** Numbers come from `criterion` on your
  machine. A model's estimate of a speedup is not evidence, and neither is a
  benchmark that was never run.
- **Does the documentation describe what exists?** Every doc link resolves,
  every example compiles, every `# Panics` corresponds to a real panic.
- **Would you have written it this way?** If the answer is "probably not, but
  it looks fine", read it again until you have an opinion.

Small, understood patches are welcome from anyone, whatever wrote the first
draft. Large, unread ones are not, and are the fastest way to have a pull
request closed.

## Shaders

The engine's shaders are Slang, in `crates/ancorix_render/shaders`. The build
script compiles them with `slangc` when it is available and falls back to the
committed SPIR-V in `shaders/prebuilt` when it is not, so you can build the
workspace without a shader compiler installed. If you change a shader,
install `slangc` and refresh the committed output:

```sh
ANCORIX_SHADERS_UPDATE=1 cargo build -p ancorix_render
```

`ANCORIX_SLANGC` points at the compiler if it is not on `PATH`.

The shader library in `ancorix_slang` is tested on the CPU: `slangc` builds
the same modules into a console binary, so `cargo test -p ancorix_slang`
checks the maths without a GPU. Anything that depends on screen-space
derivatives - `aa` and `coverage` - cannot be tested that way.

## License

By contributing you agree that your work is licensed under the MIT license,
the same as the rest of the project.
