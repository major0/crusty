# ADR-0019: scoped threads — __scope__

## Status
accepted

## Context

ADR-0018 implemented `__thread__ { … }` → `std::thread::spawn(move || { … })`
and recorded scoped threads (`std::thread::scope`) as a follow-on, because
`scope` needs a surrounding context the single `__thread__` block cannot
express. ADR-0017 probe 3 proved `scope` lets a spawned closure **borrow**
enclosing locals with a structured, automatic join — strictly more capable than
the `move`-everything top-level form, which rejects a borrow (rustc E0373).

Validated shape (`rustc`, edition 2021):
```rust
let total = std::thread::scope(|s| {
    let h = s.spawn(|| data.iter().sum::<i32>()); // borrows `data`
    s.spawn(|| println!("len={}", data.len()));   // borrows `data`
    h.join().unwrap()
});                                                 // auto-joins here
```
Two facts decide the surface: (1) `scope` passes a **scope handle `s`** into its
closure, and spawns are `s.spawn(...)`, not `std::thread::spawn(...)`; (2) a
scoped spawn closure is **not `move`** — borrowing locals is the entire point.

## Decision

Add `__scope__ { … }`, a block that establishes a thread scope. Inside it,
`__thread__ { … }` lowers to a **scoped, borrowing** spawn instead of the
top-level moving one.

### Surface

```c
let total = __scope__ {
    let h = __thread__ { return sum(data); };  // borrows `data`, no move
    __thread__ { log(data); };                 // also borrows
    return h.join().unwrap();
};
```

- `__scope__ { <body> }` is a **value-producing expression** (like `__thread__`
  and `loop`): its value is the scope closure's returned value, so
  `let total = __scope__ { … return v; };` works and a bare `__scope__ { … };`
  is a statement.
- A `__thread__` **lexically inside** a `__scope__` body is a **scoped** spawn:
  it borrows (no `move`) and is auto-joined when the scope ends.
- A `__thread__` **outside** any `__scope__` keeps the ADR-0018 behaviour:
  `std::thread::spawn(move || …)`.

### Lowering

```
__scope__ { body }   →   std::thread::scope(|__crusty_scope| { body })
__thread__ { body }        (inside a __scope__)   →   __crusty_scope.spawn(|| { body })
__thread__ { body }        (outside)              →   std::thread::spawn(move || { body })
```

The scope handle is named `__crusty_scope` — a fixed, double-underscore,
reserved-looking name the user does not write and is extremely unlikely to
collide with (consistent with how generated names are kept out of the user's
namespace). A scoped `__thread__` is **not `move`**, so it borrows; the scope
guarantees every spawned thread is joined before the borrowed data is released,
which is exactly why rustc accepts the borrow.

### AST

Additive `Expression::Scope { body: Box<Block> }` (mirroring
`Expression::Thread`). The scoped-vs-top-level distinction for `__thread__` is
**not** a new field on `Thread`; it is decided at codegen time by whether a
`__scope__` encloses it. Codegen already walks a sub-generator per block, so it
threads a small "inside a scope, use this handle name" flag down while emitting
a `__scope__` body — the body's `__thread__` nodes read that flag. This keeps
`Expression::Thread` unchanged and the two behaviours one node, decided by
lexical position (smart data / dumb code: the position carries the meaning).

### Semantics

- Analyse the `__scope__` body as a block (errors surface), expected-return-type
  cleared like `__thread__`/`AnonFunction` (a `return` inside returns from the
  scope closure). The expression types `Auto` (the scope's returned value;
  rustc infers it).
- No borrow/`Send`/lifetime checking — rustc owns it (ADR-0001). A scoped
  `__thread__` that outlives its borrow, or a non-scoped one borrowing a local,
  is a rustc error, not a Crusty one.

### Non-goals

- **No nested-scope handle disambiguation beyond the innermost.** A `__thread__`
  binds to the nearest enclosing `__scope__`. Nested `__scope__` blocks each get
  their own handle; a `__thread__` uses the innermost — the common case. (Rust's
  own `scope` nests the same way.)
- No thread-pool, no runtime, no async — unchanged from ADR-0017/0018.

## Consequences

- **Easier:** the borrowing concurrency case — "spawn some workers over my local
  data, join them all here" — becomes expressible without `Arc`/clone, matching
  Rust's `scope`. `__thread__` keeps one spelling; its behaviour follows its
  position.
- **Harder / accepted:** `__thread__` is now context-sensitive (scoped vs
  top-level). That is a real subtlety, documented plainly: *inside `__scope__`
  it borrows and auto-joins; outside it moves and runs free.* The generated
  handle name `__crusty_scope` is reserved by convention, not enforced — a user
  who writes that exact identifier inside a scope could shadow it (vanishingly
  unlikely given the double-underscore + prefix; rustc would flag a type error,
  not silent breakage).
- **Reversible cost:** Low. Additive `Expression::Scope`; the codegen flag is
  local to the thread/scope arms.

## Verification

- [x] `Expression::Scope { body }` exists in `src/ast.rs`; additive.
- [x] `__scope__ { <block> }` parses in expression position; `__scope__` is a
      double-underscore keyword-form; a guard test proves `__scope__x` is a
      plain identifier.
- [x] Codegen: `__scope__` emits `std::thread::scope(|__crusty_scope| { … })`;
      a `__thread__` inside emits `__crusty_scope.spawn(|| { … })` (no `move`);
      a `__thread__` outside still emits `std::thread::spawn(move || { … })`.
- [x] Semantics analyse the scope body; no borrow/Send pre-check.
- [x] End-to-end under `rustc -D unused_parens`: a `__scope__` whose scoped
      `__thread__`s borrow a shared local, one returning a value joined inside
      the scope, compiles and runs; the borrowed local is still usable after.
- [x] `docs/syntax/concurrency.md` documents `__scope__`, the scoped-vs-top-level
      `__thread__` distinction, and that the scope auto-joins; the roadmap notes
      scoped threads landed.

## Alternatives considered

- **A distinct `__scoped_thread__` keyword instead of context-sensitive
  `__thread__`.** Rejected: two spellings for "spawn a thread" is more surface;
  lexical position already disambiguates, matching how Rust nests `scope`.
- **A field on `Expression::Thread` marking it scoped.** Rejected: the parser
  would have to know at parse time whether a `__scope__` encloses it, which it
  does (lexically), but a codegen-time flag keeps the node identical to the
  top-level case and avoids a redundant field — the position is the data.
- **Pass the scope handle as a visible Crusty value.** Rejected: exposing
  `__crusty_scope` as a nameable binding invites misuse (storing it, spawning
  after scope end) that rustc would reject anyway; keeping it generated and
  implicit matches the block's intent.
