# Crusty Documentation

Crusty is a C-like programming language that transpiles to Rust. It provides familiar C syntax while leveraging Rust's safety guarantees and ecosystem.

## Documentation Categories

- [Syntax](syntax/README.md) - Language syntax reference
- [Architecture](architecture/README.md) - Compiler architecture and components
- [Design](design/README.md) - Design decisions and rationale

## Quick Start

Crusty programs use C-like syntax that maps cleanly to Rust semantics:

```crusty
// Hello World in Crusty
void main() {
    println("Hello, World!");
}
```

## Key Features

- C-like syntax for familiar development experience
- Direct transpilation to Rust source code
- Full access to Rust's standard library
- Safety guarantees inherited from Rust
- Bidirectional transpilation (Crusty ↔ Rust)

## License

MIT License - See LICENSE.txt for details.
