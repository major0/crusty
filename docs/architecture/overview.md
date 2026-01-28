# Compiler Architecture Overview

## Introduction

The Crusty compiler (crustyc) is a bidirectional transpiler that translates between Crusty (a C-like language) and Rust source code.

## Rationale

A multi-phase architecture provides clear separation of concerns and enables bidirectional transpilation through a shared AST representation.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         crustyc CLI                          │
│  (Command-line interface, file I/O, option parsing)         │
└──────────────────┬──────────────────────────────────────────┘
                   │
                   ├──────────────────┐
                   │                  │
         ┌─────────▼────────┐  ┌─────▼──────────┐
         │  Crusty Parser   │  │  Rust Parser   │
         │  (Custom parser) │  │  (syn library) │
         └─────────┬────────┘  └─────┬──────────┘
                   │                  │
                   └────────┬─────────┘
                            │
                   ┌────────▼─────────┐
                   │   Unified AST    │
                   │  (Shared repr.)  │
                   └────────┬─────────┘
                            │
                   ┌────────▼──────────┐
                   │ Semantic Analyzer │
                   │ (Type checking,   │
                   │  scope resolution)│
                   └────────┬──────────┘
                            │
                   ┌────────▼──────────┐
                   │  Code Generator   │
                   │ (Crusty/Rust emit)│
                   └────────┬──────────┘
                            │
                   ┌────────▼──────────┐
                   │   Pretty Printer  │
                   │ (Format output)   │
                   └────────┬──────────┘
                            │
                   ┌────────▼──────────┐
                   │  rustc Invoker    │
                   │  (Optional)       │
                   └───────────────────┘
```

## Compilation Phases

1. **Lexical Analysis**: Tokenize source code into a stream of tokens
2. **Parsing**: Build Abstract Syntax Tree (AST) from tokens
3. **Semantic Analysis**: Validate types, scopes, and language rules
4. **Code Generation**: Emit target language source code
5. **Compilation**: Invoke rustc to produce binaries (optional)

## Bidirectional Flow

**Crusty → Rust:**
```
Crusty Source → Crusty Parser → AST → Semantic Analysis → Code Generator → Rust Source → rustc → Binary
```

**Rust → Crusty:**
```
Rust Source → Rust Parser (syn) → AST → Semantic Analysis → Code Generator → Crusty Source
```

## Key Design Principles

- **Shared AST**: Use a unified AST representation that can represent both Crusty and Rust constructs
- **C-like Function Syntax**: Crusty uses C-style function declarations with return types before function names (e.g., `int main()`, `void foo()`), NOT Rust's `fn` keyword syntax
- **Rust Standard Library**: Crusty programs use Rust's std library directly without wrappers
- **Safety First**: Reject C features that violate Rust's safety guarantees
- **Familiar Syntax**: Provide C-like syntax that maps cleanly to Rust semantics
- **Escape Hatch**: Support `__rust__ { }` blocks for embedding raw Rust code when needed
