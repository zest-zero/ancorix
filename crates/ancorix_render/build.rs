use std::ffi::OsString;
use std::path::Path;
use std::process::Command;
use std::{env, fs};

// Listed, not discovered by walking `shaders/`: a shader only matters if
// some pipeline includes it, and that list lives in `src/pipelines.rs`.
//
// One `.slang` module per pipeline, holding both stages - the vertex and
// fragment halves of a pipeline share the varyings struct, and Slang lets
// them share it as a declaration instead of two matching copies.
// Entry point and the artifact suffix each produces. The SPIR-V entry point
// is always emitted as `main`, whatever the function is called here.
const BOTH: &[(&str, &str)] = &[("vertexMain", "vert"), ("fragmentMain", "frag")];

// `shaded2d` is the vertex half the engine pairs with a *user's* fragment
// shader, so it has no fragment stage of its own.
const VERTEX_ONLY: &[(&str, &str)] = &[("vertexMain", "vert")];

const SHADERS: [(&str, &[(&str, &str)]); 5] = [
    ("solid2d", BOTH),
    ("sdf2d", BOTH),
    ("roundedrect2d", BOTH),
    ("sprite2d", BOTH),
    ("shaded2d", VERTEX_ONLY),
];

const COMPILER: &str = "slangc";

// Overrides `COMPILER` with an explicit path, for an SDK that isn't on PATH.
const COMPILER_VAR: &str = "ANCORIX_SLANGC";

// Writes freshly compiled SPIR-V back over `shaders/prebuilt/`, which is
// otherwise only ever read. Opt-in rather than automatic: a build script
// that writes inside its own crate makes `cargo publish` fail with
// "source directory was modified by build.rs".
const UPDATE_VAR: &str = "ANCORIX_SHADERS_UPDATE";

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is always set");
    let shaders = Path::new(&crate_dir).join("shaders");
    // the shader library lives in its own crate - see `ancorix_slang`
    let lib = ancorix_slang::include_path();
    let prebuilt = shaders.join("prebuilt");
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR is always set");
    let out_dir = Path::new(&out_dir);

    println!("cargo::rerun-if-env-changed={UPDATE_VAR}");
    println!("cargo::rerun-if-env-changed={COMPILER_VAR}");

    // every library module is a dependency of every shader that imports it
    for module in ancorix_slang::MODULES {
        println!(
            "cargo::rerun-if-changed={}",
            lib.join(format!("{module}.slang")).display()
        );
    }

    let compiler = compiler();
    let update = env::var_os(UPDATE_VAR).is_some();

    if compiler.is_none() {
        println!(
            "cargo::warning={COMPILER} not found - using the prebuilt SPIR-V in \
             shaders/prebuilt. Edits to the Slang sources will have no effect. Set \
             {COMPILER_VAR} to its path if it is installed but not on PATH."
        );
    }

    for (name, stages) in SHADERS {
        let source = shaders.join(format!("{name}.slang"));
        println!("cargo::rerun-if-changed={}", source.display());

        for &(entry, suffix) in stages {
            let artifact = format!("{name}.{suffix}.spv");
            let prebuilt = prebuilt.join(&artifact);
            let out = out_dir.join(&artifact);

            println!("cargo::rerun-if-changed={}", prebuilt.display());

            let Some(compiler) = &compiler else {
                copy(&prebuilt, &out);
                continue;
            };

            compile(compiler, &source, lib, entry, &out);

            if update {
                copy(&out, &prebuilt);
            } else if fs::read(&out).ok() != fs::read(&prebuilt).ok() {
                println!(
                    "cargo::warning={artifact} in shaders/prebuilt is stale - run \
                     `{UPDATE_VAR}=1 cargo build -p ancorix_render` and commit it, or anyone \
                     building without {COMPILER} gets the old shader."
                );
            }
        }
    }
}

fn compiler() -> Option<OsString> {
    let compiler = env::var_os(COMPILER_VAR).unwrap_or_else(|| COMPILER.into());

    Command::new(&compiler)
        .arg("-v")
        .output()
        .is_ok_and(|output| output.status.success())
        .then_some(compiler)
}

fn compile(compiler: &OsString, source: &Path, lib: &Path, entry: &str, out: &Path) {
    // slangc optimizes by default, so there is no `-Os` equivalent to pass;
    // `-I` is what makes `import ancorix` resolve.
    let output = Command::new(compiler)
        .arg("-target")
        .arg("spirv")
        .arg("-I")
        .arg(lib)
        .arg("-entry")
        .arg(entry)
        .arg("-o")
        .arg(out)
        .arg(source)
        .output()
        .unwrap_or_else(|e| {
            panic!(
                "failed to run {} on {}: {e}",
                compiler.display(),
                source.display()
            )
        });

    if !output.status.success() {
        panic!(
            "{} failed on {}:{}\n{}{}",
            compiler.display(),
            source.display(),
            entry,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
}

fn copy(from: &Path, to: &Path) {
    fs::copy(from, to)
        .unwrap_or_else(|e| panic!("failed to copy {} to {}: {e}", from.display(), to.display()));
}
