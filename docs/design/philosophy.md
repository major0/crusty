# Design Philosophy

## Introduction

Crusty is a C-like programming language that transpiles to Rust. It aims to provide the familiar syntax of C while leveraging Rust's safety guarantees, type system, and ecosystem.

## Rationale

Many developers are comfortable with C syntax but want Rust's safety features. Rather than learning an entirely new syntax, Crusty lets developers write in a C-like style and get valid, safe Rust code. The transpiler handles the mechanical translation, letting developers focus on logic rather than syntax.

## Core Principles

### C-Like Function Syntax
Crusty uses C-style function declarations with return types before function names. There is no `fn` keyword — functions are declared as `int add(int a, int b)` rather than Rust's `fn add(a: i32, b: i32) -> i32`. The `static` keyword controls visibility: static functions are private, non-static functions are public.

### Syntax-Only Transpilation
Crusty is a syntax layer over Rust, not a semantic transformation. The transpiler translates syntax constructs one-to-one without changing program semantics. What you write in Crusty maps directly to what you get in Rust.

Method names, function names, and identifiers pass through unchanged between Crusty and Rust. This preserves bidirectional transpilation (Crusty ↔ Rust), avoids conflicts with user-defined functions, and provides transparent mapping between languages.

Allowed syntax transformations:
- Type syntax: `int` → `i32`, `float` → `f64`, `void` → `()`
- Error types: `Type?` → `Result<Type, Box<dyn std::error::Error>>`
- Error propagation: `expr?` → `expr?`
- Type-scoped calls: `@Type.method()` → `Type::method()`
- Loop labels: `.label:` → `'label:`
- NULL keyword: `NULL` → `Option::None` (the only semantic transformation)

What is NOT transformed: method names (`.is_err()` stays `.is_err()`), function names (`Ok()` stays `Ok()`), and all user-defined identifiers pass through unchanged.

### Safety First
C features that cannot be safely represented in Rust are rejected at compile time:
- **No unions** — Use Rust enums instead
- **No goto** — Use structured control flow
- **No #include** — Use Crusty's module system

### Rust Standard Library
Crusty programs use Rust's standard library directly. There are no wrapper types or compatibility layers. `println!`, `Vec`, `String`, and all other std types work as-is.

### Escape Hatch
When Crusty's syntax doesn't cover a Rust feature, the `rust!` macro allows embedding raw Rust code directly. This ensures developers are never blocked by transpiler limitations.

## Unsupported C Features

| Feature | Reason | Alternative |
|---------|--------|-------------|
| `union` | Violates Rust memory safety | Use `enum` |
| `goto` | No Rust equivalent | Use loops with `break`/`continue` |
| `#include` | Incompatible with Rust modules | Use `#import`/`#export` |
| Raw pointers (unrestricted) | Unsafe by default | Use references, or `unsafe` blocks |
