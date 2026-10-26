# ADR-0024: tuple and unit structs

## Status
accepted

## Context

Split from ADR-0023 (which proposed enum data variants and tuple/unit structs
together). The struct half is implemented first: it is the smaller, self-contained
reshape and establishes the `StructBody` shape-axis pattern that ADR-0023's
`VariantBody` then mirrors.

Crusty structs require a braced, named-field body. `Struct.fields: Vec<Field>`
where `Field` is `{ visibility, name, ty, … }` — the C `struct { int x; }` shape.
There is no unnamed-positional (tuple) or field-less (unit) form, so Rust newtypes
and marker types have no surface and must be written in `__rust__{}`. Verified
against the current binary: `struct Point(int, int);` and `struct Marker;` are
parse errors.

C has no tuple or unit struct, so there is no C idiom to inherit — Crusty adopts
the Rust-facing surface by extending its existing struct-definition form
(`struct Name { … }`, the `struct_def` rule → `Item::Struct`). Crusty is a syntax
layer (ADR-0001): these lower to the exact Rust struct and inherit its semantics.
(Note: `typedef struct { … } @Name` is a different surface — the impl-block form,
ADR-0009 — and is untouched here.)

## Decision

Add two struct shapes alongside today's named-field form, distinguished by what
follows the struct name in the `struct Name …` definition:

### Tuple structs

```c
struct Point(int, int);
struct Meters(f64);        // newtype
```

lower to `pub struct Point(i32, i32);` and `pub struct Meters(f64);`.

### Unit structs

```c
struct Marker;
```

lowers to `pub struct Marker;`.

### AST

`Struct.fields: Vec<Field>` is replaced by `Struct.body: StructBody`, a shape axis
on the one struct node:

```rust
pub enum StructBody {
    Named(Vec<Field>),          // `struct { T name; … }` (today)
    Tuple(Vec<Type>),           // `struct(T, …)`
    Unit,                       // `struct;`
}
```

Everything else on `Struct` (visibility, generics, where-clause, attributes,
methods) is unchanged and applies to all three shapes.

### Lowering

- Tuple struct: emit `Name(T0, T1, …)` — the positional type list, no field names,
  terminated with `;`.
- Unit struct: emit `Name;`.
- Named struct: unchanged (`Named(fields)`), byte-identical output.

### Not in scope

- **Per-field `pub` on a tuple field.** Tuple fields inherit the struct's
  visibility (the common case); a per-field modifier is a later additive change on
  `StructBody::Tuple` if a real need appears.
- **Enum data variants** — ADR-0023, built on this `StructBody` precedent.

## Consequences

**Easier.** Newtype wrappers (`struct Meters(f64)`) and marker types get a native
surface; field access on a tuple struct is the existing `.0`/`.1` index path.

**Harder / accepted costs.** `Struct.fields: Vec<Field>` becomes
`body: StructBody` — a reshape rippling to every construction/destructuring site.
`cargo test --no-run` enumerates them (AST helpers, parser, codegen, and the
semantic/codegen test modules that build a `Struct`). Bounded and mechanical; a
one-way door, hence an ADR. The semantic analyzer registers a struct's fields —
tuple structs register positional fields and unit structs none, so named-field
lookup must tolerate the non-`Named` shapes.

A tuple struct's name is also a **constructor**: calling it (`Point(3, 4)`) is a
call whose callee resolves to the struct's own `Type::Ident`, which the analyzer
previously rejected as "cannot call non-function type". The Call arm now permits a
callee that names a known type, typing the result as that type and deferring the
positional arity/field-type check to rustc (ADR-0001 relaxation, same spirit as
the `Auto`/`Ok`/`Err` callee relaxations). Locked by a regression test.

**Neutral.** Named structs emit byte-identical Rust (`Named` case). No new
keywords — `struct Name` already opens the definition; `(` after the name → tuple,
`;` → unit, a field body → named.

## Verification

- [x] `Struct` carries `body: StructBody` (`Named(Vec<Field>) | Tuple(Vec<Type>) | Unit`); the named case is unchanged output.
- [x] `struct Point(int, int);` parses to a Tuple struct with two positional types.
- [x] `struct Marker;` parses to a Unit struct.
- [x] Codegen: tuple struct emits `Name(T, …);`; unit struct emits `Name;`.
- [x] A bare named struct emits byte-identical Rust to pre-ADR output.
- [x] End-to-end under `rustc -D unused_parens`: a program defining a tuple struct and a unit struct compiles, constructs a value of each (`Point(1, 2)`, `Marker`), and reads a tuple field (`.0`).
- [x] Full suite green (`cargo test`), clippy clean.
- [x] `docs/syntax/` documents tuple and unit structs with their lowerings.

## Alternatives considered

- **Separate `Item::TupleStruct` / `Item::UnitStruct` item types.** Rejected: a
  tuple/unit struct shares visibility, generics, where-clause, attributes and
  `impl` methods with a named struct. Parallel item types would duplicate all of
  that and fork the struct codegen; a `body` shape-axis keeps one struct node.
- **Keep `fields: Vec<Field>` and add an `is_tuple` / `is_unit` flag pair.**
  Rejected: the flags permit nonsense combinations (tuple *and* unit) and a tuple
  struct's positional types are not `Field`s (no names). `StructBody` carries the
  right data per shape and makes illegal states unrepresentable.
- **Represent a tuple struct as a named struct with synthesised names `_0`,
  `_1`.** Rejected: that emits `struct Point { _0: i32 }` (a named struct), not a
  tuple struct, breaking `.0` access and tuple-struct construction `Point(1, 2)`.
  The surface must lower to the actual Rust tuple struct.
