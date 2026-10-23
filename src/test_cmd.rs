// Licensed under the MIT License. See LICENSE.txt in the project root.

//! `crusty test` — a thin orchestration subcommand that delegates to Rust's
//! native `cargo test` harness (ADR-0006). It does NOT reimplement a harness:
//! it ensures this `crustyc` binary is discoverable on PATH (so a project's
//! `build.rs` can transpile `.crst` sources), then runs `cargo test`,
//! forwarding every extra argument straight through.

use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Run `crusty test [args…]`: delegate to `cargo test`, forwarding `args`.
/// Returns the process exit code to propagate.
pub fn run_test(args: &[String]) -> i32 {
    // A cargo project is required (we shell to `cargo test`). Give a clear
    // message rather than letting cargo fail obscurely from the wrong dir.
    if find_cargo_manifest().is_none() {
        eprintln!(
            "crusty test: no Cargo.toml found in this directory or any parent.\n\
             `crusty test` runs a cargo project's tests; run it inside a Crusty\n\
             cargo package (one whose build.rs transpiles its .crst sources)."
        );
        return 1;
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("test");
    cmd.args(args);

    // Make this very binary resolvable as `crustyc` on PATH, so the project's
    // build.rs finds the transpiler without a separate install. We prepend the
    // directory containing the current executable to PATH.
    if let Ok(exe) = env::current_exe() {
        if let Some(dir) = exe.parent() {
            let path = env::var_os("PATH").unwrap_or_default();
            let mut dirs = vec![dir.to_path_buf()];
            dirs.extend(env::split_paths(&path));
            if let Ok(joined) = env::join_paths(dirs) {
                cmd.env("PATH", joined);
            }
        }
    }

    match cmd.status() {
        Ok(status) => status.code().unwrap_or(1),
        Err(e) => {
            eprintln!("crusty test: failed to run `cargo test`: {}", e);
            1
        }
    }
}

/// Walk up from the current directory looking for a `Cargo.toml`.
fn find_cargo_manifest() -> Option<PathBuf> {
    let mut dir = env::current_dir().ok()?;
    loop {
        let candidate = dir.join("Cargo.toml");
        if candidate.is_file() {
            return Some(candidate);
        }
        if !dir.pop() {
            return None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_cargo_manifest_finds_this_crate() {
        // crustyc itself is a cargo project, so from its tree a manifest is
        // always found by the walk-up.
        assert!(
            find_cargo_manifest().is_some(),
            "should find crustyc's own Cargo.toml from the test working dir"
        );
    }
}
