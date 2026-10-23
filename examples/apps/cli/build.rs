// Transpile this app's Crusty source to Rust before compilation.
// crustyc is resolved from PATH; the integration-test workflow builds and
// exposes it before building the app (see .github/workflows/integration-tests.yml).
use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");

    let src = Path::new(&manifest_dir).join("src").join("main.crst");
    let out = Path::new(&out_dir).join("main.rs");

    println!("cargo:rerun-if-changed={}", src.display());
    println!("cargo:rerun-if-changed=build.rs");

    let status = Command::new("crustyc")
        .args([
            src.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
            "--emit=rust",
        ])
        .status()
        .expect("failed to execute crustyc (is it on PATH?)");

    if !status.success() {
        panic!("crustyc failed to transpile {}", src.display());
    }
}