// Cargo-native integration test for the config app (in tests/, compiled as its
// own crate per Rust's harness). It runs the app binary and asserts the output
// matches the parsed TOML — proving the transpiled Crusty program links `toml`
// and `serde`, derives Deserialize, builds, and behaves correctly end to end.

use std::process::Command;

#[test]
fn config_prints_parsed_fields() {
    // CARGO_BIN_EXE_<name> points at this package's built binary.
    let exe = env!("CARGO_BIN_EXE_crusty-app-config");
    let output = Command::new(exe)
        .output()
        .expect("failed to run the config binary");

    assert!(
        output.status.success(),
        "config exited with failure: {:?}",
        output.status
    );

    let stdout = String::from_utf8(output.stdout).expect("stdout was not UTF-8");
    let line = stdout.trim();

    // The app parses: name="demo", server.host="localhost", server.port=8080
    // and prints "<name> -> <host>:<port>".
    assert_eq!(line, "demo -> localhost:8080", "unexpected output: {:?}", line);
}
