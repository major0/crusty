# ADR-0014: Nested functions hoist; anonymous functions capture (PS-07)

## Status
accepted

Supersedes ADR-0013.

## Context
ADR-0013 (accepted) specified PS-07 as two workstreams: function-pointer
*types* (`fn(T) -> R`, Workstream A) and anonymous functions
(`[ret] __func__(typed-params) { body }`, Workstream B), with **named nested
functions** lowered — as the code does today — to a `let name = |…| { … }`
closure binding.

Implementing against that revealed a semantic conflict that ADR-0013 did not
resolve, surfaced in review:

- A Crusty **named nested function** currently lowers to a `let`-closure
  (`src/codegen.rs`, the `Statement::NestedFunction` arm). A Rust `let`-closure
  is **not hoisted** (unusable before its own `let`) and **cannot recurse** (the
  binding is not in scope inside its own initializer, and a closure cannot name
  itself).
- Crusty's stated goal (ADR-0001 lineage) is a **C-like surface over Rust's
  native features**. C nested functions (and Rust nested `fn` items) are
  **hoisted** — callable before their textual definition — and support
  **recursion**. The `let`-closure lowering silently denies both.
- In Rust, **hoisting and capture are mutually exclusive**: a nested `fn` item
  is hoisted and recursive but **cannot capture** enclosing locals; a closure
  captures but is neither hoisted nor recursive. One Crusty surface form cannot
  have both from one Rust construct.

ADR-0013 put capture on the nested-function form (via the closure lowering) and
therefore gave up hoisting and recursion. Review chose the opposite split: make
each Crusty surface map to the Rust construct whose semantics match it.

## Decision
Map the two Crusty surface forms **one-to-one** onto the two Rust constructs,
each taking exactly that construct's semantics. This changes two of ADR-0013's
decisions; everything else in ADR-0013 (Workstream A; the `__func__` surface,
typing, and non-goals) carries forward unchanged.

| Crusty surface | Rust target | Hoisted | Captures | Recursive |
|---|---|---|---|---|
| named nested function | nested `fn` item | **yes** | **no** | **yes** |
| `__func__(typed-params){body}` | `let = \|…\| {…}` closure | no | **yes** | no |

### Change 1 — named nested functions lower to Rust `fn` items
- `Statement::NestedFunction` lowers to a Rust **`fn name(params) -> ret { body }`
  item inside the enclosing function body**, NOT a `let`-closure. It is then
  hoisted (callable anywhere in the enclosing body, before or after its
  definition) and may recurse and mutually recurse with sibling nested `fn`s.
- A nested `fn` item **cannot capture** enclosing locals; referencing an outer
  local is a rustc error. A nested function that needs an outer value takes it
  as a parameter (C-style). This is the deliberate cost of hoisting.
- **Capture ownership moves to `__func__`.** The existing nested-function
  capture machinery (`nested_function_captures`, the mutable-capture `let mut`
  logic, the capture tests) described the old closure lowering; it is removed
  from the nested-function path and is the mechanism the `__func__` closure
  path uses instead.

### Change 2 — `__func__` keeps the closure lowering and captures
- `__func__(typed-params){body}` lowers to `let <tmp> = |params| -> ret { body }`
  / an inline closure, exactly as ADR-0013's Workstream B specified. It captures
  enclosing locals (rustc infers the capture), is not hoisted, and is not
  recursive.
- **`return` inside a `__func__` body is overloaded but unambiguous.** It
  returns from the anonymous function to its caller, not from the enclosing
  function. The overload is safe to reason about *because* a `__func__` closure
  cannot recurse or name itself — there is no self-call reading for `return` to
  be confused with, so inside a `__func__` body `return` can only mean "return
  from this `__func__`". Recursion, when wanted, is written as a **named nested
  function** (Change 1), which hoists to a recursive `fn` item.

### Carried forward from ADR-0013 (unchanged)
- **Workstream A** — the `fn(T1, T2) -> R` function-pointer *type* surface, a
  single `type_expr` arm over the existing `Type::Function`. Resolves DRIFT-3.
- **`__func__` surface** — a normal Crusty function declaration with `__func__`
  in the name slot, typed parameters (no untyped-param inference), a statement
  body, an optional C-style return type, `Expression::AnonFunction { params,
  return_type, body }` reusing `Param`/`Block`.
- **Multiple independent `__func__` definitions** coexist (keyword, not a name;
  never in the symbol table), even with identical signatures.
- **Non-goals** — no `move`, no capturing `__func__` stored in a struct field
  (needs `Box<dyn Fn>`, deferred), no `Fn*` trait-bound reasoning, no
  higher-ranked lifetimes. A capturing value stored in a field still awaits the
  boxed-trait-object decision.

## Consequences
- **Easier:** nested functions become genuinely C-like — forward references,
  recursion, mutual recursion all work, because they are Rust `fn` items. The
  two surface forms now have a clean, memorable rule: **name it to hoist and
  recurse; `__func__` it to capture.** `return` ceases to be ambiguous.
- **Harder / accepted cost — this is a behavioural CHANGE, not an addition:**
  a nested function that today captures an enclosing local (relying on the
  closure lowering) will, after Change 1, be a rustc error and must pass the
  value as a parameter. The capture tests on the nested-function path are
  rewritten or moved to `__func__`. This is the one migration cost; it is
  accepted because it buys the C-like semantics the project targets.
- **Reversible-cost:** the lowering choice is an implementation detail behind a
  stable surface, so it can be revisited without breaking source — except that
  a program which came to rely on nested-function capture would break, which is
  precisely why the change is recorded here rather than made silently.

### Alternatives considered
- **Keep nested functions as capturing closures (ADR-0013).** Rejected on
  review: it denies hoisting and recursion, which C nested functions have and
  the project's C-like goal wants.
- **Lower a nested function to a `fn` item when it captures nothing, else a
  closure.** Rejected: "hoisted sometimes" is a subtler rule to teach than
  "named = `fn` item, `__func__` = closure", and it blurs the clean one-to-one
  mapping. The split by surface form is simpler (worse-is-better).
- **Drop `__func__` and make nested functions do both jobs.** Rejected: no
  single Rust construct both hoists and captures, so one form cannot serve both.

## Verification
- [x] `Statement::NestedFunction` codegen emits a Rust `fn name(params) -> ret
      { body }` item, not a `let`-closure.
- [x] A named nested function is callable **before** its textual definition in
      the enclosing body (hoisting), proven end-to-end: transpile, compile under
      `rustc -D unused_parens` (RC=0), run.
- [x] A **recursive** named nested function (e.g. factorial) transpiles,
      compiles, and runs with the expected result.
- [x] A nested function that references an enclosing local is a rustc error
      (documented as the non-capture consequence), while the same logic written
      with the local passed as a parameter compiles.
- [x] `__func__` still lowers to a capturing closure: a `__func__` that reads an
      enclosing local transpiles, compiles, and runs.
- [x] The nested-function capture machinery no longer runs on the
      nested-function path; any capture tests are rewritten against `__func__`
      or removed.
- [x] Docs (control-flow / functions) state the split: named nested function →
      hoisted, recursive, non-capturing Rust `fn`; `__func__` → capturing,
      non-hoisted, non-recursive closure; and the overloaded-but-unambiguous
      `return`.
- [x] ADR-0013 status is set to `superseded by ADR-0014`.
- [x] `cargo test` green and `clippy` clean after each phase.
