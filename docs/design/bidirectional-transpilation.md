# Bidirectional Transpilation

## Introduction

Crusty supports bidirectional transpilation between Crusty and Rust. Beyond the primary Crusty-to-Rust direction, the transpiler can also convert Rust source code back to Crusty syntax. This capability validates that Crusty is a true syntactic layer over Rust.

## Rationale

Bidirectional transpilation serves as the ultimate validation of syntax stability. If code can be transpiled from Crusty to Rust and back without loss, it proves that Crusty's syntax is a complete and faithful representation of Rust's semantics. It also enables adopting existing Rust projects into Crusty syntax.

## Round-Trip Property

The key correctness property is that round-trip transpilation preserves program structure:

```
Crusty source → Rust code → Crusty source (structurally equivalent)
```

While whitespace and formatting may differ, the AST structure and program semantics must be identical after a round trip.

## Syntax Mapping

Reverse translation applies the inverse of Crusty's syntax rules:
- `fn name(a: i32) -> i32` → `int name(int a)`
- `let mut x = 42;` → `var x = 42;`
- `println!("...")` → `__println__("...")`
- `Type::method()` → `@Type.method()`
- `match expr { ... }` → `switch(expr) { ... }`

## Unified AST

Both directions share a single AST representation. The parser produces the same AST whether parsing Crusty or Rust source, and the code generator can emit either syntax from the same AST.
