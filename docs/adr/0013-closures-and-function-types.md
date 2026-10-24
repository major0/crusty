# ADR-0013: Anonymous functions and function-pointer types (PS-07, DRIFT-3)

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
  `int add(int a, int b)`. So a Crusty function *type* and anonymous-function
  syntax must be chosen; the anonymous form reuses Crusty's own function
  declaration grammar rather than importing Rust's `|…|`.

So PS-07 is two separable pieces of very different size:
1. **Function-pointer types** — small: the type machinery exists; only a
   `type_expr` arm and a surface spelling are missing. Resolves DRIFT-3.
2. **Anonymous functions** — larger: a new expression node and grammar, reusing
   the existing typed-param list, statement body, and nested-function closure
   lowering.

## Decision
We will add both, as **two independent workstreams** (a function type is useful
on its own — a callback parameter, or a struct field assigned a named function —
without anonymous functions, and an anonymous function can be typed `Auto`
without a written function type). Each is phased and independently committable.
Anonymous functions are a surface-and-capture decision; function types are a
near-complete feature needing only a surface.

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

### Workstream B — anonymous functions (`__func__`)
- **Surface:** an anonymous function is written as a **normal Crusty function
  declaration with the `__func__` keyword in the name slot**, in expression
  position: `[ret] __func__(typed-params) { body }`. It reuses the existing
  function syntax wholesale — the same typed parameter list
  (`function_params`), the same statement-block body, and the optional C-style
  return type before `__func__` (omitted ⇒ `void`/`()`). Examples:
  `int __func__(int x) { return x + 1; }`, `void __func__(int fd) { … }`.
  It produces a **callable value**, assigned or passed at its declaration site:
  `fops.read = int __func__(int fd, int n) { return fd + n; };`, called later
  as `fops.read(fd, n)`.
- **Why this spelling (not `|x| …`):** it introduces **no new parameter or
  body syntax** — `__func__` is literally a function minus its name — so it
  needs no block-expression (the body is already a statement block, unlike a
  `|x| expr` closure), has a natural slot for the return type, reads as native
  Crusty (`__rust__`/`__func__` share the double-underscore convention), and
  lowers through the **existing nested-function → closure path** (a Crusty
  nested function already emits `let name = |params| -> ret { body }`).
- **Phase B1 — AST + grammar:** add
  `Expression::AnonFunction { params: Vec<Param>, return_type: Option<Type>,
  body: Block }` — it reuses `Param` (typed, same as every function) and
  `Block`, not a new closure-param type. Add a `anon_function()` grammar rule
  in the expression atom tier, built from the existing `function_params` and
  `block` rules with `__func__` in the name position. Parse-and-store only.
- **Phase B2 — codegen:** emit `|<typed params>| -> <ret> { <body> }`, the same
  shape nested functions already emit. Captures are inferred by rustc, so **no
  explicit `move`/capture annotation is emitted** — a passthrough. Reuses the
  param-type and block emitters.
- **Phase B3 — semantics:** introduce the (typed) parameters into the body
  scope and analyse the body so its own errors surface; the anonymous function
  itself types as `Auto` (rustc infers the concrete `fn` pointer or `Fn*`
  trait), consistent with ADR-0001. This mirrors the existing nested-function
  capture analysis.
- **Phase B4 — docs:** document the `__func__` surface, that it is callable
  (not a run-once block), the non-capturing-into-`fn`-pointer rule, and the
  nested-function companion.

### Nested functions (companion, already implemented)
A named function declared inside another function body
(`Statement::NestedFunction`) already parses, lowers to a `let name = |…| { … }`
closure binding, and has capture analysis. PS-07 **ratifies** this as the
scoped-helper form and adds documentation + any missing regression tests; it
is not new implementation. Its two rules hold by construction of the lowering:
(1) a nested function is not visible outside its enclosing function (inner
scope), and (2) it is not usable before its own declaration (a `let` binding is
not in scope above itself — Rust does hoist a nested `fn`, but the closure
lowering deliberately does not).

### Non-goals (explicit)
- **No `move`, no explicit capture lists, no capturing anonymous function
  stored in a struct field.** A non-capturing `__func__` is a bare
  `fn(args) -> ret` pointer and can be stored in a function-pointer field
  (the `file_operations` case). A *capturing* `__func__` is not a bare `fn`
  pointer — storing one in a field needs `Box<dyn Fn…>`, a heavier allocation
  and trait-object decision deferred to a later ADR. A capturing function that
  needs `move` uses a `__rust__ { … }` expression today.
- **No untyped parameters.** `__func__` parameters are typed exactly like any
  Crusty function parameter (`int __func__(int x)`), so the grammar is the
  existing `function_params` rule unchanged — no inference path at the surface.
- **No `Fn`/`FnMut`/`FnOnce` trait-bound reasoning.** The anonymous function
  types `Auto`; rustc arbitrates the concrete `fn` pointer or `Fn*` trait. A
  written function *type* (Workstream A) is a concrete `fn` pointer, not a
  boxed `dyn Fn`; whether a given `__func__` satisfies it is rustc's call.
- **No higher-ranked lifetimes / `for<'a>` function types.** Out of scope.

## Consequences
- **Easier:** Crusty gains callbacks and inline functions — a function-type
  field/parameter (Workstream A) plus an `int __func__(int x) { … }` value
  (Workstream B) — covering the C `file_operations`-style pattern directly.
  Workstream A is nearly free (the type machinery already exists). The
  `__func__` form reuses the existing function grammar AND the nested-function
  closure lowering, so it is far smaller than a fresh closure feature.
- **Harder / accepted cost:** `Expression::AnonFunction` is a new, additive
  `Expression` variant (a `Match`-sized change: the exhaustive expression
  matches in codegen/semantic/pretty each gain one arm, no struct-field
  reshape, so the construction-site churn is small). Capture/trait inference is
  pushed to rustc, so a `__func__` that captures where a bare `fn` pointer is
  required surfaces as a rustc error, not a Crusty one — the deliberate
  syntax-layer tradeoff. A capturing `__func__` in a struct field is simply not
  expressible until the deferred `Box<dyn Fn>` decision.
- **Reversible-cost:** both workstreams are additive. Workstream A is a single
  grammar arm over an existing type. The surface spellings are the awkward-to-
  revisit commitments: `fn(T) -> R` for the type (chosen to match what the
  compiler already emits) and `[ret] __func__(params) { body }` for the value
  (chosen to reuse Crusty's own function syntax rather than import `|x|`).

### Alternatives considered
- **Rust's `|params| body` closure spelling (the first draft of this ADR).**
  Rejected in favour of `__func__`: `|x| expr` is new syntax with no return-type
  slot and an expression-only body (blocked by Crusty's missing
  block-expression), whereas `__func__` reuses the existing typed-param list and
  statement-block body, has a natural return-type slot, and needs no new body
  form.
- **Invent a C-flavoured function-type spelling** (`R (*)(T1, T2)`). Rejected:
  `Type::Function` already lowers to Rust's `fn(T) -> R`; matching that spelling
  means one syntax, no translation table, no ambiguity. The C pointer-to-
  function spelling is notoriously hard to read and parse.
- **One combined workstream.** Rejected: a function *type* is independently
  useful (callback parameters, named-fn assignment) and nearly complete, while
  anonymous functions are a larger decision. Coupling them delays the cheap,
  drift-closing half behind the expensive half.
- **Untyped `__func__` parameters inferred by rustc.** Rejected: it would make
  `__func__` diverge from normal Crusty function syntax (which requires typed
  params) for marginal ergonomic gain; typed params keep the grammar a direct
  reuse of `function_params`.
- **Capturing `__func__` as `Box<dyn Fn…>` for struct fields now.** Rejected
  for this ADR: it imposes allocation and a trait-object choice the syntax
  layer has no basis to make; the non-capturing → `fn`-pointer path covers the
  kernel case, and the boxed form is a separate decision.

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

### Workstream B — anonymous functions (`__func__`)
- [ ] `src/ast.rs` has `Expression::AnonFunction { params, return_type, body }`
      reusing `Param` and `Block`.
- [ ] `int __func__(int x) { return x + 1; }` and a return-less
      `void __func__(int fd) { … }` parse into an `AnonFunction` with the
      expected typed params and body, proven by a parser unit test; an untyped
      param (`__func__(x)`) is a parse error.
- [ ] Codegen emits `|x: i32| -> i32 { return x + 1; }` (and the unit-return
      form); a program with no anonymous function is byte-unchanged.
- [ ] End-to-end: a `file_operations`-style struct with a `fn(int,int) -> int`
      field (Workstream A), a `__func__` assigned to it, and a call through the
      field transpiles, compiles under `rustc -D unused_parens` (RC=0), and runs
      with the expected output.
- [ ] A `__func__` parameter is in scope in its body (a binding-scope semantic
      test); an unknown identifier in the body still errors.
- [ ] Nested functions: a regression test proves a named nested function lowers
      to a `let`-closure, is not in scope outside its enclosing function, and is
      not usable before its own declaration.
- [ ] The docs describe the `__func__` surface (callable, not a run-once
      block), the non-capturing-into-`fn`-pointer rule, and the nested-function
      companion.

### Both
- [ ] `cargo test` green and `clippy` clean after each phase.
