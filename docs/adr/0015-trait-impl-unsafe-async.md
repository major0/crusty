# ADR-0015: trait / impl / unsafe / async (PS-09)

## Status
proposed

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
- **`impl` already emits — for struct methods only.** `generate_struct` emits
  `impl Name<…> where … { fn … }` for a struct's own methods (ADR-0010/SEM-00).
  There is NO user-written `impl` block and NO trait `impl`. `impl` is NOT a
  reserved keyword in the real grammar.
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

### Unit 2 — inherent `impl` blocks (COMMIT)
- **Surface:** a free-standing `impl Type { <methods> }` item, letting methods
  be written outside the struct body (as Rust allows). `impl` becomes a
  reserved keyword.
- **Lowering:** the method-emission path already produces `impl Name { fn … }`
  for struct methods; a user `impl` block reuses it directly. Generics/`where`
  reuse ADR-0011's helpers.
- **AST:** an `Item::Impl { ty, generic_params, where_clause, methods }`.
- **Phase:** grammar+AST, then codegen (largely reuse), with a proof that a
  method defined in a separate `impl` block compiles and is callable.

### Unit 3 — `trait` + `impl Trait for Type` (DEFER to ADR-00NN)
- Needs a new `Item::Trait` (method signatures + optional default bodies) and an
  `impl Trait for Type` form, plus associated-item surface. Trait-bound checking
  stays rustc's. This is a feature the size of PS-08 and deserves its own ADR
  with its own phases; it should NOT be bundled into PS-09's first landing.
- Until then, a trait and its impls are written in `__rust__ { … }`.

### Unit 4 — `async` / `await` (DEFER to ADR-00MM, lowest priority)
- `async fn` + `.await` can lower verbatim, but async is viral and inert without
  a runtime (tokio/async-std) that the syntax layer does not own, so it buys
  little today and is the least aligned with the C-compat goal. Defer until a
  concrete need and a runtime-integration decision exist. Writable in
  `__rust__ { … }` meanwhile.

### Non-goals (explicit)
- **No unsafety, trait-coherence, or send/sync reasoning.** rustc owns all of
  it (ADR-0001). Crusty's `unsafe` is a passthrough marker, not a checker.
- **No trait or async surface in this ADR** — Units 3 and 4 are deferred, not
  specified here.
- **No `unsafe` SEMANTIC effect** — marking a block `unsafe` does not let Crusty
  permit anything it otherwise rejects; it only emits the keyword for rustc.

## Consequences
- **Easier:** the two cheap wins (`unsafe`, inherent `impl`) land first and
  unblock FFI/raw-pointer code and out-of-struct method organisation, reusing
  existing emission. PS-09 stops being a monolith.
- **Harder / accepted cost:** `trait` and `async` remain `__rust__`-only until
  their ADRs. Making `impl` a reserved keyword is a small source-compat break
  (a program using `impl` as an identifier stops compiling) — acceptable, since
  `impl` as an identifier in C-like code is rare and the reservation matches
  every other Rust-item keyword Crusty already reserves (`struct`, `enum`, …).
- **Reversible-cost:** Units 1 and 2 are additive variants/flags plus one new
  reserved word. The deferral of 3 and 4 is the main judgement; it is recorded
  here so a future reader does not read PS-09 as "all four, now".

### Alternatives considered
- **One PS-09 ADR specifying all four.** Rejected: trait and async are each
  PS-08-sized; one ADR covering four features of this spread is unreviewable and
  violates one-decision-per-ADR. This ADR commits the cheap two and defers the
  big two by name.
- **Do `trait` before `unsafe`/`impl`** because it is "more important".
  Rejected: the house rule is simplest-first; `unsafe` and inherent `impl` are
  small, reuse existing paths, and deliver value immediately, while `trait`
  needs a design pass. Value-ordering a harder feature ahead buys nothing but
  risk.
- **Skip `unsafe` as a surface feature (keep it `__rust__`-only).** Rejected:
  raw-pointer / FFI work is squarely in the C-compat target, `unsafe` is already
  reserved, and the lowering is a one-line passthrough — it earns its surface.

## Verification

### Unit 1 — `unsafe`
- [ ] `Statement::Unsafe(Block)` and `Function.is_unsafe` exist in `src/ast.rs`.
- [ ] `unsafe { … }` and `unsafe int f(...) { … }` parse into those nodes
      (parser tests); `unsafe` remains rejected as an identifier.
- [ ] Codegen emits `unsafe { … }` / `unsafe fn …`; non-unsafe output is
      byte-unchanged.
- [ ] End-to-end: an `unsafe` block performing a raw-pointer deref (value from
      `__rust__`) transpiles, compiles under `rustc -D unused_parens`, and runs.

### Unit 2 — inherent `impl`
- [ ] `Item::Impl { ty, generic_params, where_clause, methods }` exists.
- [ ] `impl Point { int get_x(self) { … } }` parses; `impl` is reserved.
- [ ] Codegen emits `impl Point { fn get_x(&self) -> i32 { … } }`, reusing the
      method emitter; a method defined in a separate `impl` block is callable
      end-to-end under rustc.

### Both committed units
- [ ] `cargo test` green and `clippy` clean after each phase.

### Deferred
- [ ] Unit 3 (`trait` / `impl Trait for`) is specified in its own ADR before
      implementation.
- [ ] Unit 4 (`async` / `await`) is specified in its own ADR before
      implementation.
