// Cargo-native integration test for the cli app (in tests/, compiled as its own
// crate per Rust's harness). It runs the app binary and asserts on its stdout —
// proving the transpiled Crusty program links `clap`, builds a Command via the
// builder API, parses arguments, and behaves correctly end to end.

use std::process::Command;

#[test]
fn cli_prints_the_expected_greeting() {
    // CARGO_BIN_EXE_<name> points at this package's built binary.
    let exe = env!("CARGO_BIN_EXE_crusty-app-cli");
    let output = Command::new(exe)
        .output()
        .expect("failed to run the cli binary");

    assert!(
        output.status.success(),
        "cli exited with failure: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout was not UTF-8");
    let line = stdout.trim();

    // The app parses a fixed argv (["greet", "Crusty"]) via clap's builder, so
    // the positional "name" argument resolves to "Crusty".
    assert_eq!(line, "Hello, Crusty!", "unexpected output: {:?}", line);
}
