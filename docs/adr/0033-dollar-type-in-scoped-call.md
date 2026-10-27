# ADR-0033: $type generic arguments in the type-scoped call position

## Status
superseded by ADR-0034

## Context

ADR-0031 introduced `$Type` generic arguments for **instance** method calls
(`recv.method($T, args)` → `recv.method::<T>(args)`), and explicitly deferred to
a follow-up ADR whether the same `$type` surface should extend to the
**type-scoped** call position — the Rust turbofish `@mem.size_of::<i32>()` /
`Type::method::<T>(args)` (PS-14, `Expression::TurbofishCall`).

The gap analysis against ~90% direct-Rust coverage found that after ADR-0031,
the type-scoped generic call had **no `$type` surface at all**: `@mem.size_of($i32)`
was a parse error, leaving `::<>` (the C++-ism the project set out to avoid) or
`__rust__` as the only ways to spell it. That is the inconsistency this ADR
resolves: `$type` should mean the same thing in both call positions.

## Decision

We will accept `$Type` leading generic arguments in the `@`-form type-scoped
call, with the same shape and lowering as ADR-0031's instance form:

```c
@mem.size_of($i32)            // -> mem::size_of::<i32>()
@mem.size_of($i64)            // -> mem::size_of::<i64>()
@Map.of($int, $bool, k, v)    // -> Map::of::<i32, bool>(k, v)
```

- One or more leading `$Type` arguments, comma-separated, followed by the
  ordinary value arguments — identical to the instance-method rule.
- The `$`-marked arguments are the method's turbofish type arguments, so the
  form lowers to the **existing** `Expression::TurbofishCall`
  (`ty::method::<G1, G2>(args)`). No new AST node and no new codegen path.
- Tried before the plain `@Type.method(args)` form so a leading `$` wins; a
  type-scoped call with no `$` is unchanged (`TypeScopedCall`).
- Like all `$type` calls, it types as `Auto` — rustc arbitrates (ADR-0001).

The Rust-syntax turbofish `Type::method::<T>(args)` (PS-14) is **retained**: this
ADR adds the `$type` spelling to the `@`-form, it does not remove the `::<>`
form. Both parse.

## Consequences

Easier:
- `$type` now means the same thing in both call positions (instance and
  type-scoped), so a Crusty programmer learns one generic-argument surface.
- The type-scoped generic call is reachable without `::<>` or `__rust__`,
  closing a 90%-coverage gap.
- Zero new AST/codegen: the form reuses `TurbofishCall` and `dollar_type_args`.

Harder / accepted:
- The `@`-form still requires the type to be a bare identifier or `Ident<…>`
  (`type_for_scoped_call`); a `::`-path base such as `@std::mem.size_of($i32)`
  is not accepted — bring the module into scope with `#import` and write
  `@mem.size_of($i32)`. This is a pre-existing `@`-form limitation, unchanged.
- Two surfaces now spell the same type-scoped generic call (`@mem.size_of($i32)`
  and `mem::size_of::<i32>()`). Retiring `::<>` entirely for one uniform syntax
  remains deferred — it is a larger, separate decision (it would touch PS-14 and
  the whole corpus) and this ADR deliberately does not take it.

## Verification
- [x] `@mem.size_of($i32)` parses to `Expression::TurbofishCall` and lowers to
      `mem::size_of::<i32>()`.
- [x] A multi-generic, with-value-args form `@Map.of($int, $bool, k, v)` lowers
      to `Map::of::<i32, bool>(k, v)`.
- [x] A `$`-less `@mem.size_of()` still parses to `TypeScopedCall` (no shadowing).
- [x] The existing `mem::size_of::<i32>()` turbofish form still parses unchanged.
- [x] End-to-end: a program using `@mem.size_of($i32)` transpiles, compiles under
      `rustc -D unused_parens`, and runs (`i32=4 i64=8`).
- [x] `cargo test` green; `cargo clippy --all-targets` clean.
- [x] expressions.md documents the type-scoped `$type` form.
