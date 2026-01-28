# Design Documentation

This section documents design decisions and rationale for the Crusty compiler.

## Topics

- [Philosophy](philosophy.md) - Design philosophy and goals
- [C Compatibility](c-compatibility.md) - C features supported and excluded
- [Rust Integration](rust-integration.md) - How Crusty integrates with Rust

## Design Principles

1. **Familiar Syntax**: C-like syntax for reduced learning curve
2. **Rust Safety**: Inherit Rust's memory safety guarantees
3. **Direct Mapping**: Syntax should map cleanly to Rust constructs
4. **No Magic**: Transparent transpilation without hidden behavior
5. **Escape Hatch**: Allow raw Rust when needed
