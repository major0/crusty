# ADR-0016: defining `trait`s and general `impl Trait for Type` (PS-09 Unit 3)

## Status
proposed

## Context
ADR-0015 scoped PS-09 into four units and **deferred Unit 3** — defining new
`trait`s and a general `impl Trait for Type` — to its own ADR, naming it
"a feature the size of PS-08". This is that ADR. It is design-first: it decides
the surface and the lowering before any code, per `adr.md`.

What the code holds today (read in `src/ast.rs`, `src/parser.rs`,
`src/codegen.rs`, `src/semantic.rs`, not speculation):

- **A general trait-impl is already half-built.** ADR-0015 Unit 2 landed
  `Item::ImplBlock { visibility, target, trait_name: Option<Ident>, name,
  methods, … }`. Its codegen (`generate_impl_block`) already emits
  `impl <trait_name> for <target> { … }` **for any `trait_name`**, stripping
  `pub` from trait-impl methods (rustc E0449). The ONLY thing that pins it to
  `Default` is the parser: `impl_block_kind()` reads `struct → None` /
  `default → Some(Ident::new("Default"))` and nothing else. So generalising the
  **impl** side to `impl SomeTrait for Type` is almost entirely a **grammar**
  change — the AST field and the emission already carry an arbitrary trait name.
- **There is no way to DEFINE a trait.** No `Item::Trait`, no grammar, no
  codegen. `trait` is not reserved (it appears only in proptest generators), so
  a Crusty program could use `trait` as an identifier today.
- **Methods are `Function` nodes with the "first param named `self`"
  convention** (no `SelfParam`); `return_type: Option<Type>`. A trait method
  *signature* (no body) and a *default* method (with body) differ only by
  whether `Function.body` is present — and `Function.body` is already
  `Option<Block>` (ADR, used by `extern`). So a trait's method list is a
  `Vec<Function>` where a `None` body means "required signature" and a `Some`
  body means "default method". No new method node is needed.
- **Trait-bound checking is explicitly rustc's job (ADR-0001).** Crusty already
  carries generic bounds as **raw strings** passed verbatim (`GenericParam::
  Type { bounds: Vec<String> }`, PS-08/ADR-0011). Trait coherence, orphan rules,
  object safety, and whether an `impl` actually satisfies the trait are all
  rustc's — Crusty only needs to spell the surface and lower it.
- **`__rust__ { … }`** can express a trait and its impls today, so this is not a
  blocker; the question is the first-class surface.

The one real design fork is **how to spell defining a trait** in Crusty's
C-flavoured, `typedef`-based style — the impl side is nearly free because Unit 2
already generalises.

## Decision
Add first-class surface for (A) **implementing a named trait for a type** by
generalising Unit 2's impl-block, and (B) **defining a new trait** with a new
`typedef trait` item and an `Item::Trait` AST node. Keep the `typedef`/`@Type`
family consistent with Unit 2 — **no new `impl` or `trait` keyword is reserved**;
`trait` becomes a *contextual* word after `typedef`, exactly as `struct` and
`default` already are. Trait-bound satisfaction stays rustc's.

### Part A — `impl Trait for Type` (generalise Unit 2)
- **Surface:** generalise the impl-block kind from the fixed `struct`/`default`
  to an arbitrary trait name, reusing the exact Unit 2 shape:
  ```c
  typedef Drawable { void draw(&self) { … } } @Circle;   // impl Drawable for Circle
  ```
  `typedef default { … } @T;` stays valid (it is just the `Default` case of this
  general form); `typedef struct { … } @T;` stays the inherent-impl case.
- **Lowering:** **none new** — `generate_impl_block` already emits
  `impl <trait_name> for <target>` for any `trait_name` and already strips `pub`
  from trait-impl methods. This part is a parser change only.
- **AST:** **no change** — `ImplBlock.trait_name: Option<Ident>` already holds
  an arbitrary name. `impl_block_kind()` gains a third arm: a trait-name
  identifier (tried AFTER `struct`/`default` so those keep their meaning).
- **Grammar fork handled:** `struct` and `default` must still win over a bare
  trait-name ident, so `impl_block_kind()` orders `kw_struct` / `kw_default`
  first, then `ident()` as the general trait name. A trait literally named
  `struct`/`default` is impossible (they are keywords), which is correct.

### Part B — defining a `trait` (new item)
- **Surface:** a `typedef trait` item, mirroring the `typedef struct { } Name;`
  type-definition shape (no `@`, because a trait DEFINES a name rather than
  implementing on an existing `@Type`):
  ```c
  typedef trait {
      void draw(&self);                 // required method (signature only)
      int area(&self) { return 0; }     // default method (has a body)
  } Drawable;
  ```
  → `pub trait Drawable { fn draw(&self); fn area(&self) -> i32 { 0 } }`.
  `static typedef trait { … } Name;` → a private (crate-local) trait.
- **AST:** a dedicated `Item::Trait(Trait)` with
  `Trait { visibility, name: Ident, generic_params: Vec<GenericParam>,
  methods: Vec<Function> }`. A method with `body: None` is a required
  signature; `body: Some(_)` is a default method. Reuse `GenericParam`
  (ADR-0011) for `typedef trait Container<T> { … }`. Additive variant → zero
  construction-site churn (the SEM-00/PS lesson: additive `Item`/`Statement`
  variants ripple only to the few exhaustive matches in codegen + the two
  semantic walkers).
- **Grammar:** a `trait_def` rule, `[static] typedef trait { trait_member* }
  Name ;`, where a `trait_member` is a method **signature** (`ret name(params);`)
  or a **default method** (`ret name(params) { block }`). Reuse `struct_method`
  for the default-body case and add a signature-only arm ending in `;`. `trait`
  is matched as a contextual word right after `typedef` (like `struct`), so it
  is NOT added to `keyword()` and stays usable as an identifier elsewhere.
- **Codegen:** a `generate_trait` emitting `pub trait Name<…> { … }`; a method
  with no body emits `fn name(&self) -> R;` (signature, trailing `;`, no `pub`
  — trait items share the trait's visibility, E0449); a default method emits
  `fn name(&self) -> R { … }` (also no `pub`). Crusty target round-trips to
  `typedef trait { … } Name;`.
- **Semantics:** register the trait name; analyse each default-method body like
  a function body (so body errors surface); a signature-only method is recorded
  but not body-analysed. Whether an `impl` satisfies the trait, trait-object
  safety, and bound resolution are **rustc's** (ADR-0001) — Crusty does not
  check them.

### Phasing (simplest-first, each independently provable)
1. **Part A** first — it is a one-arm parser change over existing AST+codegen,
   provable immediately: define a trait in `__rust__ { … }`, implement it with
   `typedef MyTrait { … } @Type;`, compile under rustc, call through the trait.
2. **Part B** second — the new `Item::Trait`, grammar, codegen, semantics. Prove
   with a wholly-Crusty trait (required + default method) implemented via Part A
   on a Crusty struct, compiled and run under `rustc -D unused_parens`.

### Non-goals (explicit)
- **No trait-bound / coherence / object-safety checking.** rustc owns it
  (ADR-0001). Bounds on trait generics stay raw strings (ADR-0011).
- **No associated types or associated consts in this ADR.** If needed they are a
  follow-up; the first landing is methods (required + default) only, which is
  the C-compat-relevant core. Recorded so a reader does not read trait support
  as "everything traits can do".
- **No supertraits / `where`-clause-on-trait surface beyond raw bounds.** A
  supertrait (`trait A: B`) can be written in `__rust__` until a concrete need;
  not specified here.
- **No `dyn`/trait-object surface decision** — `dyn Trait` is a *type* spelling
  question (does Crusty want `dyn`, or a C-style spelling?), orthogonal to
  defining/implementing a trait. Deferred.

## Consequences
- **Easier:** Part A unlocks arbitrary trait impls for essentially free (parser
  arm only), because Unit 2 already generalised the AST and codegen. Part B adds
  the missing half — defining traits — so a Crusty program can declare an
  interface and implement it without dropping into `__rust__`. The `typedef`
  family stays consistent (`struct` = inherent, `default`/`<Trait>` = trait
  impl, `trait` = definition).
- **Harder / accepted cost:** associated types/consts, supertraits, and `dyn`
  objects remain `__rust__`-only until their own follow-ups — a trait whose
  contract needs an associated type cannot yet be spelled in pure Crusty.
  Trait-method *typing* (resolving a call through a trait to its return type)
  rides on SEM-00's method-signature table (ADR-0010) and inherits its current
  `Type::Auto` laundering for anything not yet keyed — i.e. local diagnostics on
  trait-method calls stay weak, rustc arbitrates. Accepted, consistent with
  ADR-0001.
- **Reversible-cost:** Part A is a single additive parser arm; Part B is an
  additive `Item::Trait` variant (zero construction-site churn) plus one grammar
  rule, one codegen fn, and walker arms. Nothing reshapes an existing node. The
  main judgement is the DEFINE-a-trait spelling (`typedef trait { } Name;`),
  chosen for consistency with `typedef struct { } Name;`; superseding it later
  is a grammar change, not a data-model break.

### Alternatives considered
- **Rust-style `trait Name { } ` / `impl Trait for Type { }` keywords.**
  Rejected for the same reason ADR-0015 rejected a Rust-style `impl`: it would
  introduce two reserved keywords and a second surface alongside the `typedef`/
  `@Type` family already shipped in Unit 2. The `typedef trait`/`typedef <Trait>
  @Type` spelling reuses the established family and reserves nothing.
- **A separate `interface` keyword (C#/Go flavour).** Rejected: a new reserved
  word for a concept `typedef` already frames, and "interface" misleads (Rust
  traits carry default bodies and associated items, unlike a pure interface).
- **Fold trait-definition into `typedef struct`** (a struct with only method
  signatures). Rejected: conflates a type definition with a contract; the
  codegen targets are different Rust constructs (`struct` vs `trait`) and the
  member grammar differs (signature-only methods), so a distinct `trait` kind
  is clearer and smarter-data.
- **Model a trait method's required-vs-default by a new flag/node.** Rejected:
  `Function.body: Option<Block>` already encodes exactly this (`None` =
  required, `Some` = default), reusing the existing node — smart data, no new
  variant.
- **Do Part B (define) before Part A (impl).** Rejected by simplest-first: Part
  A is a one-arm parser change over code that already exists and is provable
  immediately; Part B is the larger half. Landing A first banks the cheap win
  and gives B a working impl-surface to prove against.

## Verification

### Part A — general `impl Trait for Type`
- [ ] `impl_block_kind()` accepts an arbitrary trait-name identifier (after
      `struct`/`default`), producing `ImplBlock.trait_name = Some(<name>)`; no
      AST or codegen change is required for emission.
- [ ] `typedef Drawable { … } @Circle;` parses to an `ImplBlock` with
      `target = Circle`, `trait_name = Some(Drawable)`, and lowers to
      `impl Drawable for Circle { … }` with no `pub` on the methods.
- [ ] `typedef struct { } @T;` (inherent) and `typedef default { } @T;`
      (`impl Default for T`) still parse unchanged — a guard test proves
      `struct`/`default` are not captured as trait names.
- [ ] End-to-end: a trait declared in `__rust__ { … }`, implemented via
      `typedef <Trait> { … } @Type;`, compiles under `rustc -D unused_parens`
      and a method is called through the trait at runtime.

### Part B — defining a `trait`
- [ ] `Item::Trait(Trait { visibility, name, generic_params, methods })` exists
      in `src/ast.rs`; a method's `body: None` means required, `Some` means
      default.
- [ ] `[static] typedef trait { <sig>; <default-method> } Name;` parses into
      `Item::Trait`; `trait` is NOT added to `keyword()` (it stays a valid
      identifier outside the `typedef trait` position — a guard test proves a
      variable named `trait` still parses).
- [ ] Codegen emits `pub trait Name { fn req(&self); fn def(&self) -> R { … } }`
      (private `trait` for `static`); no `pub` on trait items.
- [ ] End-to-end: a Crusty-defined trait (one required + one default method),
      implemented via Part A on a Crusty struct, compiles under
      `rustc -D unused_parens` and runs, exercising both the overridden required
      method and the inherited default.

### Both parts
- [ ] `cargo test` green and `clippy` clean after each part.
- [ ] `docs/syntax/typedef.md` documents the general `typedef <Trait> @Type;`
      impl form and the `typedef trait { } Name;` definition form; the roadmap
      marks PS-09 Unit 3 landed.
