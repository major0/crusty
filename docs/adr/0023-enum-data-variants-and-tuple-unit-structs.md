# ADR-0023: enum data variants and tuple / unit structs

## Status
proposed

## Context

Crusty's aggregate types are strictly C-shaped today:

- **Enums** are discriminant-only. `EnumVariant` is `{ name: Ident, value: Option<i64> }`
  — a name and an optional integer value, exactly C's `enum { A, B = 2 }`. There
  is no way to attach data to a variant.
- **Structs** require a braced, named-field body. `Struct.fields: Vec<Field>`
  where `Field` is `{ visibility, name, ty, … }` — the C `struct { int x; }`
  shape. There is no unnamed-positional or field-less form.

This means the entire Rust **sum-type / newtype** vocabulary has no surface and
must be written in a `__rust__{}` block:

```c
// none of these parse today
enum Shape { Circle(f64), Rect { f64 w; f64 h; }, Unit }
typedef struct(int, int) Point;     // tuple struct
typedef struct Marker;              // unit struct
```

Verified against the current binary — `enum E { A(int), B }`,
`typedef struct(int,int) Point;` and a unit struct are all parse errors. Because
data-carrying enums and newtype wrappers are how idiomatic Rust models `Result`,
`Option`, state machines and typed scalars, their absence is the single largest
driver of the `__rust__` fallback rate: a C program interfacing a real crate hits
`Result`/`Option` immediately and must drop to `__rust__` to construct or match
them.

C itself has **no** enum-with-payload and no tuple/unit struct, so there is no C
idiom to inherit — this is a place where Crusty must choose a Rust-facing surface
rather than mirror C. That choice (and the fact it touches two widely-constructed
AST nodes) is why this is an ADR.

Crusty is a syntax layer (ADR-0001): these forms lower to the *exact* Rust
construct and inherit its semantics. Crusty does not type-check variant payloads
or newtype fields beyond what it already does for named struct fields; rustc
arbitrates.

## Decision

Add three data-carrying aggregate shapes, each lowering to its direct Rust
counterpart. The surface reuses existing grammar wherever a shape already exists.

### 1. Enum data variants

A variant may carry a **tuple payload** `(T, …)` or a **struct payload**
`{ T name; … }` (the braced body reuses the existing struct-field grammar),
in addition to today's bare / `= value` forms:

```c
enum Shape {
    Unit,                       // field-less (today)
    Code = 7,                   // discriminant (today)
    Circle(f64),                // tuple variant         (new)
    Rect { f64 w; f64 h; },     // struct variant        (new)
}
```

lowers to:

```rust
pub enum Shape {
    Unit,
    Code = 7,
    Circle(f64),
    Rect { w: f64, h: f64 },
}
```

A single variant carries exactly one of: nothing, `= <int>`, a tuple payload, or
a struct payload. A discriminant `= value` and a payload are mutually exclusive
(Rust forbids combining them without `#[repr]`-level machinery Crusty does not
model); `EnumVariant` makes that unrepresentable.

### 2. Tuple structs

```c
typedef struct(int, int) Point;
typedef struct(f64) Meters;        // newtype
```

lowers to `pub struct Point(i32, i32);` and `pub struct Meters(f64);`.

### 3. Unit structs

```c
typedef struct Marker;
```

lowers to `pub struct Marker;`.

### AST

`EnumVariant` gains a `payload`, replacing the bare `value` with a body enum so
the three data shapes and the discriminant are mutually exclusive by
construction:

```rust
pub struct EnumVariant {
    pub name: Ident,
    pub body: VariantBody,
}
pub enum VariantBody {
    Unit,                       // `A`
    Discriminant(i64),          // `A = 7`
    Tuple(Vec<Type>),           // `A(T, …)`
    Struct(Vec<Field>),         // `A { T name; … }`
}
```

`Struct` gains a `body` describing its shape, leaving the common named-field case
as the default:

```rust
pub enum StructBody {
    Named(Vec<Field>),          // `struct { T name; … }` (today)
    Tuple(Vec<Type>),           // `struct(T, …)`
    Unit,                       // `struct;`
}
```

`Struct.fields: Vec<Field>` is replaced by `Struct.body: StructBody`; the
`Named` case carries today's `Vec<Field>`.

### Lowering

- Tuple variant / tuple struct: emit `Name(T0, T1, …)` — the positional type
  list verbatim, no field names.
- Struct variant: emit `Name { n0: T0, n1: T1, … }`, reusing the named-field
  emitter already used for struct bodies.
- Unit struct: emit `Name;`. Field-less enum variant and `= value` are
  unchanged.

### Not in scope

- **Generic payloads beyond what the type grammar already parses.** A payload
  type is an ordinary `Type`, so `Circle(Vec<f64>)` works for free; no new type
  machinery.
- **Pattern-matching / construction ergonomics.** Constructing `Circle(1.0)` and
  matching it are the existing call-expression and `match` surfaces (ADR's
  Match), which already pass through to rustc. This ADR adds the *type
  declaration* surface only.
- **Visibility per tuple field** (`pub` on an individual positional field). Tuple
  fields inherit the struct's visibility, matching the common case; a per-field
  modifier can be a later additive change if a real need appears.

## Consequences

**Easier.**
- `Result`/`Option`-shaped enums, state machines, and newtype wrappers get a
  native surface — the biggest single reduction in forced `__rust__` usage for a
  program modelling real data.
- The surface reuses existing grammar: struct-variant bodies are the struct-field
  rule; payload types are ordinary `Type`s; tuple payloads are a type list.

**Harder / accepted costs.**
- `EnumVariant` and `Struct` are **reshaped** (not merely extended):
  `EnumVariant.value: Option<i64>` becomes `body: VariantBody`, and
  `Struct.fields: Vec<Field>` becomes `body: StructBody`. This ripples to every
  construction and destructuring site — `cargo test --no-run` enumerates them
  (AST helpers, parser, codegen, and several semantic/codegen test modules).
  Bounded and mechanical, but a one-way door, which is why it is an ADR.
- The semantic analyzer registers each struct's field set; a tuple struct has
  positional fields and a unit struct none, so field-name lookups must tolerate
  the non-`Named` shapes (field access on a tuple struct is `.0`/`.1`, which is
  already the index/tuple path, not named-field access).
- Two new mutually-exclusive-by-construction invariants (variant body, struct
  body) replace looser `Option` fields — more enum arms in exhaustive matches,
  but illegal states (a variant with both a discriminant and a payload) become
  unrepresentable rather than runtime-checked.

**Neutral.**
- Named structs and discriminant/bare enum variants emit byte-identical Rust to
  today (the `Named` and `Unit`/`Discriminant` cases).
- No new reserved keywords: `struct` already opens the typedef-struct form;
  tuple/unit shapes are distinguished by what follows the name (`(` → tuple,
  `;` → unit, `{`/field-body → named).

## Verification

- [ ] `EnumVariant` is `{ name, body: VariantBody }`; `VariantBody` is `Unit | Discriminant(i64) | Tuple(Vec<Type>) | Struct(Vec<Field>)`.
- [ ] `Struct` carries `body: StructBody` (`Named(Vec<Field>) | Tuple(Vec<Type>) | Unit`); the named case is unchanged output.
- [ ] `enum E { A(int), B { int x; }, C, D = 4 }` parses to a Tuple, a Struct, a Unit and a Discriminant variant respectively.
- [ ] `typedef struct(int, int) Point;` parses to a Tuple struct; `typedef struct Marker;` parses to a Unit struct.
- [ ] Codegen: tuple variant/struct emit `Name(T, …)`; struct variant emits `Name { n: T, … }`; unit struct emits `Name;`.
- [ ] A bare named struct and a bare/`= value` enum variant emit byte-identical Rust to pre-ADR output.
- [ ] End-to-end under `rustc -D unused_parens`: an enum mixing all four variant shapes plus a tuple struct and a unit struct compiles, and a value of each is constructed and matched (construction/match via the existing call + `match` surfaces).
- [ ] A variant cannot carry both a discriminant and a payload (unrepresentable in the AST; grammar rejects `A(int) = 3`).
- [ ] Full suite green (`cargo test`), clippy clean.
- [ ] `docs/syntax/` documents the three new shapes (enum data variants, tuple struct, unit struct) with their lowerings.

## Alternatives considered

- **Add a separate `Item::TupleStruct` / `Item::UnitStruct` instead of a
  `StructBody` on the existing `Struct`.** Rejected: a tuple/unit struct shares
  everything else with a named struct (visibility, generics, where-clause,
  attributes, methods via `impl`). A parallel item type would duplicate all of
  that and fork the struct codegen; a `body` enum keeps one struct node with one
  shape axis (smart data).
- **Keep `EnumVariant.value: Option<i64>` and add a separate
  `payload: Option<…>`.** Rejected: that permits the illegal `A(int) = 3` state
  (both set) and forces every reader to check two fields. A single `VariantBody`
  makes the four shapes mutually exclusive by construction.
- **Tuple-variant-only (skip struct variants).** Rejected: struct variants cost
  almost nothing — the body is the existing struct-field grammar and the existing
  named-field emitter — and omitting them would send `Rect { w, h }`-style
  variants back to `__rust__`, leaving the sum-type gap half-open.
- **Invent a C-flavoured payload syntax (e.g. `A: int`).** Rejected: C has no
  enum payload to mirror, and the Rust forms (`A(T)`, `A { T n; }`) are what the
  variant lowers to — using the target syntax keeps the thin-layer mapping
  one-to-one and avoids a translation nobody expects.
- **Per-field `pub` on tuple fields now.** Deferred (not rejected): it is a
  further additive change on `StructBody::Tuple`/`VariantBody::Tuple` if a real
  need appears; the common case is whole-struct visibility.
