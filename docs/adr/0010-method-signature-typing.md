# ADR-0010: Method-signature typing (SEM-00)

## Status
accepted

## Context
ADR-0009 moved SEM-01 (`MethodCall` return typing) and SEM-02 (`TypeScopedCall`
real return type) out of Tier 2 into the design-first tier, each gated on a
prerequisite it named **SEM-00: method-signature typing**. This ADR is that
prerequisite. It is a design decision because the type environment holds no
method signatures today, and resolving `x.foo()` versus `Type::m()` is a
type-system choice, not a stub-finish.

What the code holds today (read in `src/semantic.rs` / `src/ast.rs`, not
speculation):

- `TypeKind::Struct` stores only `{ fields: Vec<(String, Type)> }`. It records
  nothing about methods.
- `analyze_struct` registers that `TypeKind::Struct`, then analyses each
  `struct_def.methods` entry through `analyze_function` — which inserts the
  method into the symbol table under its **bare name** (`new`, not
  `Point::new`). Methods are never keyed by their owning type.
- The full method signature nevertheless already exists in the AST:
  `Struct.methods: Vec<Function>`, and each `Function` carries
  `params: Vec<Param>` and `return_type: Option<Type>`. The data is present;
  it is simply discarded at registration time.
- `Param` is `{ name, ty }`. An instance method is distinguished only by its
  **first parameter being named `self`** — this is the convention codegen
  already relies on (`src/codegen.rs`: `if param.name.name == "self"`). There
  is no `self` AST marker.
- Current return typing of the two call forms:
  - `Expression::MethodCall` → `Type::Auto` (unconditional).
  - `Expression::TypeScopedCall` → echoes the scope `ty` (so `Vec::new()` is
    typed `Vec`, which is wrong — `new` may return anything).

Constraint from ADR-0001 / `docs/design/philosophy.md`: Crusty is a **syntax
layer over Rust, and rustc is the arbiter of semantics**. SEM-00 must therefore
improve *error locality and inferred-type propagation for Crusty's own code*
without reimplementing rustc's trait resolution, method lookup, autoref/deref,
or generics. It types what Crusty can see from the AST and defers everything
else to `Type::Auto` (pass-through), exactly as SEM-05/06/07/08 do.

## Decision
We will record method signatures in the type environment, keyed by owning type,
and use them to type `MethodCall` and `TypeScopedCall` — resolving only what the
AST determines locally and falling back to `Type::Auto` otherwise.

1. **Data.** Extend `TypeKind::Struct` to also carry methods:

   ```rust
   Struct {
       fields: Vec<(String, Type)>,
       methods: Vec<MethodSig>,
   }
   ```

   with a new sibling type in `semantic.rs`:

   ```rust
   pub struct MethodSig {
       pub name: String,
       pub params: Vec<Type>,     // NON-self params only, in order
       pub return_type: Type,     // Type::Auto when the source return is void/omitted-and-unknown
       pub is_instance: bool,     // true when the first AST param is named `self`
   }
   ```

   `methods` is the only new field; `fields` is unchanged so SEM-05/FieldAccess
   keep working untouched. A struct with no methods carries an empty `Vec`.

2. **Registration.** In `analyze_struct`, build one `MethodSig` per
   `struct_def.methods` entry before registering the `TypeInfo`:
   `is_instance` = the method's first param is named `self`; `params` = the
   remaining params' types in order; `return_type` = `return_type.clone()`
   when present else `Type::Auto`. Methods continue to be analysed through
   `analyze_function` as today (bodies still checked); this step only
   additionally *records* their signatures. Enums and aliases are unchanged.

3. **SEM-01 — `MethodCall` (`recv.m(args)`).** Resolve the receiver type; if it
   resolves to a `Type::Ident` naming a registered `TypeKind::Struct`, look up an
   `is_instance` method named `m`:
   - found → return its `return_type` (which may itself be `Type::Auto`);
   - a known struct receiver with no such instance method → push one
     `SemanticErrorKind::InvalidOperation` ("no method `m` on type `T`"), return
     `Type::Auto`;
   - receiver type is `Type::Auto`, a reference/pointer to a struct, a generic,
     or any non-struct/unknown type → return `Type::Auto`, no error (rustc
     arbitrates: autoref/deref, trait methods, and inherent methods on built-in
     types are out of scope). Arguments are still analysed for their own errors.

4. **SEM-02 — `TypeScopedCall` (`Type::m(args)`, i.e. Crusty `@Type.m(args)`).**
   If `Type` resolves to a registered `TypeKind::Struct`, look up a method named
   `m` (either `is_instance` or associated — a `Type::m()` call names it by path
   regardless) and return its `return_type`. Stop echoing the scope type. When
   the type is unknown, or no such method is recorded, return `Type::Auto`
   (not the scope type), no error. `TurbofishCall` keeps echoing the scope type
   for now; retyping it is out of scope (its generics make the real return
   type undecidable from the AST alone).

5. **Scope limits (explicit non-goals).** No trait/`impl`-block methods (none
   exist yet — PS-09), no methods on built-in or `__rust__`-defined types, no
   autoref/deref or receiver-coercion reasoning, no generic-return
   instantiation, no overload resolution by argument type (match on name only;
   arity/type mismatches are left to rustc). Every one of these resolves to
   `Type::Auto` pass-through, never an error.

Alternatives considered (recorded so they are not re-proposed): a separate
`HashMap<(String, String), MethodSig>` method table keyed by `(type, method)`
— rejected because methods belong to their owning type and `TypeKind::Struct`
is where every other per-type fact already lives, so a side table would be a
second source of truth to keep in sync. A `self` marker on `Param` — rejected
as a larger AST change than SEM-00 needs; the first-param-named-`self`
convention is already load-bearing in codegen, so SEM-00 reuses it.

## Consequences
- **Easier:** `let v = p.scaled();` and `let n = Counter::make();` get a real
  inferred type instead of `Auto`, so a downstream type error surfaces at the
  Crusty call site instead of only in rustc's output. SEM-01/SEM-02 become
  straightforward lookups once the signatures are recorded.
- **Harder / accepted cost:** `TypeKind::Struct` gains a field, so every match
  on it and every `TypeInfo::new(... Struct { .. })` construction site (incl.
  tests) must add `methods`. A method call whose receiver type Crusty cannot
  resolve still types `Auto` — SEM-00 narrows, but does not close, the gap; that
  is deliberate (rustc remains the arbiter). The name-only lookup can type a
  call that rustc will later reject on arity/argument grounds; acceptable,
  because Crusty does not claim to type-check calls, only to propagate the
  declared return type when it can see one.
- **Neutral:** codegen is untouched — this is a semantic-analysis-only change.

## Verification
- [ ] `TypeKind::Struct` has a `methods: Vec<MethodSig>` field and `MethodSig`
      is defined with `{ name, params, return_type, is_instance }`.
- [ ] `analyze_struct` populates `methods`: `is_instance` is true iff the
      method's first param is named `self`, `params` excludes that `self`, and
      `return_type` is `Type::Auto` for a void/omitted return.
- [ ] A unit test: a struct `Point` with an instance method `scaled(int) -> Point`
      makes `p.scaled(2)` on a `Point`-typed receiver analyse to `Type::Ident("Point")`.
- [ ] A unit test: calling a method that does not exist on a known struct
      receiver pushes exactly one `InvalidOperation` error.
- [ ] A unit test: `MethodCall` on an `Auto`-typed or non-struct receiver
      returns `Type::Auto` and pushes no error.
- [ ] A unit test: `TypeScopedCall` for a recorded associated method returns
      that method's declared return type, and an unknown type/method returns
      `Type::Auto` (NOT the scope type) with no error.
- [ ] `cargo test --lib --bins` and `cargo clippy --lib --bins` pass with 0
      failures and 0 warnings.
- [ ] An end-to-end Crusty program whose only observable behaviour depends on a
      method's inferred return type transpiles, compiles under
      `rustc -D unused_parens`, and runs.
