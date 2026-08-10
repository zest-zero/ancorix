//! Runs `tests/math.slang` on the CPU.
//!
//! Slang compiles the same module to a host executable that it compiles to
//! SPIR-V, so the shader library's maths can be tested without a GPU, a
//! window, or a screenshot to squint at.

use std::path::Path;
use std::process::Command;

#[test]
fn shader_maths_holds_up() {
    let Some(compiler) = compiler() else {
        eprintln!("slangc not found - skipping (set ANCORIX_SLANGC to its path)");
        return;
    };

    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/math.slang");
    let exe = Path::new(env!("CARGO_TARGET_TMPDIR")).join("slang_math_test");

    let built = Command::new(&compiler)
        .arg("-target")
        .arg("exe")
        .arg("-I")
        .arg(ancorix_slang::include_path())
        .arg("-o")
        .arg(&exe)
        .arg(&source)
        .output()
        .expect("failed to run slangc");

    assert!(
        built.status.success(),
        "slangc failed:\n{}{}",
        String::from_utf8_lossy(&built.stdout),
        String::from_utf8_lossy(&built.stderr),
    );

    let run = Command::new(&exe).output().expect("failed to run the test");
    let report = String::from_utf8_lossy(&run.stdout);

    assert!(run.status.success(), "{report}");
    // a run that asserts nothing would also "pass" - make sure it ran
    assert!(report.contains("all passed"), "{report}");
}

fn compiler() -> Option<String> {
    let compiler = std::env::var("ANCORIX_SLANGC").unwrap_or_else(|_| "slangc".into());

    Command::new(&compiler)
        .arg("-v")
        .output()
        .is_ok_and(|out| out.status.success())
        .then_some(compiler)
}
