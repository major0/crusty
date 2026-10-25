# ADR-0017: async / await and the __thread__ concurrency direction (PS-09 Unit 4)

## Status
proposed

## Context

ADR-0015 split PS-09 into four units, committed `unsafe` (Unit 1) and the
`typedef … @Type` impl surface (Unit 2), and deferred two large ones to their
own ADRs. Unit 3 (traits) landed in ADR-0016. This ADR is the design-first
decision for the last one — **Unit 4, `async`/`await`** — and it also pins down
the `__thread__` concurrency direction ADR-0015 recorded, because the two share
one root question: **Crusty is a syntax layer (ADR-0001); concurrency needs a
runtime, which a syntax layer does not own.**

Ground truth in the current tree:
- **`async` / `await` are not modelled and not even reserved.** They appear only
  in proptest keyword-strategy generators, not in the real `keyword()` set, so a
  Crusty program could use `async`/`await` as ordinary identifiers today.
- **Async already works through the escape hatch.** An `async fn`, a `.await`,
  and a runtime attribute like `#[tokio::main]` can be written verbatim in a
  `__rust__ { … }` block right now and called from Crusty. So async is **not a
  hard blocker** — the question is only whether a first-class surface earns its
  place.
- **The double-underscore family is the established pattern** for "this is a
  Rust concept with no C spelling": `__rust__` (verbatim items/exprs), `__func__`
  (a closure). A concurrency primitive would join that family, not invent a new
  keyword.

### The forces specific to async

- **Async is viral.** An `async fn` returns a future; its callers must `.await`
  it, which makes *them* async, up to a runtime entry point. A syntax layer can
  lower `async`/`.await` verbatim, but it cannot make the program *run* — that
  needs an executor (tokio, async-std, smol) the layer does not own and should
  not pick.
- **It is the least C-like of the four units.** C has no `async`. Surfacing it
  as a first-class Crusty keyword imports a Rust-and-JavaScript-shaped idiom
  into a C-like language, against the project's grain.
- **A verbatim lowering is nearly free but nearly useless alone.** `async`/
  `await` → `async`/`.await` is a one-line passthrough (like `unsafe`), but
  without a runtime-entry story it only moves the `__rust__` boundary a few
  tokens, buying little.

### The force the user named: `__thread__`

The user's stated direction is **not** to import `async`/`await` but to offer a
`__thread__ { … }` block — double-underscore family, like `__rust__`/`__func__`
— that spawns its body to run concurrently "on its own core", a go-routine-style
primitive as Crusty's own answer to concurrency. This is a style signal, and it
reframes Unit 4: the interesting question is not "how do we spell `async`" but
"what is Crusty's concurrency primitive, and does it even need `async`".

## Decision

**Defer BOTH async and `__thread__` as implementations; decide only the
direction here.** Specifically:

1. **`async` / `await` first-class surface — NOT ADOPTED (for now).** Crusty
   will *not* add `async`/`await` as reserved keywords or first-class forms in
   this unit. The `__rust__ { … }` escape hatch remains the supported way to
   write async Rust and call it from Crusty, exactly as it works today. This
   keeps the syntax layer out of a runtime decision it cannot make, and keeps
   the language C-like. The surface can be revisited if a concrete need appears
   that the escape hatch cannot serve.

2. **`__thread__ { … }` — ADOPTED AS DIRECTION, deferred as implementation.**
   Crusty's concurrency answer is a double-underscore block primitive, not
   imported `async`/`await`. A `__thread__ { <body> }` spawns its body to run
   concurrently; the leading sketch is:
   ```c
   __thread__ {
       heavy_work();            // runs concurrently
   }
   ```
   lowering to a standard-library thread spawn (`std::thread::spawn(|| { … })`),
   with the body as the closure — reusing the `__func__`-style closure lowering.
   Crucially this needs **NO runtime dependency**: `std::thread` is in Rust's
   standard library, so a `__thread__` block compiles and runs with nothing
   added to `Cargo.toml` — unlike async, which needs an executor crate. That is
   the decisive reason to prefer it as the first concurrency primitive.

   What is **not** decided here, and gates implementation:
   - **Join/handle semantics.** Does `__thread__` return a handle (so the caller
     can `.join()` and read a result), or is it fire-and-forget? A handle form
     (`let h = __thread__ { … };`) maps to `JoinHandle<T>`; a statement form is
     fire-and-forget. This is the core design fork.
   - **Data-sharing rules.** A `std::thread::spawn` closure must be `'static` +
     `Send`; captured references do not satisfy that. Crusty does not model
     lifetimes (ADR-0001) and defers to rustc, so a `__thread__` that captures a
     local will surface a rustc borrow/lifetime error. The design must decide
     whether to document that as the honest consequence (consistent with the
     `__func__`/nested-fn capture split in ADR-0014) or to restrict captures.
   - **Scoped threads.** `std::thread::scope` (stable since Rust 1.63) allows
     borrowing locals in spawned threads with a structured join. A `__thread__`
     that lowers into a `scope` is more useful but needs a scope context, which
     complicates the single-block surface.

3. **`async` and `__thread__` each get their own implementation ADR when
   pursued.** This ADR records the DIRECTION (escape-hatch for async;
   `__thread__` as the native primitive) and the open design questions, so the
   decision is captured and PS-09 Unit 4 is closed as "direction decided,
   implementation deferred" rather than left open-ended.

### Why not adopt async now (alternatives considered)

- **Adopt a verbatim `async`/`await` passthrough (like `unsafe`).** Rejected for
  now: unlike `unsafe` (which is self-contained and needs no dependency), async
  is inert without an executor the syntax layer must not choose, and it is the
  least C-like surface. The payoff over the existing `__rust__` hatch is small.
- **Adopt async AND pick a default runtime (e.g. tokio).** Rejected: picking a
  runtime is a heavy, hard-to-reverse dependency decision that belongs to the
  application, not the transpiler. It violates "the syntax layer does not own a
  runtime".
- **Do nothing for concurrency.** Rejected: the user has a clear direction
  (`__thread__`), and recording it now — with its `std::thread` lowering and the
  open join/sharing questions — is what makes the eventual implementation a
  small, well-scoped ADR instead of a fresh design from zero.

## Consequences

- **Easier:** PS-09 closes cleanly — all four units have a decided disposition
  (1–3 implemented, 4 direction-decided). The async runtime decision is kept out
  of the transpiler. `__thread__` has a concrete, dependency-free lowering target
  identified (`std::thread::spawn`), so its future ADR starts from a sketch, not
  a blank page.
- **Harder / accepted:** Async Crusty code stays in `__rust__` blocks for now —
  slightly more verbose than a first-class `async fn` would be. `__thread__` is
  named and sketched but not yet usable; a user wanting threads today uses
  `__rust__ { std::thread::spawn(...) }`. The capture/`Send`/lifetime
  consequences of `__thread__` are rustc's and will surface as rustc errors, not
  friendly Crusty diagnostics (consistent with ADR-0001).
- **Reversible cost:** Low. Nothing is implemented, so adopting async later, or
  choosing a different concurrency primitive, supersedes this ADR without
  unwinding any code. `async`/`await`/`thread` remain unreserved identifiers.

## Verification

- [ ] `async`, `await`, and `thread` remain UNreserved — a guard test proves a
      variable or function named `async` / `await` / `thread` still parses
      (documenting that no first-class surface was added in this unit).
- [ ] `docs/syntax/` records the decision: async is written via `__rust__ { … }`
      (with a short example of an `async fn` in `__rust__` called from Crusty),
      and `__thread__` is listed as the planned concurrency primitive with its
      `std::thread::spawn` lowering and open questions.
- [ ] The ADR-0009 roadmap marks PS-09 Unit 4 as "direction decided,
      implementation deferred", closing PS-09.
- [ ] When `__thread__` is implemented, it is specified in its own ADR that
      resolves the join/handle, data-sharing, and scoped-thread questions above.

## Notes

This ADR deliberately decides *less* than Units 1–3: it is a direction decision,
not an implementation spec, because the thing that gates implementation (a
runtime/threading-semantics choice) is a real design fork the user should drive,
not one to settle implicitly by shipping a passthrough. That is the honest
application of "simplest first" and "surface the irreversible decision with the
reasoning in hand" rather than expanding scope into a runtime commitment.
