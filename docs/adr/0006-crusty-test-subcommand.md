# ADR-0006: `crusty test` — delegating to Rust's test harness

## Status
accepted

## Context
Crusty authors need to test Crusty code. Rust already has an excellent native
test harness that distinguishes unit tests (`#[test]` fns inside the crate) from
integration tests (each file in `tests/` compiled as its own crate against the
public API), runnable with `cargo test`. Crusty transpiles to Rust, so the
harness is already reachable — what is missing is orchestration: today an author
must transpile `.crst` by hand and then invoke cargo themselves.

What already works (verified): a Crusty function annotated `#[test]` transpiles
to a Rust `#[test]` fn (`#[test] void test_x() {…}` -> `#[test] pub fn test_x()`),
because attributes pass through. So Crusty can already *express* tests; it cannot
yet *run* them with one command.

The tension: do we build a Crusty-native test runner, or delegate to cargo?
Rebuilding a harness (discovery, filtering, parallelism, output formatting,
`#[should_panic]`, `--nocapture`, …) would be a large, perpetually-behind
reimplementation. "Worse is better" says delegate.

## Decision
`crusty test` will be a thin orchestration subcommand that DELEGATES to Rust's
native harness, not a new harness:

1. In a cargo project (the normal case — Crusty programs are cargo packages
   whose `build.rs` transpiles `.crst`, as the example apps do), `crusty test
   [args…]` transpiles the project's Crusty sources via the existing `build.rs`
   path and then runs `cargo test [args…]`, forwarding filters and flags
   (`--lib`, `--test <name>`, `-- --nocapture`, a test-name filter) straight
   through. The unit/integration split, parallelism, and output are Cargo's.
2. Crusty unit tests are `#[test]`-annotated Crusty functions in `src/*.crst`;
   Crusty integration tests are `.crst` files under `tests/` (transpiled to Rust
   test crates by the project's build). `crusty test` does not invent a new test
   shape — it reuses Cargo's locations and semantics.
3. The subcommand resolves `crustyc` as itself (same binary) for any transpile
   step it must drive, so no separate PATH install is required.

This also subsumes the example-apps runner: `scripts/run-integration-apps.sh`
can become `crusty test` per app once the subcommand exists.

Out of scope for this ADR: a standalone "test a single .crst with no cargo
project" mode, and Crusty-native assertion sugar (authors use `assert!` /
`assert_eq!` via the existing `name!(...)` macro-call syntax). Both can be
layered later without changing this decision.

## Consequences
- Easier: one command to test Crusty code; authors get Cargo's full harness
  (filtering, `#[should_panic]`, parallel runs, `--nocapture`) for free.
- Cost accepted: `crusty test` requires a cargo project context (it shells to
  `cargo test`). A bare `.crst` with no project is not testable by this command
  in its first form — acceptable, since real programs are cargo packages.
- The transpiler binary grows a subcommand surface (`crusty <verb>`), a change
  from today's single-shot `crustyc <input>` CLI. The existing single-shot form
  is preserved; `test` is additive.
- Delegating means Crusty never diverges from Rust's test semantics, which is the
  point of a transpiler that coexists with cargo.

## Verification
- [ ] `crusty test` run in a Crusty cargo project transpiles its `.crst` sources
      and invokes `cargo test`, exiting non-zero iff a test fails.
- [ ] A `#[test]`-annotated Crusty function in `src/*.crst` is discovered and run
      as a unit test.
- [ ] A `.crst` under `tests/` runs as a Cargo integration test (its own crate).
- [ ] Test filters/flags pass through: `crusty test <name>` and
      `crusty test -- --nocapture` behave as the cargo equivalents.
- [ ] The example apps can be exercised by `crusty test` in place of the shell
      runner.
