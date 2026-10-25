# ADR-0018: __thread__ concurrency primitive — implementation

## Status
proposed

## Context

ADR-0017 (accepted) decided Crusty's concurrency direction: async stays behind
the `__rust__` escape hatch, and `__thread__ { … }` is adopted as the native
primitive, lowering to `std::thread::spawn` (no runtime dependency). Its
validation section proved five `rustc` facts and fixed the one refinement the
sketch had wrong: the emitted closure must be `move`, not the bare `__func__`
closure, because a borrow-capturing `spawn` closure is an E0373 hard error.

ADR-0017 left three questions for this implementation ADR: join/handle
semantics, data-sharing rules, and whether to also offer scoped threads. This
ADR resolves them and specifies the implementation, in the shape of ADR-0014
(`Expression::Loop`) and ADR-0015 Unit 1 (`unsafe`): an additive AST node, a
one-form grammar rule, a `move`-closure codegen passthrough, and a `rustc`
proof.

Ground truth this builds on:
- `async`/`await`/`thread` are **not reserved**; `__thread__` is a
  double-underscore keyword-form (like `__rust__`/`__func__`), never an
  identifier, so it adds no reservation and cannot collide.
- `Expression::Loop { label, body }` is the precedent for a block construct that
  is first-class in expression position, so `let x = loop { break v; };` works.
  `__thread__` follows it exactly, which is what makes the handle form fall out
  for free.
- `std::thread::spawn<F, T>(f) -> JoinHandle<T>` where `F: FnOnce() -> T + Send +
  'static`. `JoinHandle::<T>::join(self) -> Result<T, Box<dyn Any + Send>>`.

## Decision

Implement `__thread__` as a **value-producing expression** lowering to
`std::thread::spawn(move || { <body> })`.

### Surface

```c
// Handle form — the spawn value is a join handle.
let h = __thread__ { heavy(); 42 };   // h : JoinHandle<i32>
let r = h.join();                     // r : Result<i32, …> (rustc's type)

// Fire-and-forget — the handle is simply dropped (expression statement).
__thread__ { log_event(); };
```

- **One surface, both uses.** `__thread__ { <body> }` is an **expression** whose
  value is the `JoinHandle<T>` that `spawn` returns. Binding it (`let h = …`)
  gives the handle form; using it as an expression statement and dropping the
  value gives fire-and-forget. This is the ADR-0017 "both forms" result with no
  second syntax — the same economy as `Expression::Loop` serving both
  `loop {}` and `let x = loop {}`.
- **`join()` is not sugared.** The user calls `h.join()` (and `.unwrap()` /
  `?` as they wish) — it is an ordinary method call on the handle, lowering to
  Rust's `JoinHandle::join` verbatim. Crusty adds no join keyword; the result
  type is rustc's.

### Lowering

```
__thread__ { <body> }   →   std::thread::spawn(move || { <body> })
```

- **`move` is mandatory** (ADR-0017 validation, E0373). The body's last
  expression is the thread's return value, so `std::thread::spawn` infers `T`
  from it — the same value-carrying-exit model as a `__func__` body / Rust
  closure.
- **No `label`, no params.** Unlike `Expression::Loop`, a thread body takes no
  parameters and carries no label. The node is just `{ body }`.

### AST

Additive variant, mirroring `Expression::Loop`:
```rust
Expression::Thread { body: Box<Block> }
```
An additive `Expression` variant rippled only to the exhaustive matches in
codegen + the two semantic walkers at Match/Loop time (the PS-12 lesson), so
construction-site churn is zero.

### Semantics

- Analyse the body as a block (so body errors surface), exactly like the
  `__func__` / `Loop` sub-body analysis.
- **No `Send`/`'static` pre-check.** Whether the captured, moved values are
  `Send + 'static` is rustc's to decide (ADR-0001). A borrow that cannot move,
  or a non-`Send` capture (`Rc`, raw pointer), surfaces as the rustc E0373 /
  E0277 the ADR-0017 probes showed — not a Crusty diagnostic.
- The expression's type is `Auto` (an opaque `JoinHandle<T>`); a subsequent
  `.join()` is a normal method call typed `Auto`, deferring to rustc (SEM-00
  territory, same as any method call).

### Data-sharing — the decided model

The **move-capture** model is the answer, and it is the honest C-like one,
consistent with ADR-0014's nested-fn capture split ("hand the work its data;
you don't borrow across the boundary"):
- An **owned** local is moved into the thread (ADR-0017 probe 4a).
- Sharing *back*, or sharing one value across several threads, is done with the
  Rust tools the user writes **explicitly** — `Arc`, a channel, a `Mutex` —
  constructed in a `__rust__ { … }` value or via normal calls. Crusty does not
  wrap or hide them; that would be the syntax layer smuggling in a concurrency
  model it does not own.

### Non-goals (this ADR)

- **Scoped threads (`std::thread::scope`) are a follow-on, not this ADR.**
  ADR-0017 probe 3 proved they work and are strictly more capable (borrow
  locals, structured auto-join), but they need a *scope context* wrapping one or
  more spawns — a second surface (e.g. a `__scope__ { … }` enclosing block), not
  the single `__thread__` block. Simplest-first: ship `move`+`spawn`, add scope
  only if a real need appears. Recorded so the idea is not lost.
- **No thread-pool, no runtime, no `Send`/`Sync` reasoning** — all rustc's.
- **No `async` surface** — settled in ADR-0017.

## Consequences

- **Easier:** Crusty gets real, dependency-free concurrency in one additive
  node. The handle and fire-and-forget forms are one surface. `join`, `Arc`,
  channels are all just ordinary calls — nothing new to learn beyond the block.
- **Harder / accepted:** the `move` semantics mean a thread body that *reads* an
  enclosing local it cannot take ownership of is a rustc error the user must
  resolve (clone, `Arc`, or restructure) — the honest consequence, surfaced by
  rustc, not softened by Crusty. Sharing-back is explicit (`Arc`/channels), more
  verbose than a borrow-everything model but matching Rust's actual rules.
- **Reversible cost:** Low. Additive variant; removing or supersession touches
  one grammar rule, one codegen arm, two walkers, and the node. Adding scoped
  threads later is purely additive on top.

## Verification

- [ ] `Expression::Thread { body }` exists in `src/ast.rs`; additive (no
      construction-site churn beyond exhaustive matches).
- [ ] Grammar: `__thread__ { <block> }` parses in expression position;
      `__thread__` is a keyword-form (double underscore), and a guard test
      proves `thread` is still a valid ordinary identifier.
- [ ] Codegen emits `std::thread::spawn(move || { <body> })` — the **`move`**
      form (not the bare `__func__` closure); the body's tail expression is the
      thread's return value.
- [ ] Semantics analyse the body; no `Send`/`'static` pre-check is added.
- [ ] End-to-end under `rustc -D unused_parens`: a handle-form `__thread__`
      computing a value, `join()`ed back, runs and prints the value (mirrors
      ADR-0017 probe 1); and a fire-and-forget `__thread__ { … };` statement
      compiles and runs.
- [ ] A negative proof documents the move boundary: a `__thread__` body touching
      an owned local compiles (moved), and the docs note that borrowing a local
      or capturing a non-`Send` value is a rustc error (ADR-0017 probes 2/4b),
      not a Crusty one.
- [ ] `docs/syntax/` documents the `__thread__` block, the handle vs
      fire-and-forget uses, the `move`-capture model, and the explicit `Arc`/
      channel sharing-back path; the roadmap notes `__thread__` implemented.

## Alternatives considered

- **A dedicated join keyword / fire-and-forget as a separate form.** Rejected:
  one expression node serving both (bind the handle vs drop it) is less surface
  for the same power, exactly as `Expression::Loop` serves both loop uses.
- **Lower to `thread::scope` by default.** Rejected for the first cut: `scope`
  needs a surrounding context and borrows locals, which is a different (richer)
  surface — a follow-on, not the simplest primitive.
- **Pre-check `Send`/`'static` in Crusty.** Rejected: duplicates rustc, against
  ADR-0001; rustc's E0373/E0277 are already clear and point at the fix.
