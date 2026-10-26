# ADR-0022: the C `#ifdef` directive family — `#elif` / `#elifdef` / `#elifndef` condition chains

## Status
proposed

## Context

Conditional compilation currently supports a two-branch form (ADR-0021):

```c
#ifdef FOO
    void impl() { /* A */ }
#else
    void impl() { /* B */ }
#endif
```

lowering to per-item `#[cfg(FOO)]` on the first group and `#[cfg(not(FOO))]` on
the `#else` group. The AST models this with ONE condition (`cfg` + `negated`) and
two item lists (`items`, `else_items`), the else predicate being the inverse of
the if predicate.

This cannot express an **N-way** selection — "A if FOO, else B if BAR, else C" —
without nesting a second `#ifdef` inside the `#else`:

```c
#ifdef FOO
    void impl() { /* A */ }
#else
    #ifndef BAR            // awkward: must restate / invert by hand
    #else
        void impl() { /* B */ }
    #endif
#endif
```

`#elif NAME` is the idiomatic C-preprocessor form for a condition chain, and
ADR-0021 explicitly deferred it, naming the exact reason: a chain "needs a
different AST — a list of (predicate, items) arms, not one condition +
inversion." This ADR makes that reshape.

Beyond the chain, Crusty should adopt the **standard C `#ifdef` directive
family** so a C programmer's conditional blocks transpile unchanged: the
`defined`-shorthand directives `#ifdef` / `#ifndef` / `#elif` / `#elifdef` /
`#elifndef` / `#else` / `#endif`. We deliberately do NOT adopt `#if <expr>` /
`#elif <expr>` general constant-expression conditions: C evaluates those at
preprocessing time, whereas Crusty never evaluates a condition — it passes a
single `defined`-style NAME through to rustc's `cfg` (ADR-0001). Arithmetic and
version tests have no `cfg` equivalent and stay in `__rust__`. The family we
adopt is exactly the subset that keys off whether a NAME is defined, which maps
one-to-one onto `cfg(NAME)` / `cfg(not(NAME))`.

The design question is again lowering. Rust has no block preprocessor and no
`cfg`-level `else if`: `cfg` is an attribute on an individual item. `#elif` is
**else-if** — an arm is selected only when its own condition holds AND every
earlier arm's condition failed. So arm *k* in the chain lowers to the
conjunction of its own predicate with the negation of all predicates before it:

```c
#ifdef A        a()
#elif B         b()      // #elif NAME ≡ #elifdef NAME (positive defined test)
#elifndef C     c()      // negated arm: selected when C is NOT defined
#else           d()
#endif
```

->

```rust
#[cfg(A)]                           fn a() { … }
#[cfg(all(not(A), B))]              fn b() { … }
#[cfg(all(not(A), not(B), not(C)))] fn c() { … }
#[cfg(all(not(A), not(B), C))]      fn d() { … }
```

This cascade of negations is what makes a *sequential* chain correct as a set of
*independent* per-item `#[cfg]` attributes — without it, overlapping conditions
(both `A` and `B` set) would activate two arms, which the preprocessor semantics
forbid. The conjunction is exactly `cfg(all(…))`, which rustc already supports;
no new rustc surface, no new semantics, and Crusty still never evaluates the
conditions (ADR-0001).

## Decision

We will adopt the standard C `#ifdef` directive family — `#ifdef` / `#ifndef`
openers, `#elif` / `#elifdef` / `#elifndef` chain arms, `#else`, `#endif` — and
reshape `ConditionalCompilation` from a two-branch record into an ordered list of
condition arms plus an optional final `#else`.

1. **Surface**: `#ifdef NAME` / `#ifndef NAME` opens the chain; zero or more
   chain arms follow, each spelled `#elif NAME`, `#elifdef NAME`, or
   `#elifndef NAME`; an optional `#else` closes with an unconditional arm; and
   `#endif` terminates. Every arm keys off a single `defined`-style NAME:
   - `#elif NAME` and `#elifdef NAME` are equivalent positive defined-tests
     (`#elif` is accepted as a C-programmer-friendly shorthand even though
     standard C spells the defined-shorthand `#elifdef`; Crusty has no
     `#elif <expr>` form to collide with).
   - `#elifndef NAME` is the negated arm (selected when NAME is not defined).

   `#if <expr>` / `#elif <expr>` general constant-expression conditions are not
   part of the surface (see *Not in scope*).

2. **AST**: replace the `{ cfg, negated, items, else_items }` record with

   ```rust
   pub struct ConditionalCompilation {
       pub arms: Vec<CondArm>,       // >= 1; arms[0] is the #if* opener
       pub else_items: Vec<Item>,    // the #else arm, empty when absent
   }
   pub struct CondArm {
       pub cfg: Ident,               // the NAME this arm tests
       pub negated: bool,            // #ifndef opener / #elifndef arm
       pub items: Vec<Item>,
   }
   ```

   The two-branch form is the special case `arms.len() == 1`. `negated` records
   each arm's polarity: the `#ifdef`/`#ifndef` distinction on `arms[0]`, and the
   `#elif`/`#elifdef` (positive) vs `#elifndef` (negated) distinction on each
   chain arm.

3. **Lowering**: arm *k* emits each of its items under
   `cfg(all(not(c_0), …, not(c_{k-1}), c_k))`, where `c_i` is arm *i*'s predicate
   (`NAME` or `not(NAME)` per its `negated` flag). The `#else` items emit under
   `cfg(all(not(c_0), …, not(c_{n-1})))` — the negation of every arm. A single
   arm with no `#else` collapses to the current `#[cfg(NAME)]` emission (an
   `all(c_0)` of one term is simplified to `c_0`), so existing output is
   byte-identical.

4. **Not in scope**: `#if <expr>` / `#elif <expr>` general constant-expression
   conditions. C evaluates those at preprocessing time; Crusty never evaluates a
   condition (ADR-0001) — it keys off a single `defined`-style NAME and passes it
   to `cfg`. Arithmetic/version tests (`#if VERSION > 2`) have no `cfg` form and
   stay in `__rust__`. `defined`-expression combinators (`#if defined(A) &&
   !defined(B)` → `cfg(all(A, not(B)))`) are a plausible future extension — a
   richer per-arm predicate over the same cascade — but a single NAME per arm is
   the committed surface here. `cfg` feature-flag values (`feature = "x"`) are
   out of scope.

This supersedes ADR-0021's AST shape (the two-branch record); ADR-0021's
*decision* about `#else` lowering stands and is subsumed here as the
`arms.len() == 1` + `else_items` case.

## Consequences

**Easier.**
- N-way compile-time selection is a flat chain with each condition written once,
  instead of hand-nested `#ifdef`/`#else` blocks with inverted duplicates.
- Lowering is one cascade-negation rule that generalises both the single-arm and
  `#else` cases; the two-branch behaviour falls out as `arms.len() == 1`.

**Harder / accepted costs.**
- `ConditionalCompilation` is **reshaped**, not extended — `cfg`/`negated` move
  onto a new `CondArm` and the top level gains `arms`. This ripples to every
  construction and destructuring site (~5 files: `ast.rs`, `parser.rs`,
  `codegen.rs`, `semantic.rs`, `codegen_unit_tests.rs`), and the parser tests
  that assert on `cc.negated` / `cc.items` now read `cc.arms[0].*`. Bounded and
  mechanical; `cargo test --no-run` enumerates every site. This is a one-way
  door (an accepted ADR's shape is not re-litigated), which is why it is an ADR.
- The semantic analyzer must analyze every arm's items; mutually-exclusive arms
  defining the same name are not real duplicates, so each arm after the first
  (and the `#else`) analyzes in a throwaway child scope — generalising the
  ADR-0021 `#else`-scope device to N arms.
- Crusty still does not evaluate conditions; an always-true pair of overlapping
  NAMEs is rustc's concern, and the cascade negation makes the *emitted* cfg
  unambiguous regardless (at most one arm's `all(…)` can hold).

**Neutral.**
- No change to `#ifdef`/`#ifndef`/`#endif`/`#else` surface syntax or to any other
  `#` directive. The chain arms (`#elif`/`#elifdef`/`#elifndef`) are only
  recognised between a matching `#if*` opener and its `#endif`, before any
  `#else`.
- Single-arm and `#else`-only blocks emit byte-identical Rust to today.

## Verification

- [ ] `ConditionalCompilation` is `{ arms: Vec<CondArm>, else_items: Vec<Item> }`; `CondArm` is `{ cfg, negated, items }`.
- [ ] `#ifdef A <a> #elif B <b> #elif C <c> #else <d> #endif` parses to 3 arms + else_items; `arms[0].negated == false`, elif arms `negated == false`.
- [ ] `#ifndef A <a> #elif B <b> #endif` parses: `arms[0].negated == true`, `arms[1].negated == false`, `else_items` empty.
- [ ] `#elif NAME` and `#elifdef NAME` parse to the same arm (`negated == false`); `#elifndef NAME` parses to a negated arm (`negated == true`).
- [ ] A single `#ifdef A <a> #endif` still emits exactly `#[cfg(A)]` on each item (no `all(…)` wrapper) — byte-identical to pre-ADR output.
- [ ] Codegen emits the cascade: arm k under `#[cfg(all(not(c_0), …, c_k))]`, `#else` under `#[cfg(all(not(c_0), …, not(c_{n-1})))]`, where a `#elifndef` arm contributes `not(NAME)` as its own term `c_k` and `NAME` to later arms' negations.
- [ ] End-to-end under `rustc -D unused_parens`: a chain mixing `#elif`/`#elifndef` compiles and runs the correct single arm under each relevant `--cfg` combination, with no combination activating two arms.
- [ ] Same NAME-defining item in multiple arms is not a `DuplicateDefinition`.
- [ ] A stray chain arm (`#elif`/`#elifdef`/`#elifndef`) with no open `#if*`, any chain arm after `#else`, and `#else` after `#else` are parse errors.
- [ ] Full suite green (`cargo test`), clippy clean.
- [ ] `docs/syntax/` documents the full `#ifdef` family (`#elif`/`#elifdef`/`#elifndef` chain + cascade-negation lowering); ADR-0021's status reflects the superseded AST shape.

## Alternatives considered

- **Keep the two-branch AST and nest for N-way.** Rejected: that is the status
  quo whose awkwardness motivates this ADR — hand-nested `#else`/`#ifdef` with
  restated conditions is exactly what a chain removes. ADR-0021 already named the
  reshape as the correct path.
- **Lower each arm to a bare `#[cfg(c_k)]` without the prior-negations.**
  Rejected: that is a set of *independent* conditions, not an *else-if* chain —
  if two NAMEs are both set, two arms activate and rustc sees duplicate
  definitions. The cascade negation is what encodes "only if no earlier arm
  matched."
- **Store a precomputed `all(…)` predicate string per arm in the AST.** Rejected:
  that bakes codegen output into the AST (smart-data violated) and duplicates
  state the arm list already determines. The arm list is the single source of
  truth; the negation cascade is a pure function of it, computed at emission.
- **Adopt `#if <expr>` / `#elif <expr>` general constant-expression conditions
  (full C semantics).** Rejected: C evaluates these at preprocessing time, which
  would make Crusty a condition evaluator with opinions about which code exists —
  directly against ADR-0001, where rustc owns that decision via `cfg`. Arithmetic
  (`#if VERSION > 2`) has no `cfg` equivalent at all. The `#ifdef` family keys off
  a single `defined`-style NAME, which maps exactly onto `cfg`; `__rust__` covers
  the rest.
- **Require the C23 `#elifdef` spelling and drop the bare `#elif NAME`
  shorthand.** Rejected: standard C spells the positive defined-shorthand
  `#elifdef`, but Crusty has no `#elif <expr>` form for `#elif NAME` to collide
  with, so accepting `#elif NAME` ≡ `#elifdef NAME` is strictly more forgiving to
  a C programmer at zero ambiguity cost. Both spellings produce the same positive
  arm.
