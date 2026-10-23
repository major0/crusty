# ADR-0011: Generic parameters on definitions (PS-08, SEM-03, SEM-04)

## Status
proposed

## Context
ADR-0009 groups three gaps into one cohesive design-first workstream:

- **PS-08** — generic *parameter* lists on function / struct / enum
  **definitions** (`<T>`, `where`, lifetimes, const generics).
- **SEM-03** — explicit-generic calls currently discard their type args at the
  typing level.
- **SEM-04** — generic-argument inference, including the array-literal ↔
  `Vec<T>` relationship.

What the code holds today (read in `src/ast.rs` / `src/parser.rs` /
`src/semantic.rs`, not speculation):

- Generic *uses* already work: `Type::Generic { base, args }` represents
  `Vec<int>` in any type position, and the parser builds it (see
  `type_for_scoped_call`, `struct_init_type`). `Type::Function` and
  `Type::Fallible` also exist.
- Generic *definitions* do NOT: `Function`, `Struct`, and `Enum` in `ast.rs`
  carry no type-parameter list, and the parser has no `type_params` /
  `where`-clause rule. `struct Stack<T> { … }` or `T max<T>(T a, T b)` fails
  to parse today.
- Generic *calls* exist as `Expression::ExplicitGenericCall { ty, generics,
  method, args }` (Crusty `@Type(G).method()`) and `TurbofishCall` (PS-14,
  `Type::m::<G>()`). Their codegen emits the type args verbatim, but their
  SEMANTIC typing returns the scope type / `Auto` and never uses `generics`
  to instantiate a return type — that discard is SEM-03.
- Array literals type as `Type::Array { ty, size }` (SEM-05/07 era). There is
  no relationship to `Vec<T>` — that bridge is SEM-04.

Constraint from ADR-0001 / `docs/design/philosophy.md`: Crusty is a **syntax
layer over Rust; rustc is the arbiter**. PS-08 must let Crusty *carry* generic
parameters from source to generated Rust faithfully, and improve Crusty-local
inference only where the AST makes it decidable — it must not reimplement
rustc's trait solver, lifetime inference, or monomorphisation.

## Decision
We will add generic-parameter lists to definitions as a verbatim-carried
syntactic feature, and make the two semantic sub-items do the small, decidable
part while deferring the rest to rustc. The work lands in four ordered,
independently-committable phases; each phase keeps the suite green.

### Phase 1 — AST + grammar for generic params (PS-08, grammar half)
Add a `generic_params: Vec<GenericParam>` field to `Function`, `Struct`, and
`Enum`, and a `where_clause: Option<String>` (raw, verbatim span) to each.
New AST:

```rust
pub enum GenericParam {
    Type { name: Ident, bounds: Vec<String> },   // T, T: Clone + Debug
    Lifetime { name: Ident },                     // 'a
    Const { name: Ident, ty: Type },              // const N: usize
}
```

Bounds and the `where` clause are captured as RAW strings (like `__rust__`
bodies and `MacroDefinition.body`, per ADR-0001) — Crusty does not model trait
bounds, it passes them to rustc. The parser gains a `generic_params()` rule
(`< (type_param / lifetime_param / const_param) ** "," >`, all optional) tried
after the item name in `function`, `struct_def`, and `enum_def`, plus an
optional `where_clause()` raw span before the body `{`. Omitted generics parse
to an empty `Vec` so every existing definition is unchanged.

### Phase 2 — codegen passthrough (PS-08, emit half)
Codegen emits `<T, 'a, const N: usize>` after the item name and the `where …`
clause before the body, verbatim from the captured data. A definition with no
generic params emits exactly what it does today (empty list → no `<>`). This is
pure syntactic lowering; no type reasoning.

### Phase 3 — SEM-03 (explicit-generic call return typing)
When an `ExplicitGenericCall` / `TurbofishCall` names a method whose recorded
`MethodSig.return_type` (SEM-00) is exactly one of the method's own generic
parameters, substitute the corresponding supplied type argument and return it;
otherwise return `Type::Auto`. This stops *discarding* the type args in the one
case the AST makes decidable (a method returning `T` called with `G` yields
`G`). Anything more (bounds satisfaction, nested generic returns, inference
from arguments) stays `Auto` for rustc. Requires Phase 1 so a method's own
generic params are recorded on its `MethodSig`.

### Phase 4 — SEM-04 (array-literal ↔ `Vec<T>` and argument inference)
Two decidable inferences only:
1. An array literal `[a, b, c]` assigned/coerced to a declared `Vec<T>` (or
   `Type::Generic { base: Vec, args: [T] }`) type-checks element-by-element
   against `T` instead of erroring on the `Array` vs `Generic` mismatch.
2. A generic argument that appears in a parameter type is inferred from the
   corresponding call argument's type when that argument types to a concrete
   (non-`Auto`) type; the inferred binding is used only to type this call's
   result. Unbound or conflicting inferences fall back to `Auto`.
Everything else — HM-style unification, trait-driven inference, inference
across statements — is out of scope and defers to rustc.

### Scope limits (explicit non-goals)
No trait-bound checking (bounds are raw passthrough), no lifetime inference or
checking (lifetimes are raw tokens), no const-generic evaluation, no
monomorphisation, no variance or coherence. Const-generic *values* and
where-clause *contents* are opaque spans. Each is rustc's job; Crusty only
carries the syntax and makes the two Phase-3/4 decidable substitutions.

Alternatives considered (recorded so they are not re-proposed): modelling trait
bounds as a structured `Bound` AST with a known-trait table — rejected, it is a
second type system Crusty does not need and ADR-0001 forbids; raw bound strings
carry to rustc losslessly. Doing SEM-03/SEM-04 without Phase 1 — rejected,
both need a definition's declared generic params to know which type is a
parameter versus a concrete type.

## Consequences
- **Easier:** generic data structures and functions (`struct Stack<T>`,
  `T max<T>(...)`) become writable in Crusty and lower to idiomatic Rust;
  `@Vec(int).new()`-style calls can get a real element type (SEM-03); an array
  literal can initialise a declared `Vec<T>` (SEM-04).
- **Harder / accepted cost:** three AST structs and their every construction
  site (incl. the large test corpus) gain `generic_params` + `where_clause` —
  the biggest construction-site churn of any item so far; mitigate by giving
  the fields sensible `Default`-like empty values at every call. Raw bound /
  where strings mean a malformed bound is only caught by rustc, not Crusty —
  accepted, consistent with every other passthrough.
- **Neutral / deferred:** the full generic type system is explicitly NOT built;
  this ADR is the syntactic carrier plus two local inferences. A later ADR may
  revisit structured bounds if a concrete need appears.

## Verification
- [ ] `Function`, `Struct`, `Enum` carry `generic_params: Vec<GenericParam>`
      and `where_clause: Option<String>`; `GenericParam` has `Type`/`Lifetime`/
      `Const` variants with raw-string bounds.
- [ ] Parser accepts `struct Stack<T> { … }`, `T first<T>(T[] xs)`, lifetimes
      (`<'a>`), const generics (`<const N: usize>`), and a `where T: Clone`
      clause; every existing (non-generic) definition still parses with an
      empty `generic_params`.
- [ ] Codegen round-trips each form to valid Rust: a generic struct/fn/enum
      transpiles, compiles under `rustc -D unused_parens`, and the generic
      params + where clause appear verbatim.
- [ ] SEM-03: a method declared to return its own type parameter `T`, called
      with an explicit type argument `G`, types the call result as `G`; a
      method returning a concrete type is unaffected; anything undecidable is
      `Auto`.
- [ ] SEM-04: an array literal assigned to a declared `Vec<T>` type-checks its
      elements against `T` instead of erroring; a generic parameter is inferred
      from a concrete call-argument type; unbound/conflicting cases are `Auto`.
- [ ] `cargo test --lib --bins` and `cargo clippy --lib --bins` pass with 0
      failures and 0 warnings after each phase.
- [ ] An end-to-end generic Crusty program (e.g. a `Stack<T>` with push/pop, or
      a generic `max<T>`) transpiles, compiles under `rustc -D unused_parens`,
      and runs.
