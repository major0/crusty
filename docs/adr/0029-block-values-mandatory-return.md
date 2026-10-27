# ADR-0029: Block values — mandatory `return` in function bodies, bare trailing values in value-blocks

## Status
proposed

## Context

Crusty today has no block-as-expression: a `let x = { … }` does not parse, a
`match` arm must be `pat => expr` (no block body), and the only way to compute a
scoped value is an immediately-invoked `__func__` closure. The docs record the
limitation bluntly: "Crusty blocks have no bare trailing-expression value."

This forces awkward workarounds and leaves real gaps (block-bodied `match`
arms). The question is how a block should yield a value — and specifically
whether `return` should be required, optional, or forbidden in each block
position.

Two facts about Rust ground the decision (all verified by compiling and running
the forms, not by reading source):

- `return` inside a Rust **closure** returns from the **closure**, tail or
  early — it never escapes to the enclosing function. A **bare block** `{ … }`
  is *not* a function boundary, so a `return` inside it exits the nearest
  enclosing `fn`/closure, and the block yields a value only via a trailing
  (semicolon-less) expression.
- In **tail** position, `return x;` and a bare trailing `x` are semantically
  identical in both functions and closures.

The design tension: Rust lets `return` do double duty (early-exit *and* a
closure's value-yield). That overloading is exactly what makes a nested
value-block's control flow hard to read — a sibling `if`/`else` where one branch
yields locally and the other `return`s has two different reaches that look
symmetric. Crusty prefers one meaning per construct.

## Decision

A block yields a value by one of two rules, chosen by whether the block is a
**function boundary**:

1. **Function and closure bodies (`fn`, `__func__`) require `return`.** A
   non-void body must yield its value with `return v;`. A bare trailing value
   (`int f() { 42 }`) is a Crusty error — `return` is mandatory even in tail
   position. This is the C expectation (functions return with `return`) and
   keeps the function-vs-value-block distinction visible.

2. **Value-blocks yield by a bare trailing expression.** A `let x = { … }`
   block, an `if`/`else` used as a value, and a `match` arm take their value
   from the block's final semicolon-less expression. These are not function
   boundaries.

3. **`return` has exactly one meaning everywhere: leave the nearest enclosing
   function or closure.** It is never "yield this block." Inside a value-block,
   `return v;` exits the *containing* function/closure, not the block — so
   `let x = { if (c) { return 0; } 5 };` returns from the enclosing function
   when `c`, and otherwise binds `x = 5`.

`return` is therefore mandatory for value-yield in a body, mandatory for any
early exit, and never used to yield a value from a value-block.

## Consequences

Easier:
- `let x = { if (c) { 1 } else { 2 } };` and block-bodied `match` arms become
  expressible with no closure ceremony.
- `return` carries a single, C-and-Rust-consistent meaning — "leave the
  function" — so a nested value-block's control flow is legible: a `return` in
  any branch always exits the function, never the block.
- Codegen stays dumb: a value-block passes straight through to a Rust bare block
  with trailing-expression branches; a function/closure body preserves its
  `return` verbatim. No tail-rewriting.

Harder / accepted:
- Crusty `fn`/`__func__` bodies are deliberately **not** Rust-idiomatic: Rust
  allows the tail expression to be the value there, Crusty requires `return`.
  This is an intentional divergence, enforced by a new semantic check.
- `analyze_function` must gain a missing-return check (it has none today), and
  reject a non-void body whose value-path reaches the end without `return`.
- A `return` in value-block tail position (`let x = { return 0; };`) exits the
  function and leaves `x` unbound — well-defined under the single `return`
  meaning, but a probable mistake worth a lint.
- The grammar gains block-as-expression; value-block type inference follows the
  tail expression (descending `if`/`else` and `match`-arm tails).
- Reverses the current docs note "Crusty blocks have no bare trailing-expression
  value" and closes the block-bodied-match-arm gap.

## Rejected alternatives

- **Bare trailing values allowed in function bodies too (full Rust uniformity).**
  Rejected: saves 7 bytes and a keyword that documents intent, with no logical
  delta in tail position, while erasing the visible function-vs-value-block
  distinction. `return` in a function body is the C expectation.
- **Keep `return` as the only value-yield everywhere; no bare value-blocks.**
  Rejected: forces every scoped value through an `__func__(){ … }()` closure,
  adding Crusty-specific "how do I get a value" rules the developer must learn —
  more unknowns, not fewer. Value-blocks are the common, familiar need.
- **Let `return` double as a value-block's yield (Rust's overloading).**
  Rejected: a block where one branch yields via trailing expression and a
  sibling via `return` has two different control-flow reaches that read as
  symmetric — the braceless-`if` class of edit-time footgun. One meaning per
  construct instead.
- **Lower a zero-param `__func__` to a bare block (strip the tail `return`).**
  Rejected: only sound for tail returns; an early `return` would silently escape
  the enclosing function, and the lowering would flip semantics on an innocent
  edit. The closure IIFE keeps `return` closure-scoped in all cases.

## Verification

- [ ] `int f() { 42 }` (bare tail value in a function body) is a Crusty error;
      `int f() { return 42; }` compiles.
- [ ] `let x = { if (c) { 1 } else { 2 } };` parses, type-checks, and emits a
      Rust bare block with trailing-expression branches; `x` holds `1`/`2`.
- [ ] `let x = { if (c) { return 0; } 5 };` — when `c`, returns from the
      containing function; otherwise `x == 5`. Proven against emitted Rust.
- [ ] A block-bodied `match` arm (`pat => { …; v }`) yields `v` via the trailing
      expression.
- [ ] A `__func__` body still requires `return` and emits a closure whose
      `return` is closure-scoped; a value-block nested inside it yields locally
      while an inner `return` exits that `__func__`. Proven: `run(7) == 1107`,
      `run(3) == 1000`.
- [ ] Emitted Rust compiles under `rustc -D unused_parens -D unused_braces`.
