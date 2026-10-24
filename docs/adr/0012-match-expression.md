# ADR-0012: `match` expression and statement (PS-06)

## Status
proposed

## Context
ADR-0009's Tier 4 lists **PS-06** — a first-class `match` surface — as a
design-first feature. The `match` keyword is already reserved (`kw_match()` in
`src/parser.rs`), but nothing consumes it: there is no `match` grammar rule, no
`Match` AST node, and no `Pattern` type.

What the code holds today (read in `src/ast.rs`, `src/parser.rs`,
`src/codegen.rs`, not speculation):

- **A C-style `switch` already exists and already lowers to a Rust `match`.**
  `Statement::Switch { expr, cases: Vec<SwitchCase>, default }` where
  `SwitchCase { values: Vec<Expression>, body: Block }`. Its Rust codegen emits
  `match expr { v1 | v2 => { … }, _ => { … } }`. So Rust `match` *lowering*
  (value lists via `|`, blocks, `default` → `_`) is already a solved, tested
  path. PS-06 is **not** "teach codegen to emit `match`".
- `switch` is **statement-only** and its arms match against **value
  expressions**, not patterns: no binding (`x`), no wildcard arm other than
  `default`, no ranges, no guards, no struct/tuple/path patterns. It cannot
  appear in expression position (`let x = switch … {}` does not parse).
- **Crusty enums carry no payload.** `EnumVariant { name, value: Option<i64> }`
  is C-style only — a variant is a name and an optional integer discriminant.
  A Crusty-defined enum therefore cannot *model* a data-carrying variant like
  `Some(x)`, so a pattern that destructures enum data has nothing to bind
  against for any enum Crusty itself declares.
- The `__rust__ { … }` escape hatch works in both item and expression position
  and is opaque to Crusty's typer (used in the PS-08 proofs). Payload-carrying
  enums and their values live there today.

So PS-06's real content is two things the `switch` path does not provide:
1. **`match` in expression position** — `let x = match e { … };`, yielding a
   value, matching Rust's loop-as-expression precedent already in the AST
   (`Expression::Loop`).
2. **Real patterns** — a `Pattern` type with the decidable, portable subset:
   literals, identifier bindings, wildcard `_`, ranges, `|`-alternation, and a
   guard (`if cond`). Enum/tuple/struct *destructuring* patterns are bounded by
   the no-payload enum limit above.

## Decision
We will add a first-class `match` **expression** (which subsumes the statement
form, since a statement is an expression in statement position) carrying a new
`Pattern` AST type, and lower it verbatim to a Rust `match`. We keep the
existing `switch` untouched.

Carried in four ordered, independently-committable phases.

### Phase 1 — AST + grammar (parse and store)
- Add `Expression::Match { scrutinee: Box<Expression>, arms: Vec<MatchArm> }`.
- Add `MatchArm { pattern: Pattern, guard: Option<Expression>, body: Box<Expression> }`
  — an arm body is an expression (a `{ … }` block is an expression), so both
  `p => expr` and `p => { … }` are one shape.
- Add a `Pattern` enum with the decidable, portable subset:
  - `Pattern::Wildcard` (`_`)
  - `Pattern::Literal(Literal)` (int / float / string / char / bool / `Raw`)
  - `Pattern::Binding(Ident)` (a bare name binds the scrutinee)
  - `Pattern::Range { lo, hi, inclusive }` (`1..=5`, `'a'..='z'`)
  - `Pattern::Or(Vec<Pattern>)` (`a | b | c`)
  - `Pattern::Path(Vec<Ident>)` (a path pattern — `Color::Red`, a unit enum
    variant or a const — carried verbatim)
  - `Pattern::Raw(String)` — a **raw passthrough pattern** for any Rust pattern
    Crusty does not model structurally (payload destructuring `Some(x)`, tuple
    `(a, b)`, struct `Point { x, y }`), captured as a verbatim string and left
    to rustc, exactly as PS-08 does for bounds / `where`.
- Grammar: a `match_expr()` rule (`match <expr-no-struct> { arm,* }`) using the
  **no-struct-literal scrutinee** context already parameterized for `for`-in
  (ADR-0007's `allow_struct` thread), so `match p { … }` is unambiguous; a
  `match_arm()` rule (`pattern [if guard] => body`); and a `pattern()` rule for
  the structural subset, with `Pattern::Raw` the fallthrough that captures a raw
  span up to the arm's `=>`.
- Phase 1 only parses and stores. No codegen, no semantics.

### Phase 2 — codegen passthrough
- `generate_expression` emits a Rust `match`: `match <scrutinee> {` then, per
  arm, the pattern, an optional `if <guard>`, `=>`, the body expression, `,`.
  Reuse the existing block/expression emitters.
- Pattern emission: `Wildcard` → `_`; `Literal` → the literal; `Binding` → the
  name; `Range` → `lo..=hi` / `lo..hi`; `Or` → `a | b`; `Path` → `A::B`;
  `Raw` → the verbatim string.
- Crusty-target emission mirrors the input (round-trip), but Rust is the proof
  target per ADR-0001.
- End-to-end proof: a `match` over an integer and over a `char` range with a
  guard and a wildcard, compiled under `rustc -D unused_parens` and run.

### Phase 3 — semantic analysis (binding scope + arm typing)
- Analyze the scrutinee, then each arm: a `Pattern::Binding` introduces its name
  into the arm body's scope typed `Auto` (rustc narrows it); a guard must be
  `bool`-compatible (permissive on `Auto`, per the SEM-relaxation precedent);
  the match's type is the arms' common body type, or `Auto` when they disagree
  on a concrete type (defer to rustc rather than hard-error).
- No exhaustiveness checking and no reachability checking — rustc owns both.

### Phase 4 — documentation
- Document the `match` surface and the `Pattern::Raw` escape in the
  control-flow docs, and state the no-payload-enum boundary explicitly (enum
  destructuring patterns require a `__rust__`-defined enum). Resolves the
  control-flow doc's silence on `match`.

### Non-goals (explicit)
- **No structured payload/tuple/struct patterns.** They ride `Pattern::Raw`
  verbatim until Crusty enums model payloads (a separate, larger decision) —
  consistent with the no-payload `EnumVariant` boundary.
- **No exhaustiveness / reachability analysis.** rustc arbitrates (ADR-0001).
- **No change to `switch`.** `switch` stays the C-style statement; `match` is
  the pattern-matching expression. They coexist; we do not migrate one to the
  other.
- **No `@`-bindings, no `ref`/`mut` binding modes** in the structural subset —
  those fall to `Pattern::Raw` if needed.

## Consequences
- **Easier:** `match` becomes usable in expression position with real patterns
  and guards, which `switch` never offered; the Rust `match` lowering is
  already proven by `switch`, so Phase 2 is low-risk.
- **Harder / accepted cost:** a new `Pattern` enum and `Expression::Match`
  variant. The `Expression` enum is matched exhaustively in codegen, semantic,
  and pretty, so each gains a `Match` arm; this is additive (a new variant), not
  a reshape, so the construction-site churn is far smaller than PS-08's field
  additions. The `Pattern::Raw` escape means Crusty will *accept* and pass
  through Rust patterns it does not understand, so a malformed destructuring
  pattern surfaces as a rustc error, not a Crusty one — the deliberate
  syntax-layer tradeoff.
- **Reversible-cost:** additive variants; reverting is removing the `Match`
  arm and the `Pattern` type. The `Pattern::Raw` passthrough is the one
  semantic commitment that would be awkward to tighten later (narrowing it
  would reject programs that compile today), so it is scoped as a non-goal
  boundary rather than a promise of structural coverage.

### Alternatives considered
- **Extend `switch` to expression position + patterns instead of a new node.**
  Rejected: `switch`'s arms are value-expression lists, not patterns; bolting a
  `Pattern` onto `SwitchCase.values` conflates two surfaces and would make the
  C-style `switch` ambiguous. A distinct `Match`/`Pattern` keeps each surface
  honest (smart data: the node shape says which surface it is).
- **A fully structural `Pattern` enum covering tuple/struct/enum-payload
  destructuring now.** Rejected for this ADR: Crusty enums carry no payload, so
  the structural forms have nothing to bind against from Crusty-defined types;
  building the full pattern grammar before the data model supports it is churn
  with no reachable use. `Pattern::Raw` covers the `__rust__`-enum case today.
- **Skip Phase 1's `Pattern` type and store arms as raw strings end-to-end.**
  Rejected: the decidable subset (literals, bindings, ranges, guards) is where
  the local diagnostics and binding-scope value lives; raw-only would push even
  trivial matches to rustc and lose the binding-scope analysis of Phase 3.

## Verification
- [ ] `src/ast.rs` has `Expression::Match { scrutinee, arms }`, `MatchArm`, and
      a `Pattern` enum with `Wildcard`, `Literal`, `Binding`, `Range`, `Or`,
      `Path`, `Raw`.
- [ ] A `match` over an integer with literal arms + a wildcard parses into a
      `Match` node with the expected arms (parser unit test).
- [ ] A `match` scrutinee of the form `match p { … }` where `p` is a plain
      identifier parses without being read as a struct literal (no-struct
      context test).
- [ ] An arm guard (`n if n > 0 => …`) and an inclusive range arm
      (`'a'..='z' => …`) parse into `guard: Some(_)` and `Pattern::Range`.
- [ ] A structural destructuring pattern (`Some(x) => …`) parses into
      `Pattern::Raw("Some(x)")` (passthrough), not a parse error.
- [ ] Codegen emits `match <s> { <pat> [if <guard>] => <body>, … }`; a
      non-generic/non-match program's output is byte-unchanged.
- [ ] End-to-end: a `let r = match … { … };` expression-position program with
      a range arm, a guard, and a wildcard transpiles, compiles under
      `rustc -D unused_parens` (RC=0), and runs with the expected output.
- [ ] A pattern binding is in scope in its arm body (a binding-scope semantic
      test); an unknown identifier in the arm body still errors.
- [ ] `switch` behavior and tests are unchanged (the existing `switch` suite
      still passes).
- [ ] `cargo test` green and `clippy` clean after each phase.
- [ ] The control-flow docs describe `match`, the `Pattern::Raw` escape, and
      the no-payload-enum boundary.
