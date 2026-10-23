# Bidirectional Transpilation (Planned)

> **Status: not implemented.** Reverse (Rust → Crusty) transpilation is a
> planned, aspirational capability — there is no `--from-lang=rust` handling and
> no Rust-to-Crusty parser in the shipped compiler. The critical path is
> Crusty → Rust transpilation and coexistence with Rust crates. This document
> records the intended design for the reverse direction, not current behaviour.

## Introduction

The long-term goal is for Crusty to support bidirectional transpilation: beyond
the primary Crusty-to-Rust direction, the transpiler would also convert Rust
source back to Crusty syntax. This capability would validate that Crusty is a
true syntactic layer over Rust.

## Rationale

Bidirectional transpilation would serve as the ultimate validation of syntax
stability. If code can be transpiled from Crusty to Rust and back without loss,
it proves that Crusty's syntax is a complete and faithful representation of
Rust's semantics. It would also enable adopting existing Rust projects into
Crusty syntax.

## Round-Trip Property

The intended correctness property is that round-trip transpilation preserves
program structure:

```
Crusty source → Rust code → Crusty source (structurally equivalent)
```

While whitespace and formatting may differ, the AST structure and program
semantics should be identical after a round trip.

## Syntax Mapping

Reverse translation would apply the inverse of Crusty's syntax rules:
- `fn name(a: i32) -> i32` → `int name(int a)`
- `let mut x = 42;` → `var x = 42;`
- `println!("...")` → `println!("...")` (macro calls pass through unchanged)
- `Type::method()` → `@Type.method()`
- `match expr { ... }` → `switch(expr) { ... }`

## Unified AST

The design would share a single AST representation across both directions: the
parser would produce the same AST whether parsing Crusty or Rust source, and
the code generator would emit either syntax from the same AST. The current
compiler implements only the Crusty-source half of this.
