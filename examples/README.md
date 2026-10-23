# Crusty Example Applications (Integration Tests)

Real, runnable Crusty programs that link actual cargo crates. They are the
end-to-end proof that Crusty transpiles to Rust and coexists with the Rust
ecosystem, and they double as integration tests.

See `docs/adr/0005-example-apps-integration-tests.md` for the design.

## Apps

| App | Links | Demonstrates |
|-----|-------|--------------|
| `dice/` | [`rand`](https://crates.io/crates/rand) | `#import`, `@Type.method()`, method calls, C-style typed bindings |

## Running locally

Build `crustyc`, put it on PATH, then build + test every app:

```sh
cargo build --release --bin crustyc
export PATH="$PWD/target/release:$PATH"
.github/scripts/run-integration-apps.sh
```

Or test a single app directly:

```sh
cd examples/apps/dice
cargo test      # runs the native tests/ harness (asserts the app's output)
cargo run       # or just run it
```

Each app asserts its behaviour with a Cargo-native integration test under
`tests/` (compiled as its own crate), which runs the built binary and checks
its output — so a regression is a real test failure, not just a non-zero exit.

## CI

These run on **every pull request** via `.github/workflows/integration-tests.yml`
(wired into `pull-request-default.yml`). They are deliberately **NOT** part of
pre-commit — they compile real crates, which is too slow for a commit hook.

## The `__rust__` gap metric

Every place an app must drop to the `__rust__` escape hatch marks a Crusty
expressiveness gap. Count them across the suite:

```sh
scripts/count_rust_escapes
```

The count is reported (non-gating) in CI. Driving it toward zero is how the
example suite guides grammar work: a new app that needs `__rust__` names a
concrete feature to consider adding.

## Adding an app

1. `examples/apps/<name>/` with its own `Cargo.toml` (a real crate in
   `[dependencies]`, `crustyc` as a `build-dependencies` path dep).
2. A `build.rs` that transpiles `src/*.crst` to `OUT_DIR` (copy `dice/build.rs`).
3. `src/main.rs` = `include!(concat!(env!("OUT_DIR"), "/main.rs"));`.
4. `src/main.crst` = the app, in Crusty, calling into the crate.
5. Commit `Cargo.lock` (pinned deps); `target/` is gitignored.
