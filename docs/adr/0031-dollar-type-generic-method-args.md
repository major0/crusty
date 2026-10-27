# ADR-0031: `$type` generic method arguments (no turbofish)

## Status
superseded by ADR-0034

## Context

A generic method call sometimes needs an explicit type argument because
inference cannot recover it from the value arguments — the canonical case is
`str::parse`, where the target type is known only from context:

```rust
let n = "42".parse::<i32>().unwrap();   // Rust turbofish — the ::<i32> is required
```

Crusty had **no C-flavoured surface for this**. An instance generic method call
could not supply its type argument at all: the `cli` example app hit exactly
this with clap's `matches.get_one::<String>("name")` and had to work around it
by anchoring the type in a C-style declaration (`&String name = …`). Crusty's
only explicit-type-argument surfaces are *type-level*:

- `@Type(G).method()` → `Type::<G>::method()` (`ExplicitGenericCall`), and
- `@Type.method::<G>()` → `Type::method::<G>()` (`TurbofishCall`, PS-14),

and neither reaches an **instance** receiver (a value, not a type), which is
where the turbofish is most often forced.

Three C-family languages informed the surface (verified, not recalled):

- **C++/Rust**: `f<T>()` / `f::<T>()` — the angle-bracket/`::` turbofish the
  user explicitly rejected as un-C-like.
- **Objective-C**: adds *no* call-site generic syntax; the C idiom for "this
  call yields type T" is the cast `(T)expr`.
- **Zig**: has no generic syntax at all — a type **is a value**, passed as an
  ordinary argument: `f(u32, x)`. The type rides in the argument list.

The Zig model — type in the argument list, no `<>`/`::` — reads most naturally
to a C programmer. But Crusty types are **not** values (it is a C interface to
Rust, ADR-0001), so a bare `get_one(String, "n")` is ambiguous: the parser
cannot tell the type `String` from a variable named `String` (confirmed — a
bare type name parses as a value identifier and only fails later as an undefined
variable). A one-character marker resolves this while keeping the Zig feel.

A collision scan settled the marker: `:` already carries ~9 grammar meanings
(struct-init field, ternary else, loop label, `switch` case, typed const,
generic-bound lead, `::` path) and sits dangerously close to several inside an
argument list. `$` has **zero** meaning in Crusty source (every `$` in the
grammar is the PEG capture operator in the meta-grammar; `$` is not an
identifier character) and appears in Rust only inside `macro_rules!` bodies,
which this form never emits. `$` is free and reads as a "meta/type" marker.

## Decision

An instance generic method call takes its explicit type arguments as
**leading `$Type` arguments** in the ordinary argument list, Zig-style, with `$`
marking each as a type:

```c
matches.get_one($String, "name")     // -> matches.get_one::<String>("name")
"42".parse($i32).unwrap()            // -> "42".parse::<i32>().unwrap()
m.convert($String, $i32, value)      // -> m.convert::<String, i32>(value)
m.parse($i32)                        // -> m.parse::<i32>()   (type args only)
```

- One or more `$Type` arguments come **first**, comma-separated, followed by the
  ordinary value arguments. Each `$`-marked argument is a TYPE; the `$` is what
  tells the parser so, so a bare type name need not be distinguishable from a
  variable.
- Lowering lifts the `$`-marked arguments out of the value list into the
  method's Rust turbofish: `recv.method::<T1, T2>(valargs)`.
- The call types as `Auto` semantically — the explicit type arguments are
  arbitrated by rustc, the same treatment as the existing `TurbofishCall`.
- A plain method call with no `$` argument is unchanged.

Represented by an additive `Expression::GenericMethodCall { receiver, method,
generics, args }` (and a parser `PostfixOp::GenericMethodCall`), tried before the
plain method-call postfix so a leading `$` wins.

## Consequences

Easier:
- The instance generic-method gap is closed with a C-readable surface: the type
  rides in the argument list (Zig), marked by a single `$`, with no `<>`/`::`.
- `$` is unambiguous and collision-free, so the parser needs no fragile
  "is this identifier a known type?" lookup and no shadowing hazard.
- Additive variant — zero construction-site churn; the AST reshape touches only
  the three exhaustive-match sites (codegen + two semantic walkers).

Harder / accepted:
- **Scope: instance methods only.** This ADR covers `recv.method($T, args)`. The
  *type-scoped* turbofish `@mem.size_of::<i32>()` (PS-14 `TurbofishCall`) is left
  as-is; whether `$type` should also replace it — `@mem.size_of($i32)` — for one
  uniform generic-argument syntax (and retire `::<>` entirely) is deferred to a
  follow-up ADR, so this change does not touch PS-14.
- `$type` calls type as `Auto`, so rustc remains the arbiter for their return
  type — consistent with ADR-0001 and the SEM-00 method-typing stance, but no
  local diagnostics for a wrong type argument.
- A new, Crusty-specific spelling to learn and document (no C/Rust programmer has
  seen `$Type` as a type argument), accepted because it is unambiguous and the
  cast-free Zig shape is the most C-intuitive option that resolves the
  type-vs-value ambiguity.

## Rejected alternatives

- **Rust/C++ turbofish `recv.method::<T>(args)`.** Rejected: the `::<>` C++-ism
  the user explicitly dislikes.
- **Markerless Zig `recv.method(String, args)`.** Rejected: ambiguous in Crusty
  (a bare `String` is indistinguishable from a variable), reliable only for
  primitive-keyword type names and fragile under shadowing for user types.
- **A `:type` marker.** Rejected: `:` already carries ~9 grammar meanings and
  collides inside an argument list (ternary, struct-init field).
- **A leading cast `(T) call()`.** Rejected: conflates the method's type argument
  with an `as`-cast of the result (two different operations, one surface).
- **D-style `method!(T)(args)`.** Rejected: `!` is already macro invocation in
  Crusty (`name!(...)`), so it collides.

## Verification

- [x] `m.get_one($String, "n")` emits `m.get_one::<String>("n")`.
- [x] `m.convert($String, $i32, 5)` emits `m.convert::<String, i32>(5)`.
- [x] `m.parse($i32)` (type args only, no value args) emits `m.parse::<i32>()`.
- [x] A plain `m.plain(1, 2)` (no `$`) is unchanged.
- [x] `"42".parse($i32).unwrap()` compiles under `rustc -D unused_parens` and
      runs (`n == 42`, used in arithmetic so a mis-lowering would fail).
- [x] The full existing suite stays green (additive variant, 3 match sites).
- [x] `docs/syntax/expressions.md` documents the `$type` surface and that it is
      instance-method-scoped (PS-14 type-scoped turbofish unchanged).
