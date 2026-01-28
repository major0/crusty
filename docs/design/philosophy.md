# Design Philosophy

## Introduction

Crusty is designed to provide a comfortable C-like syntax while maintaining full Rust compatibility and safety guarantees.

## Goals

### Primary Goals
- Provide familiar C-style syntax for developers
- Generate valid, idiomatic Rust code
- Maintain Rust's safety guarantees
- Enable gradual migration from C to Rust

### Non-Goals
- Full C compatibility (some features are intentionally excluded)
- Runtime overhead (transpilation is purely syntactic)
- Custom standard library (use Rust's std directly)

## Design Decisions

### Syntax-Only Transpilation
Crusty is a syntax transformation layer, not a semantic transformation. The generated Rust code should be readable and maintainable.

### Rust Standard Library
Crusty programs use Rust's standard library directly. There are no Crusty-specific wrappers or abstractions.

### Safety First
C features that would violate Rust's safety guarantees are rejected at compile time rather than generating unsafe code.

### Escape Hatch
When Crusty syntax is insufficient, developers can embed raw Rust code using `__rust__ { }` blocks.

## Trade-offs

| Decision | Benefit | Cost |
|----------|---------|------|
| C-style syntax | Familiar to C developers | Some Rust idioms harder to express |
| No manual memory | Safety guaranteed | Can't do low-level memory tricks |
| No null pointers | No null pointer bugs | Must use Option explicitly |
| No goto | Structured control flow | Some algorithms harder to express |
