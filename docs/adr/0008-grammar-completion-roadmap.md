# ADR-0008: Grammar-completion roadmap

## Status
accepted

## Context
A full read-only audit of the Crusty transpiler (parser surface, codegen,
semantic analysis, and docs-vs-implementation, 2026-10-23) produced a large,
evidence-backed gap inventory. The gaps are uneven in both cost and value, and
several are entangled (a feature is built in the AST/codegen/docs but blocked by
one missing grammar rule). Without an agreed ordering, the gaps would be closed
in arbitrary order, redone, or bundled into oversized changes.

The audit's load-bearing finding: **the grammar surface is the binding
constraint**. Repeatedly, a feature exists in `ast.rs`, is emitted by
`codegen.rs`, and is specified in `docs/syntax/` — and is unreachable only
because the `peg` grammar has no production for it. Those gaps are therefore
cheap to close relative to their value. A second cluster is genuine semantic
stubs (checks whose data already exists in the type environment), and a third
is two codegen mis-compiles that emit wrong Rust.

This ADR records ONE decision: the PRIORITIZATION of the grammar-completion
work into tiers. It does not implement any gap. Each gap whose closure is a
hard-to-reverse surface-syntax or type-system choice graduates to its own ADR
when it is taken (per `adr.md`); this roadmap is the ordering and the rationale.

Gap IDs below reference the audit facets: `PS-*` parser surface, `CG-*`
codegen, `SEM-*` semantic, `DRIFT-*` docs. Full per-facet reports were captured
in the audit run; the consolidated report is the source for this ADR.

## Decision
We will close grammar-completion gaps in the following tiers, top to bottom.
Within a tier, order is by value-to-effort. A later tier is not started until
the earlier tiers are either done or explicitly deferred with a recorded reason.

### Tier 0 — Correctness bugs (fix before adding features)
The transpiler emits *wrong* Rust here, not merely incomplete Rust. These
outrank every new feature.
- **CG-3** `i++` / `i--` / `--i` / `++i` codegen double-evaluates the operand
  and takes `&mut` of a copy — wrong target / borrow errors on any non-trivial
  lvalue.
- **CG-2** assignment and compound-assignment (`=`, `+=`, …) emit as a
  parenthesised value-expression — invalid as a statement; chained assignment
  mistypes.

### Tier 1 — Surface rules for features already built behind them
Highest value-to-effort: the AST variant, the codegen arm, and the docs all
exist; only the grammar production is missing. Each is a small, local grammar
change that lights up a whole feature end to end.
- **PS-01** error-propagation `expr?` (AST `Expression::ErrorProp`, codegen `?`,
  `docs/syntax/error-handling.md` all ready).
- **PS-02** fallible return type `Type?` (AST `Type::Fallible`, codegen
  `Result<T, Box<dyn Error>>`, docs ready). PS-01 + PS-02 together make the
  documented error-handling story usable; do them as one change.
- **PS-10** make the `__rust__{ … }` escape hatch reachable in item and
  expression position, not only inside `extern`. This is the designated
  fallback for everything not yet expressible, so it gates the usefulness of
  every "use `__rust__`" workaround elsewhere.

### Tier 2 — Finish the semantic stubs (data already exists)
These are unfinished functions, not missing infrastructure — the symbol table
and type environment already hold what the check needs. They currently launder
types through `Type::Auto`, silently disabling checking.
- **SEM-05** `StructInit` field-name / field-type / completeness checking
  (struct fields are already recorded and used for `FieldAccess` reads).
- **SEM-01** `MethodCall` receiver/arity/return typing (currently a stub → `Auto`).
- **SEM-02** `TypeScopedCall` real return type (stop echoing the scope type).

### Tier 3 — Literals and small surface conveniences
Local grammar/lexer additions; each is independently shippable.
- **PS-03** non-decimal integer literals `0x` / `0o` / `0b` and `_` digit
  separators.
- **PS-04** integer/float literal type suffixes (`0u8`, `2.5f32`).
- **PS-05** raw / byte strings and `\x` / `\u{…}` escapes.
- **PS-11** C/POSIX type aliases (`size_t`, `int32_t`, `byte`, …) and
  `usize` / `isize` keyword rules (AST `PrimitiveType::Usize`/`Isize` exist;
  `docs/syntax/types.md` already documents them).
- **PS-12** array-repeat literal `[x; N]`; **PS-13** struct functional-update
  `..base`; **PS-17** `pub` visibility modifiers and expression/statement
  attributes.
- Confirmed smaller items: **PS-15** `@Type.CONST` no-paren associated access,
  **PS-14** turbofish `::<>`, **PS-16** type-keyword method names (`.short()`),
  **CG-1** `unused_parens` from the `Binary` codegen arm, **SEM-06/07/08**
  (field-access-on-unknown-type, `usize` indexing + `Vec<T>` indexing, enum→int
  cast) — each small, independently fixable.

### Tier 4 — Large language-design features (each its own ADR before coding)
These are not incremental fixes; each is a surface and type-system decision that
must be designed (and ADR'd) before implementation, and sequenced deliberately.
- **PS-08** generic parameters on function/struct/enum *definitions*
  (`<T>`, `where`, lifetimes, const generics), with its semantic counterparts
  **SEM-03** (explicit-generic calls discard their type args) and **SEM-04**
  (generic-argument inference; array-literal ↔ `Vec<T>`). One cohesive workstream.
- **PS-07** closures / lambdas (`|x| expr`) and function-pointer *types*
  (`Type::Function`, DRIFT-3).
- **PS-06** `match` expression/statement (keyword already reserved).
- **PS-09** `trait` / `impl` items; `unsafe`; `async`.

### Tier 5 — Documentation reconciliation (do alongside the relevant tier)
Fix the spec so it stops describing unimplemented syntax as working. Do each
item when its feature is implemented, or now if the feature is deferred.
- **DRIFT-1** `typedef.md` impl-block family (`typedef struct {…} @Point`,
  `typedef default`, named impls) — no grammar and no AST; cut it or mark it
  "Planned". Largest doc lie; fix now.
- **DRIFT-5** `expressions.md` shows multi-segment / `->` / no-paren `@Type`
  calls as working before the caveat — demote those fences to "Planned".
- **DRIFT-4** `control-flow.md` `for_stmt` EBNF marks the condition optional;
  the grammar requires it — fix the EBNF (or make the condition optional when
  Tier-3-adjacent work touches `for`).
- **DRIFT-2 / DRIFT-3** fallible-type and function-pointer-type docs are ahead
  of the grammar — resolved when PS-02 / PS-07 land.

## Consequences
- **Easier:** work proceeds lowest-risk-first (correctness, then cheap high-value
  surface rules, then semantic depth), so the transpiler is always shippable and
  each change is small and reviewable. The error-handling unlock (Tier 1) is
  reached quickly.
- **Harder / accepted cost:** the big features (Tier 4) are explicitly deferred
  behind smaller work; users wanting generics / closures / `match` / traits wait
  longer. This is deliberate — those need design, not a rushed grammar rule.
- The tiers are a planning aid, not a contract: a tier item may be reordered
  with a recorded reason, and a Tier-4 item may be pulled forward if a concrete
  need justifies writing its ADR early.
- Closing a gap does not retire its ID here; this ADR is not edited per-gap.
  Progress is tracked by the gaps' own commits/ADRs. If the ranking itself is
  changed materially, supersede this ADR rather than editing the decision.

## Verification
- [ ] An engineer or agent can pick the next task from this ADR without
      re-deriving priorities — the top undone item in the lowest incomplete tier
      is the next action.
- [ ] Every gap ID in the consolidated audit appears in exactly one tier here
      (no gap is dropped silently).
- [ ] Tier 0 (CG-2, CG-3) is addressed before any Tier 2+ feature work begins.
- [ ] Each Tier 4 item, when started, is preceded by its own `ADR-00NN` per
      `adr.md` (generics, closures, match, trait/impl are each one decision).
- [ ] Tier 1 error-handling (PS-01 + PS-02) is implemented as a single change,
      not split, since the feature is only usable with both.
