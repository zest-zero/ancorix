pub mod sections;

use std::path::{Path, PathBuf};

/// Looks for `*.project.json5` / `*.project.json` in the crate whose
/// `build.rs` calls this function, and compiles each present section
/// into a separate `.axb` file under `.ancorix/`.
///
/// If no project file is found, this is a no-op - the project.json
/// pipeline is opt-in, not required to build.
///
/// # Panics
///
/// Panics if a project file is found but is not valid JSON/JSON5.
pub fn build() {
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let root = Path::new(&manifest);

    let Some(project_file) = find_project_file(root) else {
        return;
    };
    println!("cargo:rerun-if-changed={}", project_file.display());

    let project = parse_project_file(&project_file);

    let out = root.join(".ancorix");
    std::fs::create_dir_all(&out).expect("failed to create .ancorix/");

    compile_section(
        &project,
        "input",
        &out.join("input.axb"),
        sections::input::compile,
    );
    compile_section(
        &project,
        "window",
        &out.join("window.axb"),
        sections::window::compile,
    );
    compile_section(
        &project,
        "assets",
        &out.join("assets.axb"),
        sections::assets::compile,
    );
}

/// Looks for `*.project.json5` / `*.project.json` directly inside `root`.
///
/// Shared by [`build()`] (searching `CARGO_MANIFEST_DIR` at build time) and
/// `ancorix_window::Runner` (searching the current directory at runtime).
pub fn find_project_file(root: &Path) -> Option<PathBuf> {
    // prefer .json5 for comments support, fall back to .json
    std::fs::read_dir(root)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                return false;
            };
            let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
                return false;
            };
            stem.ends_with(".project") && (ext == "json5" || ext == "json")
        })
}

/// Reads and parses a `*.project.json5` / `*.project.json` file found by
/// [`find_project_file`].
///
/// # Panics
///
/// Panics if `path` can't be read, or isn't valid JSON/JSON5.
pub fn parse_project_file(path: &Path) -> serde_json::Value {
    let src = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));

    // json5 is a superset of json - accepts both .json and .json5
    json5::from_str(&src).unwrap_or_else(|e| panic!("failed to parse {}: {e}", path.display()))
}

fn compile_section(
    project: &serde_json::Value,
    key: &str,
    out: &Path,
    compile: fn(&serde_json::Value) -> Vec<u8>,
) {
    // section is optional - if missing, write empty axb with defaults
    let section = project.get(key).unwrap_or(&serde_json::Value::Null);

    // skip null sections entirely
    if section.is_null() && key == "assets" {
        return;
    }

    let empty = serde_json::Value::Object(serde_json::Map::new());
    let bytes = compile(if section.is_null() { &empty } else { section });

    std::fs::write(out, &bytes)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", out.display()));

    eprintln!("[ancorix] compiled {} -> {}", key, out.display());
}
