# ADR-0025: `if let` and `while let` pattern-binding conditionals

## Status
superseded by ADR-0026

> Superseded before implementation. The `if (let PAT = expr)` pattern-match form
> (lowering to Rust `if let`) was replaced by the C99-style `if (<decl>; <cond>)`
> bind-then-test conditional (ADR-0026), which needs no `let` keyword, scopes the
> binding to the statement (C99 `for`-header semantics), and is more C-native.
> The two are different capabilities — pattern destructuring vs. bind-and-test —
> and ADR-0026 adopts the bind-and-test form as Crusty's conditional-binding
> surface; `match` remains the destructuring path. A dedicated `if let`
> destructuring ADR may follow if a real need appears.

## Context

Crusty has `match` for pattern analysis, but no `if let` / `while let` — the
single-pattern conditional-binding forms a programmer reaches for constantly when
working with `Option`/`Result` and the enum data variants just added (ADR-0023):

```c
// today: no surface — must use match or __rust__
match opt {
    Some(x) => { use(x); },
    _ => {},
}
```

Rust's `if let PAT = expr { … }` and `while let PAT = expr { … }` are the
idiomatic one-arm forms. Crusty already has the machinery: the `match` grammar's
`pattern()` rule parses the decidable structural patterns and falls back to
`Pattern::Raw` (verbatim, rustc-arbitrated) for payload/tuple/struct
destructuring (ADR-0012). `if let` / `while let` are the same pattern grammar in a
conditional position — no new pattern infrastructure.

The design questions are the surface (Crusty parenthesises conditions; Rust's
`if let` does not) and whether to reshape `If`/`While` or add variants.

## Decision

Add `if let` and `while let` as pattern-binding conditionals, reusing the
existing `pattern()` grammar.

### Surface

Crusty wraps every condition in parentheses (`if (cond)`, `while (cond)`), so the
pattern-binding form keeps the parens with a `let` binding inside:

```c
if (let Some(x) = opt) {
    use(x);
} else {
    fallback();
}

while (let Some(line) = next()) {
    process(line);
}
```

This is consistent with Crusty's parenthesised-condition convention (a C
programmer reads `if (let …)` as "an if whose condition is this binding"), and the
`let` keyword unambiguously distinguishes it from an ordinary boolean condition.
A labeled `while let` uses the existing `.label:` prefix.

### AST

Two additive statement variants — no reshape of `If` / `While`, so zero
construction-site churn:

```rust
Statement::IfLet {
    pattern: Pattern,
    expr: Expression,
    then_block: Block,
    else_block: Option<Block>,
},
Statement::WhileLet {
    label: Option<Ident>,
    pattern: Pattern,
    expr: Expression,
    body: Block,
},
```

`pattern` is the same `Pattern` the `match` arms use (structural + `Raw`).

### Lowering

Drop the Crusty parens and emit Rust's native form:

```rust
if let <pattern> = <expr> { <then> } [else { <else> }]
while let <pattern> = <expr> { <body> }
```

The pattern emits through the existing `Pattern` codegen (structural forms
rendered, `Raw` verbatim), so payload destructuring (`Some(x)`, `Shape::Rect { w, h }`)
rides `Pattern::Raw` to rustc exactly as in a `match` arm.

### Not in scope

- **`let … else`** (the diverging-binding form `let PAT = expr else { … }`) — a
  different construct (a `let` statement, not a conditional); a later ADR if
  wanted.
- **let-chains** (`if (let A = x && let B = y)`) — unstable in Rust; out.
- **Binding in an ordinary boolean condition.** `if (cond)` stays exactly as it
  is; only a condition that opens with `let` is the pattern form.

## Consequences

**Easier.** The common `Option`/`Result`/data-variant one-arm match gets a direct
surface instead of a full `match` or `__rust__`. Reuses the `pattern()` grammar
and `Pattern` codegen wholesale — no new pattern machinery.

**Harder / accepted costs.** Two new `Statement` variants ripple to the exhaustive
statement matches (codegen `generate_statement`, semantic `analyze_statement`, and
the capture walkers) — additive arms, not a reshape. The parser's condition rule
must try the `let PAT = expr` form before an ordinary expression when the
condition opens with `let`.

**Neutral.** Ordinary `if (cond)` / `while (cond)` are untouched (byte-identical).
No new reserved keyword — `let` is already one; it simply becomes legal at the
head of a parenthesised condition.

## Verification

- [ ] `if (let Some(x) = opt) { … }` parses to `Statement::IfLet` with the `Some(x)` pattern as `Pattern::Raw` and the `opt` scrutinee.
- [ ] `if (let … ) { … } else { … }` populates `else_block`.
- [ ] `while (let Some(x) = it.next()) { … }` parses to `Statement::WhileLet`; a `.label:` prefix sets its `label`.
- [ ] An ordinary `if (x > 0) { … }` / `while (c) { … }` still parses to `Statement::If` / `Statement::While` (no regression).
- [ ] Codegen emits `if let <pat> = <expr> { … }` / `while let <pat> = <expr> { … }`, parens dropped, pattern via the existing `Pattern` emitter.
- [ ] End-to-end under `rustc -D unused_parens`: an `if let` binding a tuple-variant payload and a `while let` draining an iterator compile and run.
- [ ] Semantic analysis walks the scrutinee, then the block(s) with the binding in scope; the capture walkers recurse through both variants.
- [ ] Full suite green (`cargo test`), clippy clean.
- [ ] `docs/syntax/` documents `if let` / `while let` with their lowerings.

## Alternatives considered

- **Bare `if let PAT = expr { … }` (no parens, Rust-style).** Rejected: every
  other Crusty condition is parenthesised (`if (…)`, `while (…)`); dropping the
  parens only for the `let` form is an inconsistency a C programmer would trip on.
  Keeping `if (let …)` makes the condition slot uniform.
- **Reshape `If` / `While` to carry an optional pattern.** Rejected: that ripples
  to every `If`/`While` construction and destructuring site for a form most
  conditionals do not use. Two additive variants isolate the feature (the same
  additive-over-reshape call as `Expression::Match` and `StructVariantInit`).
- **Model the bound pattern structurally instead of reusing `Pattern::Raw`.**
  Rejected: the `match` grammar already decided the structural-vs-raw split
  (ADR-0012); `if let` must behave identically, so it reuses `pattern()` rather
  than inventing a second pattern surface.
- **`let … else`, let-chains in this ADR.** Deferred/rejected: `let else` is a
  distinct statement, and let-chains are unstable in Rust — neither belongs in the
  one-arm conditional-binding surface.
