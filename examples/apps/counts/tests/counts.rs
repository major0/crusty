// Cargo-native integration test for the counts app (in tests/, compiled as its
// own crate per Rust's harness). It runs the app binary and asserts the output
// matches the insertion-ordered IndexMap it builds — proving the transpiled
// Crusty program links `indexmap`, builds, and behaves correctly end to end.

use std::process::Command;

#[test]
fn counts_prints_entries_in_insertion_order() {
    // CARGO_BIN_EXE_<name> points at this package's built binary.
    let exe = env!("CARGO_BIN_EXE_crusty-app-counts");
    let output = Command::new(exe)
        .output()
        .expect("failed to run the counts binary");

    assert!(
        output.status.success(),
        "counts exited with failure: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout was not UTF-8");

    // IndexMap preserves insertion order, so the three entries print in the
    // order inserted, followed by the count of kinds.
    let expected = "apples = 3\npears = 1\nplums = 4\ntotal kinds = 3\n";
    assert_eq!(stdout, expected, "unexpected output: {:?}", stdout);
}
