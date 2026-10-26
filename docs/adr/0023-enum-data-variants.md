# ADR-0023: enum data variants

## Status
accepted

## Context

> Originally covered enum data variants **and** tuple/unit structs. The struct
> half was split into ADR-0024 (accepted and implemented first, because it is the
> smaller self-contained reshape and proves the `StructBody` pattern). This ADR
> now covers only the enum half and builds on ADR-0024's `VariantBody` precedent.

Crusty's enums are discriminant-only. `EnumVariant` is
`{ name: Ident, value: Option<i64> }` — a name and an optional integer, exactly
C's `enum { A, B = 2 }`. There is no way to attach data to a variant, so the Rust
sum-type vocabulary (`Result`-like, `Option`-like, state machines) has no surface
and must be written in a `__rust__{}` block. Verified against the current binary:
`enum E { A(int), B }` is a parse error.

C has no enum-with-payload, so there is no C idiom to inherit — Crusty adopts the
Rust-facing surface. Crusty is a syntax layer (ADR-0001): a data variant lowers to
the exact Rust variant and inherits its semantics; rustc arbitrates payload types.

## Decision

A variant may carry a **tuple payload** `(T, …)` or a **struct payload**
`{ T name; … }` (the braced body reuses the struct-field grammar), in addition to
today's bare / `= value` forms:

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

A single variant carries exactly one of: nothing, `= <int>`, a tuple payload, or a
struct payload. A discriminant and a payload are mutually exclusive (Rust forbids
combining them without `#[repr]` machinery Crusty does not model); the AST makes
that unrepresentable.

### AST

`EnumVariant` replaces the bare `value` with a body enum, mirroring ADR-0024's
`StructBody`:

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

### Lowering

- Tuple variant: emit `Name(T0, T1, …)` — the positional type list verbatim.
- Struct variant: emit `Name { n0: T0, … }`, reusing the named-field emitter.
- Unit / discriminant variant: unchanged.

### Not in scope

- **Construction / pattern-matching ergonomics.** Constructing `Circle(1.0)` and
  matching it are the existing call-expression and `match` surfaces, which pass
  through to rustc. This ADR adds the *variant declaration* surface only.

  > Follow-up (implemented after acceptance): a tuple variant constructs via the
  > existing `@Type.Variant(args)` call form, but a *struct-payload* variant had
  > no surface (it needed `__rust__{ Type::Variant { … } }`). A
  > `@Type.Variant { field: value, … }` construction form was added — a new
  > additive `Expression::StructVariantInit` lowering to `Type::Variant { … }`,
  > reusing the struct-initializer field + functional-update grammar. Matching is
  > still the raw `match` pattern passthrough.
- **Generic payloads beyond the type grammar.** A payload type is an ordinary
  `Type`, so `Circle(Vec<f64>)` works for free.

## Consequences

**Easier.** `Result`/`Option`-shaped enums and state machines get a native
surface — the largest remaining reduction in forced `__rust__` usage for data
modelling.

**Harder / accepted costs.** `EnumVariant.value: Option<i64>` becomes
`body: VariantBody` — a reshape rippling to every construction/destructuring site
(`cargo test --no-run` enumerates them: AST helpers, parser, codegen, semantic and
codegen test modules). Bounded and mechanical; a one-way door. The
`enum_variant_list` / `enum_variant_item` grammar returns richer tuples. Builds on
ADR-0024's already-landed `StructBody`, so the `Field`-body emitter is shared.

A **behavior change the rustc proof surfaced**: a bare variant (`A`) no longer
emits an auto-numbered `A = 0` discriminant — it emits the bare Rust variant `A`.
The old C-enum auto-numbering was invalid Rust (E0732) the moment the enum also
carried a payload variant, and Rust assigns the same sequential discriminant to a
bare variant anyway, so `as int` semantics are preserved. Only an explicit `A = n`
emits `= n`. This changes the emitted output of pure C-style enums (`Red = 0,` →
`Red,`), a cosmetic, semantically-identical change; the affected codegen/parser
tests were updated. A separate, unrelated defect was also observed (not fixed
here): a float literal `2.0` lowers to `2`, dropping the fractional part.

**Neutral.** A variant with an explicit `= value` emits byte-identical Rust (bare
variants changed — see the behavior-change note above). No new keywords — the
shape is distinguished by what follows the variant name (`(` → tuple, `{` →
struct, `=`/nothing → today).

## Verification

- [x] `EnumVariant` is `{ name, body: VariantBody }`; `VariantBody` is `Unit | Discriminant(i64) | Tuple(Vec<Type>) | Struct(Vec<Field>)`.
- [x] `enum E { A(int), B { int x; }, C, D = 4 }` parses to a Tuple, Struct, Unit and Discriminant variant.
- [x] Codegen: tuple variant emits `Name(T, …)`; struct variant emits `Name { n: T, … }`; unit/discriminant unchanged.
- [x] An explicit `= value` variant emits byte-identical Rust; a bare variant now emits bare (`Red,` not `Red = 0,`) — valid in data enums, same `as int` semantics.
- [x] End-to-end under `rustc -D unused_parens`: an enum mixing all four shapes compiles, and a value of a tuple and a struct variant is constructed and matched (output `25 12 0` / `7`).
- [x] A variant cannot carry both a discriminant and a payload (unrepresentable; the grammar's variant-item alternatives never combine `= n` with a payload).
- [x] Full suite green, clippy clean.
- [x] `docs/syntax/` documents enum data variants with their lowerings.

## Alternatives considered

- **Keep `value: Option<i64>` and add a separate `payload`.** Rejected: permits
  the illegal `A(int) = 3` state and forces two-field checks. A single
  `VariantBody` makes the shapes mutually exclusive by construction.
- **Tuple-variant-only (skip struct variants).** Rejected: struct variants reuse
  the struct-field grammar and named-field emitter at near-zero cost; omitting
  them leaves `Rect { w, h }`-style variants in `__rust__`.
- **Invent a C-flavoured payload syntax (`A: int`).** Rejected: no C idiom to
  mirror, and the Rust forms are what the variant lowers to — the target syntax
  keeps the mapping one-to-one.
