# ADR-0026: C99-style `if` / `while` conditional binding — `(<decl>; <cond>)`

## Status
accepted

## Context

Supersedes ADR-0025 (which proposed `if (let PAT = expr)` lowering to Rust
`if let`). That form was a pattern-match; this ADR adopts a different, more
C-native capability: a **declaration-then-condition** header, modelled on C99's
`for (int i = 0; …)`.

C99 introduced the scoped declaration in a `for` header specifically so the loop
variable does not leak into the enclosing scope — closing the traditional-C
pattern of declaring the index outside the loop and reusing (and attacking) that
storage. Crusty can give `if` and `while` the same scope-tight binding by reusing
C's own `(init; cond)` header vocabulary:

```c
if (int i = compute(); i < 10) {
    use(i);
} else {
    recover(i);
}

while (let line = next(); line.valid()) {
    process(line);
}
```

`i` / `line` is scoped to the whole statement (header + branches), then gone — the
C99 `for`-header guarantee applied to conditionals.

Crucially this needs **no `let` keyword**: the init clause is a full Crusty
declaration, which is already self-marking — C-typed (`int i = e`), `let i = e`,
or `var i = e`. The `;` separates the declaration from the boolean condition. The
third `for`-clause (increment) is dropped — a conditional has no iteration step.

This is **bind-then-test, not pattern destructuring.** It binds a value and tests
a boolean; it does not peel `Some(x)` out of an `Option`. Rust's `if let` is a
separate capability — `match` remains Crusty's destructuring path, and a dedicated
`if let` ADR may follow if a real need appears.

## Decision

Add an optional `<decl>;` prefix to the parenthesised condition of `if` and
`while`:

```
if  "(" [ <decl> ";" ] <bool-expr> ")" <block> [ else <block-or-if> ]
while "(" [ <decl> ";" ] <bool-expr> ")" <block>
```

- `<decl>` is any Crusty declaration statement — `int i = e` (C-typed),
  `let i = e`, or `var i = e` — reusing the existing decl grammar
  (`c_style_decl_stmt` / `let_stmt` / `var_stmt`). Mutability follows the form
  (`var` is mutable). No new `let`-in-condition grammar.
- `<bool-expr>` is a single ordinary boolean condition — no comma operator
  (see *Comma handling* below).
- The `<decl>;` prefix is **OPTIONAL sugar**. With no prefix, the condition is
  exactly today's `if (cond)` / `while (cond)`, parsed to `Statement::If` /
  `Statement::While` unchanged — the bare form is the default and the common case,
  and nothing about it changes. The prefixed form is purely opt-in for when a
  scope-tight binding is wanted.

### Comma handling

C's `for (int i = 0, j = 0; …)` uses a comma that is **not** the comma operator —
it is the declarator separator, declaring `i` and `j` under the shared type `int`.
The comma *operator* (evaluate-left-discard-return-right) appears only in C's
expression-init and increment clauses, both of which this conditional header does
not have.

Crusty's conditional header admits **neither** comma form:

- The init clause is a **single declaration** — no declarator list (`int i = 0,
  j = 0` is not accepted here; see *Not in scope*).
- The condition is a **single boolean expression** — the comma operator is not
  admitted in the condition slot, so there is no obscure `(…, …)` chaining to
  reason about. (Crusty's comma-operator support elsewhere is unaffected; it is
  simply not part of this two-clause header.)

This keeps the header exactly two unambiguous clauses separated by one `;`.

### Scoping

The binding is scoped to the entire statement: the condition and both branches
(for `if`) or the body (for `while`) can see it, and it is dropped when the
statement ends. This is the C99 `for`-header scope, expressed by wrapping the
lowered statement in a block.

### AST

Two additive statement variants — no reshape of `If` / `While`:

```rust
Statement::IfBind {
    init: Box<Statement>,     // a Let / Var decl statement
    condition: Expression,
    then_block: Block,
    else_block: Option<Block>,
},
Statement::WhileBind {
    label: Option<Ident>,
    init: Box<Statement>,
    condition: Expression,
    body: Block,
},
```

`init` reuses the existing `Statement::Let` / `Statement::Var` so the three decl
forms and their mutability are inherited unchanged.

### Lowering

Wrap the binding and the conditional in a block, so the binding is scoped to the
statement and dropped after:

```rust
// if (int i = compute(); i < 10) { A } else { B }
{
    let i = compute();
    if i < 10 { A } else { B }
}

// while (var i = seed(); i < n) { body }
{
    let mut i = seed();
    while i < n { body }
}
```

The wrapping block is valid Rust today and needs no block-as-expression (it wraps
the statement, not the condition). A `while`'s binding is initialised once, before
the loop — matching the reading of `while (decl; cond)` as "bind once, then loop
on cond" (not a per-iteration rebind).

### Not in scope

- **Pattern destructuring** (`if let Some(x) = opt`). This form binds a value and
  tests a boolean; it does not destructure. `match` covers destructuring; a
  separate `if let` ADR may add it later.
- **The increment clause** (`for`'s third header slot). A conditional has no
  iteration step; `(decl; cond)` is two clauses only.
- **Multiple declarations in the init clause** (`int i = 0, j = 1`). This is C's
  declarator-list comma (one declaration, several declarators sharing a type) —
  legal in C's `for`, but deferred here: one declarator per header, matching the
  common case. A declarator list can be a later additive change. The comma
  *operator* is never admitted in the header (see *Comma handling*).

## Consequences

**Easier.** Scope-tight conditional bindings with familiar C99 `for`-header
syntax, no `let` keyword required, and no leak of the binding into the enclosing
scope. Reuses the existing declaration grammar wholesale.

**Harder / accepted costs.** Two new `Statement` variants ripple to the exhaustive
statement matches (codegen `generate_statement`, semantic `analyze_statement`, the
capture walkers) — additive arms, not a reshape. The `if`/`while` grammar must try
the `<decl>;` prefix before a bare condition; the decl rules already end at a `;`,
so the lookahead is clean. Semantic analysis must open a scope spanning the init
binding and the branches, mirroring the lowered block.

**Neutral.** A bare `if (cond)` / `while (cond)` is untouched (byte-identical). No
new reserved keyword. The labeled-loop `.label:` prefix composes with
`while (decl; cond)` via the existing mechanism.

## Verification

- [x] `if (int i = f(); i < 10) { … }` parses to `Statement::IfBind` with a `Let` init, the `i < 10` condition, and the then-block.
- [x] `let`/`var`/C-typed init clauses all parse; `var i = …` lowers to `let mut i = …`.
- [x] `if (decl; cond) { … } else { … }` populates `else_block`; an `else if` chains.
- [x] `while (let x = seed(); x < n) { … }` parses to `Statement::WhileBind`; a `.label:` prefix sets its `label`.
- [x] A bare `if (x > 0) { … }` / `while (c) { … }` still parses to `Statement::If` / `Statement::While` (no regression) — the `(decl; cond)` prefix is optional sugar, not the default.
- [x] The condition slot does not admit a declarator list (`int i = 0, j = 0; …` in the header is rejected) nor a comma-operator condition — the header is exactly `<decl>? ";" <bool>` with a single boolean.
- [x] Codegen wraps the statement in a block: `{ <init>; if <cond> { … } [else …] }` / `{ <init>; while <cond> { … } }`, with the binding scoped to that block.
- [x] End-to-end under `rustc -D unused_parens`: an `if (int i = …; i < k)` using `i` in both branches, and a `while (var i = …; i < n)` mutating and testing `i`, compile and run; `i` is not visible after the statement.
- [x] Semantic analysis: the init binding is in scope in the condition and the branch(es), and not after the statement.
- [x] Full suite green (`cargo test`), clippy clean.
- [x] `docs/syntax/` documents the `(decl; cond)` conditional with its scoping and lowering.

## Alternatives considered

- **`if (let PAT = expr)` pattern-match → Rust `if let` (ADR-0025).** Superseded:
  it is a different capability (destructuring, not bind-and-test), requires the
  `let` keyword, and is less C-native. The `(decl; cond)` header needs no keyword,
  scopes like C99 `for`, and reads as C. Destructuring stays with `match`.
- **Reshape `If` / `While` to carry an optional init.** Rejected: ripples to every
  `If`/`While` construction/destructuring site for a form most conditionals do not
  use. Two additive variants isolate the feature (same call as `Expression::Match`,
  `StructVariantInit`).
- **Lower to a Rust block-expression condition** (`if { let i = …; cond } { … }`).
  Rejected: that scopes the binding to the *condition only*, so the branches could
  not see `i` — the opposite of the C99 intent. Wrapping the whole statement in a
  block scopes `i` to the branches too, which is the point.
- **Per-iteration rebind for `while`** (re-run the init each iteration). Rejected:
  `while (decl; cond)` reads as "bind once, loop on cond"; a per-iteration rebind
  is what `for` expresses, and conflating them would surprise. The init runs once,
  before the loop.
- **Keep the increment clause (full `for`-style triple).** Rejected: an `if` has no
  iteration, and a `while` with an increment IS a `for` — Crusty already has
  `for (init; cond; incr)`. The conditional header is two clauses by design.
