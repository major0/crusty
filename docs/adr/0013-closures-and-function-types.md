# ADR-0013: Closures and function-pointer types (PS-07, DRIFT-3)

## Status
proposed

## Context
ADR-0009's Tier 4 lists **PS-07** — closures/lambdas (`|x| expr`) and
function-pointer *types* — as a design-first feature, and ties **DRIFT-3**
(the function-pointer-type documentation) to it.

What the code holds today (read in `src/ast.rs`, `src/parser.rs`,
`src/codegen.rs`, `src/semantic.rs`, not speculation):

- **The function-pointer *type* already exists in the AST and lowers
  correctly, but cannot be parsed.** `Type::Function { params: Vec<Type>,
  return_type: Box<Type> }` is a `Type` variant; codegen emits it as
  `fn(<params>) -> <ret>` (a Rust fn-pointer type) and the semantic layer
  already handles it. But `type_expr` in `src/parser.rs` has **no arm that
  produces `Type::Function`** — so no Crusty source can declare a function type
  today. This is exactly DRIFT-3: the type is real everywhere except at the
  surface.
- **There is no closure anywhere.** No `Closure`/`Lambda` AST node, no `|…|`
  grammar, no closure-expression codegen. A `|x| expr` does not parse.
- **Rust closure *lowering* is partly precedented.** A nested function
  (`Statement::NestedFunction`) already lowers to a Rust closure binding with
  capture analysis (`collect_used_variables`, the capture tests). So the
  codegen layer already knows how to emit a closure and reason about captures —
  but only for the nested-function form, not for a closure *expression*.
- Crusty's own surface has no `fn` keyword (ADR philosophy) — functions are
  `int add(int a, int b)`. So a Crusty function *type* and closure *parameter*
  syntax must be chosen; they cannot simply mirror Rust's `fn`/`|…|` names
  without a decision.

So PS-07 is two separable pieces of very different size:
1. **Function-pointer types** — small: the type machinery exists; only a
   `type_expr` arm and a surface spelling are missing. Resolves DRIFT-3.
2. **Closure expressions** — larger: a new expression node, grammar, capture
   handling at the expression level, and typing.

## Decision
We will add both, as **two independent workstreams** (a function type is useful
on its own — e.g. a callback parameter — without closure *expressions*, and
closures can be typed `Auto` without a written function type). Each is phased
and independently committable. Closures are a surface and capture-semantics
decision; function types are a near-complete feature needing only a surface.

### Workstream A — function-pointer types (resolves DRIFT-3)
- **Surface:** adopt Rust's own `fn(T1, T2) -> R` spelling as the Crusty
  function-type syntax. It is unambiguous in a type position, it is exactly
  what `Type::Function` already lowers to, and it avoids inventing a second
  spelling for a type the compiler already emits. A return-less function type
  is `fn(T1, T2)` → `fn(T1, T2) -> ()`.
- **Phase A1 — grammar:** add a `type_expr` arm producing the existing
  `Type::Function { params, return_type }`. No AST change (the variant exists),
  no codegen change (emission exists). A function-type parameter
  (`void run(fn(int) -> int cb)`) then parses, lowers, and compiles.
- **Phase A2 — docs:** document the function-type surface in the types/function
  docs and resolve DRIFT-3.

### Workstream B — closure expressions
- **Surface:** adopt Rust's `|params| body` spelling. The body is an
  expression (`|x| x + 1`); a block body (`|x| { … }`) depends on the general
  block-expression Crusty still lacks (see Non-goals), so Phase B1 accepts the
  expression-body form only. Parameter types are optional (`|x|` or
  `|int x|`); a written return type is deferred (Non-goals).
- **Phase B1 — AST + grammar:** add
  `Expression::Closure { params: Vec<ClosureParam>, body: Box<Expression> }`
  with `ClosureParam { name: Ident, ty: Option<Type> }`, and a `closure_expr()`
  grammar rule in the expression atom tier. Parse-and-store only.
- **Phase B2 — codegen:** emit `|<params>| <body>`, reusing the type emitter
  for any written parameter type. The existing nested-function capture
  machinery is the reference; a closure expression captures by the same
  inference Rust applies, so **no explicit `move`/capture annotation is
  emitted** in B2 (rustc infers the capture) — this keeps B2 a passthrough.
- **Phase B3 — semantics:** a closure introduces its parameters (typed from
  their annotation, else `Auto`) into the body scope; the closure expression
  itself types as `Auto` (rustc infers the concrete `Fn`/`FnMut`/`Fnonce`),
  consistent with ADR-0001. The body is analysed so its own errors surface.
- **Phase B4 — docs:** document the closure surface and its capture/typing
  boundary.

### Non-goals (explicit)
- **No `move` closures, no explicit capture lists, no closure return-type
  syntax** in this ADR. rustc infers captures and the `Fn*` trait; a closure
  needing `move` or a written return type uses a `__rust__ { … }` expression.
- **No block-bodied closures** (`|x| { … }`) until Crusty has a general
  block-expression — the same limit ADR-0012 recorded for `match` arms. A
  closure that needs statements uses `__rust__ { … }` as its body.
- **No `Fn`/`FnMut`/`FnOnce` trait-bound reasoning.** The closure types `Auto`;
  rustc arbitrates which trait it satisfies. A written function *type*
  (Workstream A) is a concrete `fn` pointer, not a boxed `dyn Fn`; a closure
  passed where a `fn` pointer is expected is rustc's call to accept or reject.
- **No higher-ranked lifetimes / `for<'a>` function types.** Out of scope.

## Consequences
- **Easier:** Crusty gains callbacks and inline functions — a function-type
  parameter and a `|x| x + 1` argument — which many real C and Rust APIs need.
  Workstream A is nearly free (the type machinery already exists). Closure
  lowering reuses the nested-function capture precedent.
- **Harder / accepted cost:** `Expression::Closure` is a new, additive
  `Expression` variant (a `Match`-sized change: the exhaustive expression
  matches in codegen/semantic/pretty each gain one arm, no struct-field
  reshape, so the construction-site churn is small). The capture/trait
  inference is pushed to rustc, so a closure that captures incorrectly surfaces
  as a rustc error, not a Crusty one — the deliberate syntax-layer tradeoff.
- **Reversible-cost:** both workstreams are additive. Workstream A is a single
  grammar arm over an existing type. The one commitment awkward to revisit is
  the **surface spelling** — once `fn(T) -> R` and `|x| …` are documented and
  used, changing them breaks source; they are chosen to match Rust precisely so
  that is unlikely to be wanted.

### Alternatives considered
- **Invent a C-flavoured function-type spelling** (e.g. `R (*)(T1, T2)` as in
  C, or `R fn(T1, T2)`). Rejected: `Type::Function` already lowers to Rust's
  `fn(T) -> R`; matching that spelling at the surface means one syntax, no
  translation table, and no ambiguity in type position. The C pointer-to-
  function spelling is notoriously hard to read and parse.
- **One combined workstream.** Rejected: a function *type* is independently
  useful (callback parameters) and nearly complete, while closure *expressions*
  are a larger capture-semantics decision. Coupling them delays the cheap,
  drift-closing half behind the expensive half.
- **A structured capture list / `move` support now.** Rejected for this ADR:
  rustc infers captures, so the structured form has no reachable use until
  Crusty wants to diverge from Rust's inference — a separate decision. The
  `__rust__` escape covers `move` today.
- **Boxing closures as `Box<dyn Fn…>` for a uniform value type.** Rejected:
  it imposes allocation and a trait-object choice the syntax layer has no basis
  to make; `Auto` + rustc inference is the ADR-0001-consistent path.

## Verification

### Workstream A — function-pointer types
- [ ] `type_expr` parses `fn(int) -> int` and `fn(int, bool)` into
      `Type::Function { params, return_type }` (the return-less form defaulting
      its return to unit), proven by a parser unit test.
- [ ] A Crusty function with a function-type parameter
      (`void run(fn(int) -> int cb) { … }`) transpiles to
      `fn run(cb: fn(i32) -> i32)`, compiles under `rustc -D unused_parens`, and
      runs when called with a named function.
- [ ] The types/function docs describe the `fn(T) -> R` surface; DRIFT-3 is
      marked resolved in the roadmap.

### Workstream B — closure expressions
- [ ] `src/ast.rs` has `Expression::Closure { params, body }` and
      `ClosureParam { name, ty }`.
- [ ] `|x| x + 1` and `|int x| x + 1` parse into a `Closure` with the expected
      params (optional type), proven by a parser unit test.
- [ ] Codegen emits `|x| x + 1` (and `|x: i32| x + 1` when the type is
      written); a non-closure program's output is byte-unchanged.
- [ ] End-to-end: a program binding a closure and calling it (or passing it to
      a function-type parameter from Workstream A) transpiles, compiles under
      `rustc -D unused_parens` (RC=0), and runs with the expected output.
- [ ] A closure parameter is in scope in the closure body (a binding-scope
      semantic test); an unknown identifier in the body still errors.
- [ ] The docs describe the closure surface, the expression-body-only limit,
      and the capture/typing boundary (rustc infers captures and `Fn*`).

### Both
- [ ] `cargo test` green and `clippy` clean after each phase.
