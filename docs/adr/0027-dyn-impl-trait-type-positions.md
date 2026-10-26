# ADR-0027: `dyn Trait` and `impl Trait` type positions

## Status
accepted

## Context

Crusty's type grammar has no trait-object (`dyn Trait`) or opaque-type
(`impl Trait`) form. Both are type-position constructs a programmer needs the
moment traits enter the picture — a `Box<dyn Error>` return, a `&dyn Write`
parameter, an `impl Iterator<Item = i32>` return. Verified against the current
binary: `void f(Box<dyn Error> e)` is a parse error.

Crusty does not model traits as types (a trait is a `typedef`-impl target, ADR-0009;
generic bounds and `where` clauses already ride verbatim strings to rustc,
ADR-0011). A trait-object or `impl Trait` type is therefore the same kind of thing
Crusty already defers: a Rust type Crusty has no structured model for, whose exact
text is valid Rust. This is the `Literal::Raw` / `Pattern::Raw` / verbatim-attribute
precedent applied to the *type* grammar.

The design questions are the surface (`dyn`/`impl` read as C — they are not C
keywords, but neither is the construct) and how the trait spec — which can carry
its own generics, associated-type bindings, and `+` bounds (`impl Iterator<Item = i32> + Clone`)
— is captured without modelling it.

## Decision

Add `dyn <trait-spec>` and `impl <trait-spec>` as type-position forms, captured
**verbatim** into a new additive `Type::Raw(String)` that codegen emits unchanged.

### Surface

```c
Box<dyn Error> boxed_error();          // trait object in a generic arg
void log(&dyn Write sink);             // trait object behind a reference
impl Iterator<Item = int> counter();   // impl Trait return
```

`dyn` / `impl` introduce the form; the **trait spec** that follows is captured as a
balanced, verbatim span up to the type's terminator (so `dyn Error`,
`impl Iterator<Item = int>`, `dyn Fn(int) -> int`, and `impl A + B` all capture
whole). The spec is not parsed into structure — it is the raw text handed to rustc.

`dyn` and `impl` are recognised only at the head of a type (a type-position
keyword), so a user identifier named `dynamic` or `implementation` is unaffected
(an `!ident_char()` guard after the keyword).

### AST

One additive variant — no reshape of `Type`:

```rust
Type::Raw(String),   // verbatim type text, e.g. "dyn Error", "impl Iterator<Item = i32>"
```

The stored string is the full type form including the `dyn` / `impl` keyword,
The stored string is the full type form including the `dyn` / `impl` keyword,
captured **verbatim** from source. The spec is NOT C-alias-normalised — it is
opaque passthrough text — so a primitive inside a spec must be written in Rust
spelling (`impl Iterator<Item = i32>`, not `int`), consistent with the `__rust__`
verbatim philosophy. Codegen emits the stored string unchanged.

### Lowering

`Type::Raw(s)` emits `s` unchanged. Because it composes with the existing type
operators (it is just another primary type in the precedence grammar),
`Box<dyn Error>` lowers through the generic-type arm with `dyn Error` as the
verbatim argument, `&dyn Write` through the reference arm, and an
`impl Trait` return through the function-return type position — all for free.

### Not in scope

- **Structured trait modelling.** Crusty does not type-check trait membership,
  object safety, or associated-type bindings; rustc arbitrates (ADR-0001). The
  `Raw` string is opaque to Crusty's semantic layer.
- **Lifetimes in the trait object** (`dyn Trait + 'a`). The `+ 'a` rides inside
  the verbatim spec if written, but Crusty has no lifetime grammar of its own;
  that is a separate gap.
- **`dyn`/`impl` anywhere other than a type position.** They are type-head
  keywords only.

## Consequences

**Easier.** Trait objects and `impl Trait` work in every type position that
composes from the type grammar (generic args, references, pointers, returns,
fields) with one additive variant and no per-position work. Reuses the verbatim
escape-hatch pattern the project already relies on (bounds, where-clauses,
attributes, raw literals/patterns).

**Harder / accepted costs.** One new `Type` variant ripples to the exhaustive type
matches — `generate_type_string` (codegen), the semantic type walker
(`resolve_type` / `is_compatible` / generic erasure), and any `Type` match in the
parser's type-inference helpers. Additive arms, not a reshape. The trait-spec
capture rule must balance `<...>` and `(...)` so a spec's own generics/`Fn(...)`
are captured whole and the capture stops at the enclosing type's delimiter.

**Neutral.** Every existing type is untouched (`Type::Raw` is new and only produced
by the `dyn`/`impl` arms). No new reserved identifier — `dyn`/`impl` are keywords
only at a type head, guarded by `!ident_char()`.

## Verification

- [x] `Type::Raw(String)` exists; `dyn Error` and `impl Iterator<Item = i32>` parse to it with the verbatim spec.
- [x] `Box<dyn Error>` parses (generic arg is a `Type::Raw`); `&dyn Write` parses (reference to a `Type::Raw`).
- [x] `impl Trait` in a function return type parses and emits in return position.
- [x] A trait spec with its own generics / `Fn(...)` / `+ Bound` is captured whole (`impl Iterator<Item = i32>`, `dyn Fn(i32) -> i32`, `impl A + B`).
- [x] Codegen emits `Type::Raw(s)` verbatim; `Box<dyn Error>` -> `Box<dyn Error>`, `impl Iterator<Item = i32>` -> itself.
- [x] A user type/identifier named `dynamic` / `implements` is still an ident type (the `!ident_char()` guard).
- [x] End-to-end under `rustc -D unused_parens`: a function returning `Box<dyn Display>` (and `impl Iterator`) and one taking `&dyn Display` compile and run (`sum=10` / `10` / `boxed=42`).
- [x] Semantic analysis treats a `Type::Raw` as opaque/`Auto`-compatible (no false type-mismatch), deferring to rustc.
- [x] Full suite green (`cargo test`), clippy clean.
- [x] `docs/syntax/` documents `dyn` / `impl Trait` type positions with their lowerings.

## Alternatives considered

- **Dedicated `Type::TraitObject { dyn_or_impl, spec }` structured variant.**
  Rejected: it models nothing Crusty uses — the spec is still opaque text handed
  to rustc — so a two-field struct buys only a tag over `Type::Raw(String)` while
  adding a second shape to every type match. The raw string is the honest model.
- **A fully general `Type::Raw` escape hatch for any unmodelled type (lifetimes,
  HRTB, `where`-bound types).** Tempting (one variant closes several gaps), but
  this ADR scopes the *surface* to `dyn`/`impl` heads so the grammar stays
  unambiguous; the `Type::Raw` variant itself is general and a later ADR can add
  more heads (e.g. a lifetime grammar) that produce it. We add the variant now and
  only the `dyn`/`impl` producers.
- **Model `dyn`/`impl` as a `Generic` over a synthetic `Trait` base.** Rejected:
  `dyn`/`impl` are not generic application; shoehorning them into `Generic` would
  emit the wrong Rust and conflate two unrelated type shapes.
- **Require `dyn`/`impl` constructs to go through `__rust__` type positions.**
  Rejected: `__rust__` is an item/expression escape hatch, not a type one, and
  forcing a `Box<dyn Error>` return through it would mean writing the whole
  signature in `__rust__` — exactly the fallback this ADR removes.
