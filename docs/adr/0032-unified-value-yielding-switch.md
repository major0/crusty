# ADR-0032: Unified value-yielding `switch` (retiring `match`)

## Status
accepted (supersedes ADR-0030; supersedes ADR-0012)

## Context

Crusty grew two multi-way matching constructs, plus the ternary:

- the C-style **`switch`** statement (control flow, brace-block case bodies);
- the Rust-style **`match`** expression (ADR-0012 — `=>` arms, patterns, yields
  a value);
- the C **ternary** `c ? a : b` (inline 2-way value choice — unaffected here).

Rust itself has **no `switch`** — it has only `match`, which is an *expression*:
control flow is just the case where its value is `()`. Crusty split that one
Rust construct into two surfaces (C `switch` + Rust `match`), and ADR-0030 began
cleaning up `switch`'s case-value syntax (comma-list, `...` ranges, no `|`, no
stacking) while keeping both.

Pushing on that split exposed that it does not earn its keep:

- A **`switch` is not a function**: it is not called, pushes no stack frame, and
  is not a boundary. So ADR-0029's "function bodies require `return`, no bare
  trailing value" does **not** apply to it — a `switch` arm is a *value-block*
  (ADR-0029 category 2), exactly like an `if`/`else` branch or a (former) match
  arm. It therefore can yield a value by a trailing expression, and a `return`
  inside it passes through to the enclosing function (proven: a `return` in a
  `switch` arm lowers to a `return` in a Rust `match` arm, which exits the
  enclosing `fn`, not the match).
- Once `switch` is value-yielding and takes patterns/ranges, it does everything
  `match` does — in C/Go-flavoured `case … :` syntax rather than Rust `=>`
  arms. `match` becomes a redundant second spelling of the same Rust engine.

Both constructs already lower to the **same Rust `match`**. Keeping two surfaces
for one engine is the duplication this ADR removes.

## Decision

Crusty has **one** multi-way construct: a value-yielding **`switch`**. The
Rust-style `match` expression (ADR-0012) is **retired** — its grammar, AST
(`Expression::Match`, `MatchArm`), codegen, and docs are removed, and its
pattern power is carried by `switch`. The ternary is unchanged.

`switch` is an **expression** (it yields a value; control-flow use is the case
where the value is unused / `()`), with these case forms:

```c
switch (foo) {
    case X, Y, Z: value;        // comma-list   -> X | Y | Z => value
    case a ... b: value;        // range        -> a..=b    => value
    case n:       expr;         // single, bare expression body
    case i:       { expr }      // single, block body (value via ADR-0029 tail)
    default:      expr;         // -> _ => expr
}
let i = switch (foo) { case 1: bar; default: baz; };   // i = bar / baz
```

Rules:
- **A `switch` arm is a value-block, not a boundary.** Its value is its body
  expression. A bare body `case n: expr;` yields `expr`; the trailing `;` is the
  **arm terminator** (the C `case X: value;` shape), NOT ADR-0029's
  statement-discard `;` (that rule governs function-body statements, and a
  `switch` arm is neither a function body nor a free statement block). A block
  body `case i: { … }` yields via the ADR-0029 value-block tail.
- **`return` inside an arm exits the enclosing function** (the `switch` is not a
  boundary; it passes through).
- **`break`/`continue` inside an arm target the enclosing loop**, with the
  existing dotted-label and break-value forms: `break .outer v;`,
  `continue .outer;`. (A C `break` inside a `switch` escapes only the switch;
  Crusty's targets the loop, fixing that footgun, because `switch` lowers to a
  Rust `match`, which is not itself breakable.)
- Case values follow ADR-0030: comma-list and `...` ranges, `|` rejected in a
  case value, no `case:`-stacking, no fallthrough (Rust has none to lower to).
- **Patterns**: the structural subset `match` carried (wildcard `default`/`_`,
  literal, binding, range, path) plus the `Raw` verbatim-passthrough for
  payload/tuple/struct destructuring moves onto `switch` case values.
- Lowering is unchanged in spirit: `switch (e) { … }` → Rust `match e { … }`,
  each arm an independent `pat => body`.

## Consequences

Easier:
- One multi-way construct, C/Go-flavoured, that does control-flow AND value
  return AND patterns — no C-vs-Rust duplication, one thing to learn and
  maintain. The ternary covers inline 2-way choice; `switch` covers n-way.
- The value-yielding use (`let i = switch …`) and the control-flow use are the
  same construct differing only by whether the value is used — Rust's unified
  model, in C clothing.
- No ADR-0029 conflict: `switch` is explicitly on the value-block
  (non-boundary) side of the line ADR-0029 drew.

Harder / accepted:
- **Large, cross-cutting, breaking change.** Removing `match` touches ~28
  non-test references across parser/AST/codegen/semantic plus several test files
  and two docs; every `match (e) { p => b }` in source/tests/examples migrates to
  `switch (e) { case p: b; }`. This lands in proven slices, not one pass.
- A `switch` arm's `;` is a terminator, not a statement separator — a deliberate,
  documented local rule distinct from ADR-0029's function-body `;`.
- Crusty diverges further from a 1:1 C `switch` (which is a fallthrough statement
  with no value) — accepted: the value-yielding, no-fallthrough `switch` is the
  intended Crusty semantics, and it still lowers to idiomatic Rust.
- The old statement-`switch` raised its own "case value type mismatch" when a
  case value's type disagreed with the scrutinee. That check is removed: a case
  is now a pattern, and pattern/scrutinee type agreement is rustc's, consistent
  with the retired `match` and ADR-0001. Crusty reports fewer local diagnostics
  here, deferring to rustc.

## Rejected alternatives

- **Keep `switch` and `match` both** (ADR-0030 as written). Rejected: two
  surfaces for one Rust engine; the split's only distinction ("do you want the
  value") is the artificial one Rust itself removed.
- **Collapse to `match`-only** (full Rust). Rejected: sheds the C/Go `case … :`
  surface the project values; Crusty is a C interface, not Rust reskinned.
- **Make `switch` value-yielding but keep `match`.** Rejected: leaves `match` as
  a pure Rust-flavoured duplicate of the now-more-powerful `switch`.

## Verification

- [x] `let i = switch (x) { case 1: bar; default: baz; };` type-checks and emits
      Rust `let i = match x { 1 => bar, _ => baz };`; `i` holds the arm value.
- [x] `case X, Y, Z: v;` → `X | Y | Z => v`; `case a ... b: v;` → `a..=b => v`.
- [x] `case n: { s; expr }` yields `expr` via the block tail; `case n: expr;`
      yields `expr` (the `;` terminates the arm).
- [x] `return` inside a case exits the enclosing function; `break .outer v;` /
      `continue .outer;` target the enclosing loop. Proven under `rustc`.
- [x] A destructuring case value (`Some(x)`, `[first, .., last]`) carries through
      verbatim and compiles, as the retired `match` did.
- [x] `Expression::Match` / `MatchArm` / `match_expr` are removed; no source,
      test, example, or doc references the `match` construct; the full suite is
      green under `rustc -D unused_parens`. (`match` stays a reserved word, so
      old `match (x) { … }` is a clean parse error, not an identifier.)
- [x] ADR-0030 and ADR-0012 statuses are set to `superseded by ADR-0032`.
