# ADR-0028: parenthesised `match` scrutinee and the parenthesised-header rule

## Status
accepted

## Context

Crusty is a C-flavoured surface over Rust. Every control-flow construct that
takes a condition or subject wraps it in parentheses, C-style: `if (cond)`,
`while (cond)`, `for (init; cond; incr)`, `switch (expr)`, and the ADR-0026
`if`/`while (decl; cond)` binding forms. One construct did not: `match` took a
**bare** scrutinee (`match n { … }`, Rust/ML style), parsed struct-literal-free so
the arm-body `{` was not swallowed.

An audit of the grammar found exactly two paren-free headers: `for i in seq { … }`
and `match <scrutinee> { … }`. `for-in` is a deliberate bare form — an
iterator-style header with no C equivalent, and the sole sanctioned exception.
`match`'s bare scrutinee is the odd one out: it is a C-flavoured *selection*
construct (the moral successor to `switch`, which IS parenthesised), so a bare
subject is inconsistent with the rest of the language.

## Decision

`match` takes a **parenthesised** scrutinee: `match (<expr>) { … }`.

- The scrutinee is wrapped in `(` … `)`, exactly like `switch (expr)` and
  `if (cond)`.
- Because the parens delimit the scrutinee, it is parsed with the FULL
  expression grammar (no struct-literal-free restriction needed): the arm-body
  `{` can no longer be confused with a struct literal, so `match (Point { x: 1 }) { … }`
  is now expressible where the bare form had to avoid it.

### The parenthesised-header rule

All control-flow headers that take a condition/subject are parenthesised; the
`for i in seq` iterator header is the single intentional bare exception. A future
construct that selects or tests on a value parenthesises it; it does not add a
second bare form.

## Consequences

**Easier.** `match` is consistent with `switch`/`if`/`while`/`for` — one rule for
every C-style header. The parenthesised scrutinee also lifts the struct-literal
restriction, so a struct-literal scrutinee is now expressible.

**Harder / accepted costs.** A surface change to an existing construct: every
`match n { … }` becomes `match (n) { … }`. This updates the `match_expr` grammar
rule and the match tests/docs that spell the bare form. The change is mechanical
and the old bare form is no longer accepted (a clean break — `match` is young and
unreleased).

**Neutral.** No AST change (`Expression::Match` is unchanged); only the surface
and the scrutinee's parse context change. Arms, guards, and patterns are
untouched.

## Verification

- [x] `match (n) { 1 => 10, _ => 0 }` parses to `Expression::Match`.
- [x] The bare `match n { … }` form no longer parses (parens required).
- [x] A struct-literal scrutinee `match (Point { x: 1 }) { … }` parses (full
      expression grammar inside the parens).
- [x] `match` in expression position (`let g = match (s) { … };`) still parses.
- [x] Codegen emits Rust `match <scrutinee> { … }` (Rust's match is bare; the
      Crusty parens are a surface-only wrapper and are not re-emitted).
- [x] End-to-end under `rustc -D unused_parens`: a `match (x) { … }` compiles and
      runs, and the emitted Rust carries no stray parens around the scrutinee.
- [x] Full suite green (`cargo test`), clippy clean.
- [x] `docs/syntax/` shows the parenthesised `match (…)` form.

## Alternatives considered

- **Keep `match` bare (Rust/ML style).** Rejected: `match` is a C-style selection
  construct — the successor to `switch (expr)`, which is parenthesised — so a bare
  subject is the one inconsistency in an otherwise uniformly parenthesised set of
  headers. Crusty favours the C surface here over Rust's bare `match`.
- **Parenthesise `for-in` too (no bare headers at all).** Rejected: `for i in seq`
  is an iterator header with no C analogue; parenthesising it (`for (i in seq)`)
  would read as a C `for` and mislead. It stays the one intentional bare form.
- **Accept both `match n` and `match (n)`.** Rejected: two spellings for one
  construct is exactly the kind of inconsistency this ADR removes; one form only.
