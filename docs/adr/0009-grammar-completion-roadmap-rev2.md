# ADR-0009: Grammar-completion roadmap (rev. 2)

## Status
accepted (supersedes ADR-0008)

## Context
ADR-0008 ranked the grammar-completion gaps into tiers on one load-bearing
premise: that each Tier-2 "semantic stub" could be finished with data the
symbol table and type environment *already hold*. Implementing the first
Tier-2 item exposed that this premise is false for two of the three.

Reading the data model while wiring the stubs (code in hand, not speculation):

- `TypeKind::Struct` in `semantic.rs` stores only `{ fields }`. Struct
  **methods** are analysed through `analyze_function` and land in the symbol
  table under their **bare name** (`new`, not `Point::new`) — they are *not*
  keyed by their owning type and are *not* recorded in the type environment.
- Therefore a `MethodCall` (`x.foo()`) or a `TypeScopedCall` (`Type::m()`)
  has **no existing structure** from which to resolve the called method's
  return type. Finishing SEM-01/SEM-02 requires NEW infrastructure — a
  method-signature table keyed by owning type — plus a surface/type-system
  decision (how `Type::method` resolves, how instance `self`-methods differ
  from associated/static ones). That is a design-first change, not a stub.

SEM-05 (StructInit) was genuinely a stub: struct fields are already recorded
and read by `FieldAccess`, so it was finished in place with the data that
existed, exactly as ADR-0008 assumed.

This ADR supersedes ADR-0008: it keeps the whole ranking, records the tiers
already completed, and moves SEM-01/SEM-02 out of Tier 2 (which it wrongly
framed as a stub-finish) into the design-first tier, each gated on its own
ADR. Gap IDs reference the original audit facets (`PS-*`, `CG-*`, `SEM-*`,
`DRIFT-*`).

## Decision
We will close grammar-completion gaps in the following tiers, top to bottom;
within a tier, order is by value-to-effort. A later tier is not started until
earlier tiers are done or explicitly deferred with a recorded reason. Items
marked **[done]** have landed on `main`; the ranking below is otherwise
unchanged from ADR-0008 except for the SEM-01/SEM-02 reclassification.

### Tier 0 — Correctness bugs **[done]**
- **CG-3 [done]** `i++`/`i--`/`++i`/`--i` double-evaluation + `&mut` of a copy.
- **CG-2 [done]** assignment / compound-assignment emitted as a parenthesised
  value-expression (invalid as a statement).

### Tier 1 — Surface rules for features already built behind them **[done]**
- **PS-01 [done]** error-propagation `expr?`.
- **PS-02 [done]** fallible return type `Type?`. (PS-01 + PS-02 shipped
  together, as required — the error-handling story is only usable with both.)
- **PS-10 [done]** `__rust__{ … }` escape hatch reachable in item and
  expression position, not only inside `extern`.

### Tier 2 — Finish the genuine semantic stubs (data already exists)
- **SEM-05 [done]** `StructInit` field-name / field-type / completeness
  checking (struct fields were already recorded and read by `FieldAccess`).

SEM-01 and SEM-02 were listed here by ADR-0008; they are reclassified into
Tier 4 below, because the data they need does **not** already exist.

### Tier 3 — Literals and small surface conveniences
Local grammar/lexer additions; each independently shippable.
- **PS-03** non-decimal integer literals `0x`/`0o`/`0b` and `_` separators.
- **PS-04** integer/float literal type suffixes (`0u8`, `2.5f32`).
- **PS-05** raw / byte strings and `\x` / `\u{…}` escapes.
- **PS-11** C/POSIX type aliases and `usize`/`isize` keyword rules.
- **PS-12** array-repeat literal `[x; N]`; **PS-13** struct functional-update
  `..base`; **PS-17** `pub` visibility modifiers and expr/stmt attributes.
- Confirmed smaller items: **PS-15** `@Type.CONST` no-paren associated access,
  **PS-14** turbofish `::<>`, **PS-16** type-keyword method names (`.short()`),
  **CG-1** `unused_parens` from the `Binary` codegen arm, **SEM-06/07/08**
  (field-access-on-unknown-type, `usize` indexing + `Vec<T>` indexing,
  enum→int cast) — each small and independently fixable.

### Tier 4 — Design-first features (each its own ADR before coding)
Each is a surface and/or type-system decision that must be designed (and
ADR'd) before implementation.
- **SEM-00 (method-signature typing)** — record method signatures in the type
  environment keyed by owning type, then implement:
  - **SEM-01** `MethodCall` receiver/arity/return typing.
  - **SEM-02** `TypeScopedCall` real return type (stop echoing the scope type).
  These were Tier-2 in ADR-0008; they are design-first because the type
  environment holds no method signatures today, and resolving `Type::method`
  vs instance `self`-methods is a type-system choice, not a stub completion.
  SEM-00 is specified in ADR-0010.
- **PS-08** generic parameters on function/struct/enum *definitions*
  (`<T>`, `where`, lifetimes, const generics), with **SEM-03** (explicit-
  generic calls discard type args) and **SEM-04** (generic-argument inference;
  array-literal ↔ `Vec<T>`). One cohesive workstream. Specified in ADR-0011.
- **PS-07** closures/lambdas (`|x| expr`) and function-pointer *types*
  (`Type::Function`, DRIFT-3). Specified in ADR-0013, nested-function lowering
  revised by ADR-0014.
- **PS-06** `match` expression/statement (keyword already reserved). Specified
  in ADR-0012.
- **PS-09** `trait`/`impl` items; `unsafe`; `async`. Scoped and sequenced in
  ADR-0015 (commits `unsafe` + the `typedef … @Type` impl surface, which also
  resolves DRIFT-1; defers defining new `trait`s and `async` to their own ADRs;
  records `__thread__` as a future direction).

### Tier 5 — Documentation reconciliation (do alongside the relevant tier)
- **DRIFT-1** `typedef.md` impl-block family — RESOLVED: the `typedef … @Type`
  impl surface is implemented (ADR-0015 Unit 2, `Item::ImplBlock`) rather than
  cut, and `typedef.md` marks it implemented.
- **DRIFT-5** `expressions.md` multi-segment / `->` / no-paren `@Type` calls —
  demote to "Planned".
- **DRIFT-4** `control-flow.md` `for_stmt` EBNF marks the condition optional;
  the grammar requires it — fix the EBNF.
- **DRIFT-2 / DRIFT-3** fallible-type and function-pointer-type docs — DRIFT-2
  is resolved by PS-02 (done); DRIFT-3 is resolved — the `fn(T) -> R` type
  surface landed (ADR-0013 Workstream A), so function-pointer types are
  spellable in Crusty source.

## Consequences
- **Easier:** the ranking stays lowest-risk-first and shippable at each step,
  and the roadmap now tells the truth about what each remaining item costs —
  SEM-01/SEM-02 will not be mistaken for a quick stub-finish again.
- **Harder / accepted cost:** method-call/type-scoped-call return typing is
  deferred behind a design step (SEM-00) and its own ADR. Until then those
  expressions keep laundering through `Type::Auto`, so rustc remains the
  arbiter for method-call typing — consistent with ADR-0001's syntax-layer
  stance, but weaker local diagnostics in the meantime.
- Closing a gap does not retire its ID here; progress is tracked by the gaps'
  own commits/ADRs. A further material change to the ranking supersedes this
  ADR rather than editing its decision.

## Verification
- [x] ADR-0008 status is set to `superseded by ADR-0009`.
- [x] Every gap ID in ADR-0008 appears in exactly one tier here (none dropped):
      SEM-01/SEM-02 moved to Tier 4 under SEM-00; all others carried over.
- [x] Items already landed on `main` are marked **[done]** (Tier 0, Tier 1,
      SEM-05) and match the commit history.
- [ ] SEM-01/SEM-02, when started, are preceded by an ADR for SEM-00
      (method-signature typing), per `adr.md`.
- [ ] An engineer or agent can pick the next task without re-deriving
      priorities — the top undone item in the lowest incomplete tier is next
      (currently: Tier 3).
