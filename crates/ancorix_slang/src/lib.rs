//! Ancorix's shader library for the Slang language, and the one thing a Rust
//! build script needs to use it: where the modules live.
//!
//! Slang has no package manager, so `import ancorix;` only resolves if the
//! compiler is pointed at the directory holding these modules. That path is
//! inside this crate's own source, wherever cargo happened to unpack it -
//! which is exactly what [`include_path()`] returns.
//!
//! `-profile spirv_1_3` is not optional: slangc emits SPIR-V 1.5 by default,
//! and the Vulkan 1.1 instance ancorix creates rejects anything newer than 1.3.
//!
//! ```no_run
//! // build.rs of a project with shaders of its own
//! let status = std::process::Command::new("slangc")
//!     .arg("-target").arg("spirv")
//!     .arg("-profile").arg("spirv_1_3")
//!     .arg("-I").arg(ancorix_slang::include_path())
//!     .arg("-entry").arg("pixel").arg("-stage").arg("fragment")
//!     .arg("-o").arg("water.frag.spv")
//!     .arg("shaders/water.slang")
//!     .status();
//! ```

use std::path::Path;

/// The directory to pass to `slangc -I`, so that `import ancorix;` resolves.
///
/// # Examples
///
/// ```
/// assert!(ancorix_slang::include_path().join("ancorix.slang").is_file());
/// ```
pub fn include_path() -> &'static Path {
    Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/slang"))
}

/// The modules [`include_path()`] contains, in the order they build on each
/// other. Useful for a build script that wants to rerun when any changes.
///
/// # Examples
///
/// ```
/// for module in ancorix_slang::MODULES {
///     let path = ancorix_slang::include_path().join(format!("{module}.slang"));
///     assert!(path.is_file(), "{module} is missing");
/// }
/// ```
pub const MODULES: [&str; 10] = [
    "surface", "shape", "shapes", "color", "ease", "noise", "pattern", "gradient", "distort",
    "ancorix",
];
