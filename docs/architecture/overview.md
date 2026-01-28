# Compiler Architecture Overview

## Introduction

The Crusty compiler (`crustyc`) is a bidirectional transpiler that translates between Crusty (a C-like language) and Rust source code. It follows a traditional multi-phase compiler architecture with a shared AST that supports both source and target languages.

## Rationale

A shared AST enables bidirectional transpilation: the same internal representation can be produced from either Crusty or Rust source, and emitted as either language. This design supports round-trip validation and interoperability between the two languages.

## Pipeline

The compiler processes source code through five phases:

1. **Lexical Analysis** — Tokenize source code into a stream of tokens
2. **Parsing** — Build an Abstract Syntax Tree (AST) from the token stream
3. **Semantic Analysis** — Validate types, scopes, and language rules
4. **Code Generation** — Emit target language source code from the AST
5. **Compilation** — Optionally invoke `rustc` to produce binaries

```
Source (.crst) → Lexer → Parser → Semantic Analyzer → Code Generator → Target (.rs)
                                                                         ↓
                                                                       rustc → Binary
```

## Design Principles

- **Shared AST**: A unified AST representation handles both Crusty and Rust constructs
- **Rust Standard Library**: Crusty programs use Rust's std library directly, no wrappers
- **Safety First**: C features that violate Rust's safety guarantees are rejected
- **Familiar Syntax**: C-like syntax maps cleanly to Rust semantics
- **Escape Hatch**: The `rust!` macro allows embedding raw Rust code when needed

## Bidirectional Transpilation

The compiler supports two directions:

- **Crusty → Rust**: Parse Crusty source, emit Rust code, optionally compile with `rustc`
- **Rust → Crusty**: Parse Rust source (via `syn` crate), emit Crusty code

Both directions share the same AST, semantic analyzer, and code generation infrastructure.
