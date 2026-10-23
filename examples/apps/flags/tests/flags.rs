// Cargo-native integration test for the flags app (in tests/, compiled as its
// own crate per Rust's harness). It runs the app binary and asserts the output
// shows the OR-combined flags and the AND membership tests — proving the
// transpiled Crusty program builds and behaves correctly end to end.

use std::process::Command;

#[test]
fn flags_prints_combined_membership() {
    // CARGO_BIN_EXE_<name> points at this package's built binary.
    let exe = env!("CARGO_BIN_EXE_crusty-app-flags");
    let output = Command::new(exe)
        .output()
        .expect("failed to run the flags binary");

    assert!(
        output.status.success(),
        "flags exited with failure: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout was not UTF-8");
    let line = stdout.trim();

    // A | B = 1 | 2 = 3; membership: A and B present, C absent.
    assert_eq!(
        line, "combined=3 has_a=true has_b=true has_c=false",
        "unexpected output: {:?}",
        line
    );
}
