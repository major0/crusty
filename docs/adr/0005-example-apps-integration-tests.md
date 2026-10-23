# ADR-0005: Example applications as integration tests

## Status
accepted

## Context
Crusty's goal is to transpile to Rust and coexist with the Rust ecosystem —
Crusty code should link against ordinary cargo crates. We have no end-to-end
proof of that: the existing `example/` crate transpiles a bag of feature
snippets to `OUT_DIR` but only compiles `src/main.rs`, links no third-party
crate, and is not run by CI. So nothing guarantees a Crusty program actually
builds, links a real dependency, and runs.

Two things are missing:

1. **Executable proof of cross-compatibility.** Real, small applications,
   written in Crusty, that depend on a well-known cargo crate, transpile, build
   and run — exercised by CI so a regression fails the build.
2. **A gap signal.** Every place an example must drop to the `__rust__` escape
   hatch marks a Crusty expressiveness gap. Counting `__rust__` occurrences
   across the examples turns "the grammar has gaps" into a tracked number we can
   drive down.

## Decision
Add a suite of **example applications**, each its own cargo package under
`examples/apps/<name>/`, that:

- is written in Crusty (`.crst`) and transpiled by the existing `build.rs`
  pattern (hardened to use the workspace-built `crustyc`, not a PATH lookup);
- declares at least one real third-party `[dependencies]` crate and actually
  calls into it;
- has a `main` that runs and asserts a result (so `cargo run` is a smoke test),
  and/or `#[test]` functions;
- is built and run by CI as an integration gate.

A small checker script (`scripts/count_rust_escapes`) greps the example `.crst`
sources for `__rust__` blocks and reports the count per app and the total. The
number is recorded; a reviewer watches whether a new example needed the escape
hatch, and each such use names a grammar gap to consider closing.

Start with ONE app (incremental, per house style): a tiny CLI that links a
widely-used, dependency-light crate, proving the harness end to end. Widen the
suite only once the first app builds and runs green in CI.

Structure:

```
examples/
  apps/
    <app-name>/
      Cargo.toml        # [dependencies] = a real crate; build-dep = crustyc
      build.rs          # transpile src/*.crst -> OUT_DIR (shared pattern)
      src/*.crst        # the app, in Crusty
scripts/
  count_rust_escapes    # __rust__ frequency metric over examples/apps/**
```

The legacy `example/` feature-snippet crate is left as-is for now; this ADR adds
the app suite beside it rather than reworking it.

## Consequences
- Easier: a real regression (grammar, codegen, or cargo integration) fails a
  concrete `cargo run`, not just a unit test; the `__rust__` count gives a
  measurable expressiveness target.
- Cost accepted: CI gains a slower job (it compiles real crates). Keep the first
  app's dependency light and the suite small so the cost stays bounded.
- Cost accepted: each app pins a third-party crate version, which must be
  maintained; mitigated by choosing stable, low-churn crates and few of them.
- A `__rust__` in an example is not a failure — it is a logged gap. The metric
  informs grammar work; it does not block the build.

## Verification
- [ ] `examples/apps/<name>/` exists, declares a real crate dependency, and its
      `.crst` source calls into that crate.
- [ ] From a clean checkout, building that app transpiles its `.crst`, compiles,
      and `cargo run` produces the expected output / its `#[test]`s pass.
- [ ] The app's `build.rs` resolves `crustyc` from the workspace build, not an
      ambient PATH install, so CI needs no separate install step.
- [ ] `scripts/count_rust_escapes` prints a per-app and total `__rust__` count
      and exits 0.
- [ ] CI builds and runs the example app(s) as an integration gate.
