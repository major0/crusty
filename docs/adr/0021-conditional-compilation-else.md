# ADR-0021: `#else` for conditional-compilation directives

## Status
accepted

## Context

Crusty's conditional-compilation directives are `#ifdef NAME … #endif` and
`#ifndef NAME … #endif`, lowering to `#[cfg(NAME)]` / `#[cfg(not(NAME))]` on each
contained item (ADR-0001 — the cfg predicate is passed through to rustc). There
is no `#else`: expressing "use A when FOO is set, B otherwise" today requires two
directive blocks with manually-inverted conditions —

```c
#ifdef FOO
    void impl() { /* A */ }
#endif
#ifndef FOO
    void impl() { /* B */ }
#endif
```

— which is verbose, repeats the condition, and is easy to get out of sync. `#else`
is the idiomatic C preprocessor form for this and the obvious missing piece.

The design question is lowering. Rust has no block-level preprocessor: `cfg` is an
attribute on individual items, not a surrounding block. So an `#else` branch cannot
lower to a Rust `else` — it lowers to a **second group of items gated by the
inverted condition**. `#ifdef FOO … #else … #endif` becomes `#[cfg(FOO)]` on the
first group and `#[cfg(not(FOO))]` on the second; `#ifndef FOO … #else …` inverts
both. This is exactly what the two-block workaround does by hand, so `#else` is
pure ergonomics over a lowering Crusty already performs — no new rustc surface, no
new semantics.

## Decision

We will add an optional `#else` branch to the conditional-compilation directive.

1. **Surface**: `#ifdef NAME <items> #else <items> #endif` and the `#ifndef`
   form likewise. `#else` is optional; its absence is the current behaviour
   unchanged.

2. **AST**: `ConditionalCompilation` gains `else_items: Vec<Item>` (empty when no
   `#else`). The existing `cfg` + `negated` fields are unchanged; one condition
   drives both branches via inversion, so no second predicate is stored.

3. **Lowering**: the primary items emit under the directive's predicate
   (`cfg(NAME)` for `#ifdef`, `cfg(not(NAME))` for `#ifndef`); the `else_items`
   emit under the **inverted** predicate (`cfg(not(NAME))` / `cfg(NAME)`
   respectively). Each item carries its own `#[cfg(...)]`, mirroring the existing
   per-item emission.

4. **Not in scope**: `#elif` / `#else if` (a chain of conditions). The two-branch
   `#else` covers the common case; a condition chain is a separate, larger
   decision (it needs a list of (predicate, items) arms) and earns its own ADR if
   wanted. `#if <expr>` general-expression conditions are likewise out of scope —
   the directive still keys off a single `defined`-style NAME.

## Consequences

**Easier.**
- The common "A if defined, else B" pattern is one block with the condition
  written once, instead of two blocks with a hand-inverted duplicate.
- Lowering reuses the existing per-item cfg emission and the `negated` inversion
  already in codegen — the `#else` group is the same code path with the predicate
  flipped.

**Harder / accepted costs.**
- `else_items` is a **new required field** on `ConditionalCompilation`, so every
  construction site must supply it (~5 sites across `ast.rs`, `parser.rs`,
  `codegen_unit_tests.rs`). Small and mechanical; surfaced by `cargo test
  --no-run`. A required field (not `Option`) is chosen because an empty `Vec` is
  the natural "no else" value and avoids an `Option` unwrap at every read.
- Crusty still does not evaluate the condition — a malformed or always-false
  predicate is rustc's concern (ADR-0001), exactly as today.

**Neutral.**
- No change to `#ifdef`/`#ifndef`/`#endif` syntax or to any other `#` directive.
  `#else` is only recognised between a matching `#if*` and its `#endif`.

## Verification

- [x] `ConditionalCompilation` has `else_items: Vec<Item>`; `#ifdef`/`#ifndef` without `#else` parse with `else_items` empty (existing behaviour unchanged).
- [x] `#ifdef FOO <a> #else <b> #endif` parses: `items` = [a], `else_items` = [b].
- [x] Codegen emits `#[cfg(FOO)]` on the `#ifdef` branch items and `#[cfg(not(FOO))]` on the `#else` items; the `#ifndef` form inverts both.
- [x] End-to-end under `rustc -D unused_parens`: a file selecting between two function definitions by a `--cfg` flag compiles and runs the correct branch with and without the flag.
- [x] A stray `#else` with no open `#if*`, and an `#if*` with two `#else` branches, are parse errors.
- [x] Full suite green (`cargo test`), clippy clean.
- [x] `docs/syntax/` documents the `#else` form and the two-cfg-group lowering.

## Alternatives considered

- **Add `#elif` / a condition chain now.** Rejected for this ADR: it needs a
  different AST (a list of (predicate, items) arms, not one condition + inversion)
  and the two-branch `#else` already removes the duplicated-condition pain. A
  chain can supersede this shape in its own ADR if a real need appears.
- **Lower `#else` to a Rust `else` / `cfg!` macro in function bodies.** Rejected:
  the directive is item-level (it gates whole definitions), and Rust gates items
  with `#[cfg]`, not `else`. The two-group lowering is the only correct item-level
  form.
- **Store a second explicit predicate for the else branch.** Rejected: the else
  condition is always the inverse of the if condition, so storing it invites the
  two to diverge. One condition + inversion is smart-data (the data cannot
  represent an inconsistent state).
