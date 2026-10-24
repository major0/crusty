# ADR-0015: trait / impl / unsafe / async (PS-09)

## Status
accepted

## Context
ADR-0009's Tier 4 lists **PS-09** as "`trait`/`impl` items; `unsafe`; `async`"
— the last and largest grammar-completion bundle. Unlike PS-06/PS-07, these are
four loosely-related Rust features of very different size and risk, so this ADR
is primarily a **scoping and sequencing** decision: what each costs, which are
worth doing on the syntax layer, and the order.

What the code holds today (read in `src/parser.rs`, `src/codegen.rs`,
`src/ast.rs`, not speculation):

- **`unsafe` is reserved but unused.** `kw_unsafe()` exists and is in the
  `keyword()` set, so `unsafe` cannot be an identifier — but no grammar rule
  consumes it. There is no `unsafe` block or `unsafe fn`.
- **`impl` already emits — for struct methods only — and a C-style surface for
  it is already DESIGNED but unimplemented.** `generate_struct` emits
  `impl Name<…> where … { fn … }` for a struct's own methods (ADR-0010/SEM-00).
  Separately, `docs/syntax/typedef.md` specifies a C-style impl-block surface —
  `typedef struct { methods } @Point;` → `impl Point { … }`,
  `typedef default { … } @Point;` → `impl Default for Point`, and
  `@Point.name` named blocks — but that surface has NO grammar and NO AST
  (the `@` code paths today are the unrelated expression-level `@Type.method()`
  calls of PS-14/15). This is ADR-0009's **DRIFT-1**: documented, not built.
  Crusty does NOT reserve `impl` as a keyword; the impl surface is `typedef`-
  based, so no `impl` keyword is needed.
- **`typedef default { … } @Point;` already DESIGNS a slice of trait impl** —
  implementing a named trait (`Default`) for a type — so trait-impl is not
  entirely greenfield even though Unit 3 (defining new traits) is deferred.
- **`trait`, `async`, `await` are not modelled and not even reserved.** They
  appear only in proptest keyword generators, not in the parser's `keyword()`
  set, so a Crusty program could currently use them as identifiers. No AST node,
  grammar, or codegen exists for any of them.
- **Methods, generics, and the `__rust__ { … }` item/expression escape hatch**
  all exist. A trait, an `impl Trait for Type`, an `unsafe` block, or an
  `async fn` can be written TODAY inside `__rust__ { … }` and used from Crusty —
  so none of PS-09 is a hard blocker; the question is which deserve first-class
  surface syntax.

The four features rank very differently against the house rule (simplest first;
a feature earns its surface):

1. **`unsafe` blocks / `unsafe fn`** — small, self-contained, and genuinely
   needed for FFI and raw-pointer work the C-compat goal invites. `unsafe` is
   already reserved.
2. **`impl` blocks (inherent)** — a user-written `impl Type { … }` block
   separate from the struct body. Medium. The emission path already exists
   (struct methods lower to exactly this), so it is largely surfacing.
3. **`trait` + `impl Trait for Type`** — large: a new item kind, trait-method
   signatures, default methods, and `impl … for …`. Trait-bound *checking* is
   explicitly rustc's job (ADR-0001), but the surface and lowering are
   substantial. Arguably its own ADR.
4. **`async` / `await`** — large and orthogonal: `async fn`, `.await`, and the
   fact that async is viral through signatures and needs a runtime to run. The
   syntax layer can lower `async`/`await` verbatim, but it buys little without
   runtime integration, and it is the least C-like of the four.

## Decision
Treat PS-09 as **four independent decisions**, sequenced simplest-first, each
its own implementable unit. This ADR commits firmly to the two cheap, high-value
ones and records the two large ones as **deferred to their own ADRs**, so PS-09
does not become one unreviewable mega-change.

### Unit 1 — `unsafe` blocks and `unsafe fn` (COMMIT)
- **Surface:** `unsafe { … }` as a statement/block, and `unsafe` as a function
  modifier (`unsafe int raw_read(...) { … }`). `unsafe` is already reserved.
- **Lowering:** verbatim to Rust `unsafe { … }` / `unsafe fn …`. Crusty does no
  unsafety analysis — rustc owns it (ADR-0001).
- **AST:** a `Statement::Unsafe(Block)` wrapper (mirroring `Statement::Attributed`)
  and an `is_unsafe` flag on `Function`. Additive.
- **Phase:** grammar+AST (parse-and-store), then codegen passthrough with an
  end-to-end rustc proof (an `unsafe` block dereferencing a raw pointer from a
  `__rust__` value).

### Unit 2 — `typedef … @Type` impl blocks (COMMIT; resolves DRIFT-1)
- **Surface:** the already-DESIGNED C-style impl-block surface from
  `typedef.md`, not a Rust-style `impl Type { }`:
  - `typedef struct { <methods> } @Point;` → `impl Point { … }`
  - `typedef struct { <methods> } @Point.name;` → a named organisational block,
    merged into one `impl Point` per type in codegen.
  - `typedef default { <methods> } @Point;` → `impl Default for Point` (the one
    trait-impl form this unit covers; general `trait`/`impl Trait for` stays
    Unit 3).
  No new `impl` keyword — the surface reuses `typedef` + the `@Type` target.
- **Lowering:** reuse the existing method-emission path (`generate_struct`
  already emits `impl Name { fn … }` for struct methods); generics/`where`
  reuse ADR-0011's helpers. The `default` form emits `impl Default for Name`.
- **AST:** a dedicated `Item::ImplBlock { target: Ident, trait_name:
  Option<Ident>, name: Option<Ident>, methods: Vec<Function> }` — `trait_name`
  is `Some("Default")` for the `default` form, `name` is the optional
  `.name`. A dedicated item (not an extension of `Struct`) because the `@Type`
  target and the trait form do not fit the struct-definition shape.
- **Phase:** grammar+AST (the `impl_block` / `typedef_target` productions already
  in `typedef.md`), then codegen (largely reuse), with a proof that a method
  defined in a `typedef … @Point;` block is callable, and that
  `typedef default { } @Point;` yields a working `Default` impl — then update
  `typedef.md` to mark the surface implemented and resolve DRIFT-1.

### Unit 3 — defining new `trait`s + general `impl Trait for Type` (DEFERRED; now ADR-0016)
- Needs a new `Item::Trait` (method signatures + optional default bodies) and a
  general `impl Trait for Type` form (Unit 2 covers only the `typedef default`
  → `impl Default for` case), plus associated-item surface. Trait-bound
  checking stays rustc's. This is a feature the size of PS-08 and deserves its
  own ADR with its own phases; it should NOT be bundled into PS-09's first
  landing. The `typedef default { } @Type;` form (Unit 2) is the template the
  general trait-impl surface will generalise.
- Until then, a trait and its impls are written in `__rust__ { … }`.

### Unit 4 — `async` / `await` (DEFER to ADR-00MM, lowest priority)
- `async fn` + `.await` can lower verbatim, but async is viral and inert without
  a runtime (tokio/async-std) that the syntax layer does not own, so it buys
  little today and is the least aligned with the C-compat goal. Defer until a
  concrete need and a runtime-integration decision exist. Writable in
  `__rust__ { … }` meanwhile.

### Future direction (NOT decided here) — `__thread__ { … }`
Recorded so the intent is not lost, not committed. A `__thread__ { … }` block —
double-underscore convention like `__rust__`/`__func__` — would spawn its body
to run concurrently on its own core, a go-routine-style primitive, as Crusty's
stylistic answer to concurrency rather than importing `async`/`await`. It is a
DIRECTION signal, not a solution to the missing Rust keywords, and it needs a
threading/runtime decision (which runtime, join/handle semantics, data-sharing
rules) before it can be specified — the same runtime gap that defers async
(Unit 4). It gets its own ADR if and when it is pursued; this note only reserves
the idea and the spelling.

### Non-goals (explicit)
- **No unsafety, trait-coherence, or send/sync reasoning.** rustc owns all of
  it (ADR-0001). Crusty's `unsafe` is a passthrough marker, not a checker.
- **No trait or async surface in this ADR** — Units 3 and 4 are deferred, not
  specified here.
- **No `unsafe` SEMANTIC effect** — marking a block `unsafe` does not let Crusty
  permit anything it otherwise rejects; it only emits the keyword for rustc.

## Consequences
- **Easier:** the two cheap wins (`unsafe`, the `typedef … @Type` impl surface)
  land first and unblock FFI/raw-pointer code and out-of-struct method
  organisation, reusing existing emission. Unit 2 also resolves DRIFT-1 by
  implementing a surface that was documented but unbuilt. PS-09 stops being a
  monolith.
- **Harder / accepted cost:** `trait` (defining new traits) and `async` remain
  `__rust__`-only until their ADRs. Unit 2 reuses `typedef`, so there is **no
  new reserved keyword** and no source-compat break — `impl` stays a plain
  identifier. The one semantic commitment is that `typedef default` is special-
  cased to `impl Default for`, which the general trait-impl (Unit 3) later
  generalises.
- **Reversible-cost:** Unit 1 is additive variants/flags; Unit 2 is a new
  additive `Item::ImplBlock` over an already-documented surface. The deferral of
  3 and 4 is the main judgement; it is recorded here so a future reader does not
  read PS-09 as "all four, now".

### Alternatives considered
- **One PS-09 ADR specifying all four.** Rejected: trait and async are each
  PS-08-sized; one ADR covering four features of this spread is unreviewable and
  violates one-decision-per-ADR. This ADR commits the cheap two and defers the
  big two by name.
- **A Rust-style `impl Type { }` item instead of the `typedef … @Type`
  surface.** Rejected: the C-style `typedef { } @Type` surface is already
  designed and documented (DRIFT-1), matches Crusty's C-flavoured philosophy,
  and needs no new `impl` keyword. Introducing a parallel Rust-style spelling
  would be two surfaces for one feature.
- **Do `trait` before `unsafe`/`impl`** because it is "more important".
  Rejected: the house rule is simplest-first; `unsafe` and the typedef impl
  surface are small, reuse existing paths, and deliver value immediately, while
  defining new `trait`s needs a design pass. Value-ordering a harder feature
  ahead buys nothing but risk.
- **Skip `unsafe` as a surface feature (keep it `__rust__`-only).** Rejected:
  raw-pointer / FFI work is squarely in the C-compat target, `unsafe` is already
  reserved, and the lowering is a one-line passthrough — it earns its surface.

## Verification

### Unit 1 — `unsafe`
- [x] `Statement::Unsafe(Block)` and `Function.is_unsafe` exist in `src/ast.rs`.
- [x] `unsafe { … }` and `unsafe int f(...) { … }` parse into those nodes
      (parser tests); `unsafe` remains rejected as an identifier.
- [x] Codegen emits `unsafe { … }` / `unsafe fn …`; non-unsafe output is
      byte-unchanged.
- [x] End-to-end: an `unsafe` block performing a raw-pointer deref (value from
      `__rust__`) transpiles, compiles under `rustc -D unused_parens`, and runs.

### Unit 2 — `typedef … @Type` impl blocks (resolves DRIFT-1)
- [x] `Item::ImplBlock { target, trait_name, name, methods }` exists.
- [x] `typedef struct { int get_x(self) { … } } @Point;` parses into an
      `ImplBlock` with `target=Point`, `trait_name=None`; `@Point.name` sets
      `name`; `typedef default { … } @Point;` sets `trait_name=Some(Default)`.
      No `impl` keyword is introduced (`impl` stays a valid identifier).
- [x] Codegen emits `impl Point { fn get_x(&self) -> i32 { … } }` (reusing the
      method emitter) and `impl Default for Point { … }` for the `default` form
      (with no `pub` on the trait method); a method defined in a
      `typedef … @Point;` block is callable end-to-end under rustc, and
      `typedef default` yields a working `Default` impl.
- [x] `docs/syntax/typedef.md` is updated to mark the impl-block surface
      implemented; DRIFT-1 is resolved in the roadmap.

### Both committed units
- [ ] `cargo test` green and `clippy` clean after each phase.

### Deferred
- [x] Unit 3 (`trait` / `impl Trait for`) is specified in its own ADR before
      implementation — ADR-0016 (proposed).
- [ ] Unit 4 (`async` / `await`) is specified in its own ADR before
      implementation.
