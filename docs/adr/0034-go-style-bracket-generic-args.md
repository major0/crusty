# ADR-0034: Go-style [T] generic call arguments (supersedes $type)

## Status
accepted

## Context

ADR-0031 chose a `$Type` marker for explicit generic arguments at a call site
(`s.parse($i32)`), and ADR-0033 extended it to the type-scoped `@`-form
(`@mem.size_of($i32)`). Both shipped. In review the `$` marker was judged a poor
fit for Crusty's stated goal of **reusing well-established practice wherever one
exists**: no mainstream language uses `$` for a type argument. It was invented
to sidestep the type-vs-value ambiguity of a bare `method(String)`, but that
invention is exactly what the "adopt existing conventions" rule argues against.

Surveying established call-site generic-argument syntaxes:

| Convention | Languages | Note |
|---|---|---|
| `<T>` angle brackets | C++, C#, Java, TypeScript | drags in the `<`-vs-less-than ambiguity (Rust invented `::<>` to escape it) |
| `[T]` square brackets | Go, Scala, Nim | the modern cohort; no `<`-ambiguity |
| `!(T)` | D | `!` is already macro invocation in Crusty |

Go deliberately chose `[T]` for readability and to avoid the C++ ambiguity. The
project shares Go's aversion to C++ syntax, and — decisively — Crusty **already
uses `[T]` to mean a type** since the prefix slice/array forms (`[int]`,
`[int; N]`). Using `[T]` for type arguments is therefore both an established
convention AND internally consistent.

The collision risk — `[` also starts indexing (`arr[i]`) — was tested, not
assumed. In both call positions the generic form is `method[types](args)`: the
trailing `(` after `]` distinguishes it from `member[index]`, and the parser's
ordered choice (try `method[typelist](args)`, else fall through to member +
index) resolves `obj.items[0]()` correctly as index-then-call, never a generic
call. No regression across the suite.

## Decision

We will spell explicit generic arguments at a call site with **Go-style square
brackets after the method name**, in both the instance and type-scoped
positions, and retire the `$Type` marker:

```c
s.parse[i32]().unwrap()        // -> s.parse::<i32>().unwrap()
matches.get_one[String]("n")   // -> matches.get_one::<String>("n")
m.convert[String, i32](value)  // -> m.convert::<String, i32>(value)
@mem.size_of[i32]()            // -> mem::size_of::<i32>()
@Map.of[int, bool](k, w)       // -> Map::of::<i32, bool>(k, w)
```

- Bracketed types come between the method name and the value arguments,
  comma-separated. The trailing `(` keeps the form distinct from indexing.
- Instance form lowers to `Expression::GenericMethodCall`; type-scoped form to
  `Expression::TurbofishCall` — the same AST nodes ADR-0031/0033 used, so the
  change is a parser-surface swap, not an AST reshape.
- A call with no `[...]` is an ordinary call, unchanged. The Rust turbofish
  `Type::method::<T>(args)` continues to parse (PS-14 `TurbofishCall`).
- The `$Type` surface and its `dollar_type_args` rule are removed.

This supersedes ADR-0031 and ADR-0033 in full.

## Consequences

Easier:
- Explicit generic arguments now use an established, widely-recognised syntax
  (Go/Scala/Nim) instead of a Crusty-invented marker — the reuse-existing-practice
  goal is met.
- `[T]` is internally consistent with the `[int]` / `[int; N]` slice/array type
  forms: `[` already introduces a type in Crusty.
- One spelling across both call positions; no `<`-vs-less-than ambiguity.

Harder / accepted:
- Programs written against the (same-session, unshipped-upstream) `$type` surface
  must migrate to `[T]`. The whole in-repo corpus is migrated with this change.
- `[` is now overloaded (indexing vs generic args), resolved by the trailing-`(`
  lookahead. Proven collision-free across the suite, but it is a lookahead the
  grammar now depends on.
- Generic calls still type as `Auto` — rustc arbitrates the return type
  (unchanged from ADR-0031, consistent with ADR-0001).

## Verification
- [x] `s.parse[i32]().unwrap()` lowers to `s.parse::<i32>().unwrap()`.
- [x] `@mem.size_of[i32]()` lowers to `mem::size_of::<i32>()`.
- [x] `m.convert[String, i32](v)` lowers to `m.convert::<String, i32>(v)`.
- [x] `obj.items[0]()` still parses as index-then-call, not a generic call.
- [x] `a.data[0]` still parses as indexing.
- [x] A `[...]`-free call stays a plain `MethodCall` / `TypeScopedCall`.
- [x] The Rust turbofish `mem::size_of::<i32>()` still parses.
- [x] End-to-end: a program using both `[T]` positions transpiles, compiles
      under `rustc -D unused_parens`, and runs (`size=4 parsed=42`).
- [x] `$Type` and `dollar_type_args` are removed from the parser.
- [x] `cargo test` green; `cargo clippy --all-targets` clean.
- [x] expressions.md documents the `[T]` surface; ADR-0031 and ADR-0033 are
      marked superseded by this ADR.
