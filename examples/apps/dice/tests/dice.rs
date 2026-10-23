// Cargo-native integration test for the dice app (in tests/, compiled as its
// own crate per Rust's harness). It runs the app binary and asserts the output
// is a valid die roll — proving the transpiled Crusty program links `rand`,
// builds, and behaves correctly end to end.

use std::process::Command;

#[test]
fn dice_prints_a_valid_roll() {
    // CARGO_BIN_EXE_<name> points at this package's built binary.
    let exe = env!("CARGO_BIN_EXE_crusty-app-dice");
    let output = Command::new(exe)
        .output()
        .expect("failed to run the dice binary");

    assert!(
        output.status.success(),
        "dice exited with failure: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout was not UTF-8");
    let line = stdout.trim();

    // Expected shape: "You rolled a N" with N in 1..=6.
    let prefix = "You rolled a ";
    let n: u32 = line
        .strip_prefix(prefix)
        .and_then(|s| s.parse().ok())
        .unwrap_or_else(|| panic!("unexpected output: {:?}", line));

    assert!((1..=6).contains(&n), "die roll out of range: {}", n);
}
