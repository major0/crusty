# Transformation Summary

An at-a-glance map of how Crusty surface forms lower to Rust. Each topic page
documents its forms in full with grammar and worked examples; this page is the
quick index. Crusty is a **syntax-layer** transpiler (ADR-0001): almost every
form lowers to a direct Rust counterpart and inherits its semantics. `NULL` is
the single exception.

## Pure syntax transformations

| Crusty | Rust | Reference |
|--------|------|-----------|
| `int`, `float` | `i32`, `f64` | [types.md](types.md) |
| `int8`…`int128`, `uint8`…`uint128` | `i8`…`i128`, `u8`…`u128` | [types.md](types.md) |
| `size_t`, `ssize_t`, `byte`, … (C/POSIX aliases) | `usize`, `isize`, `u8`, … | [types.md](types.md) |
| `long`, `unsigned int`, … (multi-word C) | `i64`, `u32`, … (LP64) | [types.md](types.md) |
| `Type name = v;` | `let name: Type = v;` | [variables.md](variables.md) |
| `var name = v;` | `let mut name = v;` | [variables.md](variables.md) |
| `Type?` | `Result<Type, Box<dyn std::error::Error>>` | [error-handling.md](error-handling.md) |
| `expr?` | `expr?` (pass through) | [error-handling.md](error-handling.md) |
| `@Type.method()` | `Type::method()` | [expressions.md](expressions.md) |
| `@Type(G).method()` | `Type::<G>::method()` | [expressions.md](expressions.md) |
| `.label:` / `break .label;` | `'label:` / `break 'label;` | [control-flow.md](control-flow.md) |
| `name!(...)` | `name!(...)` (pass through) | [macros.md](macros.md) |
| `struct P(int, int);` | `struct P(i32, i32);` | [types.md](types.md) |
| `struct Marker;` | `struct Marker;` | [types.md](types.md) |
| `Circle(f64)` / `Rect { f64 w; }` (enum payloads) | `Circle(f64)` / `Rect { w: f64 }` | [types.md](types.md) |
| `dyn Trait` / `impl Trait` | `dyn Trait` / `impl Trait` (verbatim) | [types.md](types.md) |
| `#[...]` / `#![...]` | `#[...]` / `#![...]` (verbatim) | [attributes.md](attributes.md) |
| `#import module;` | `use module;` | [modules.md](modules.md) |
| `#export module.symbol;` | `pub use module::symbol;` | [modules.md](modules.md) |
| `__rust__{ … }` | `…` (raw pass-through) | [expressions.md](expressions.md) |
| `__thread__ { … }` | `std::thread::spawn(move \|\| { … })` | [concurrency.md](concurrency.md) |
| `__scope__ { … }` | `std::thread::scope(\|__crusty_scope\| { … })` | [concurrency.md](concurrency.md) |

## Control-flow lowerings

| Crusty | Rust | Reference |
|--------|------|-----------|
| `for (int i = 0; i < n; ++i)` | `for i in 0..n` | [control-flow.md](control-flow.md) |
| `for i in seq { … }` | `for i in seq { … }` | [control-flow.md](control-flow.md) |
| `switch (e) { case … }` | `match e { … }` | [control-flow.md](control-flow.md) |
| `match (e) { … }` | `match e { … }` (scrutinee parens dropped) | [control-flow.md](control-flow.md) |
| `if (int i = e; cond) { … }` | `{ let i = e; if cond { … } }` | [control-flow.md](control-flow.md) |
| `while (var i = e; cond) { … }` | `{ let mut i = e; while cond { … } }` | [control-flow.md](control-flow.md) |

## Definition directives

| Crusty | Rust | Reference |
|--------|------|-----------|
| `#define __C__ lit` | `const C: … = lit;` | [macros.md](macros.md) |
| `#macro __M__(a) body` | `macro_rules! m { … }` | [macros.md](macros.md) |
| `#ifdef NAME` … `#elif` … `#else` … `#endif` | per-item `#[cfg(...)]` attributes | [macros.md](macros.md) |

## Semantic transformation (the only one)

`NULL` is the **single semantic** transformation — a C keyword with no direct
Rust equivalent, mapped to `Option`:

| Crusty | Rust |
|--------|------|
| `NULL` | `Option::None` |
| `ptr == NULL` | `ptr.is_none()` |
| `ptr != NULL` | `ptr.is_some()` |

## Pass through unchanged

Method names (`.is_err()`, `.is_ok()`, `.unwrap()`), function names (`Ok()`,
`Err()`), user-defined identifiers, and all Rust operators and expressions are
emitted unchanged — Crusty does not rename or wrap the Rust API.
