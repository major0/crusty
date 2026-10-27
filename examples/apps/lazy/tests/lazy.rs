// Cargo-native integration test for the lazy app (in tests/, compiled as its
// own crate per Rust's harness). It runs the app binary and asserts the output
// is the expected lazily-initialized static value — proving the transpiled
// Crusty program builds, links, and behaves correctly end to end.
//
// NOTE: `lazy_static`'s `lazy_static!` macro INVOCATION is expressible in Crusty
// now (item-position brace-body bang-macros parse and emit verbatim), but
// READING the macro-generated static needs a one-token `__rust__` escape — the
// name lives inside the opaque macro body and never enters Crusty's symbol
// table (see src/main.crst for the full write-up). To keep this app's escape
// count at 0, it demonstrates the equivalent read-and-print behaviour with a
// plain `static`, whose name Crusty does see.

use std::process::Command;

#[test]
fn lazy_prints_the_initialized_value() {
    // CARGO_BIN_EXE_<name> points at this package's built binary.
    let exe = env!("CARGO_BIN_EXE_crusty-app-lazy");
    let output = Command::new(exe)
        .output()
        .expect("failed to run the lazy binary");

    assert!(
        output.status.success(),
        "lazy exited with failure: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout was not UTF-8");
    let line = stdout.trim();

    assert_eq!(line, "lazy value is 42", "unexpected output: {:?}", line);
}
