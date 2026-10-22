# Compiler Architecture Overview

## Introduction

The Crusty compiler (`crustyc`) is a transpiler that translates Crusty (a C-like
language) into Rust source code. It follows a traditional multi-phase compiler
architecture built around a single AST. The AST is designed so a future reverse
(Rust → Crusty) direction could share it, but only the Crusty → Rust direction
is implemented.

## Rationale

A single AST keeps the pipeline simple and makes the generated Rust predictable
from the Crusty source. The same representation is intended to later support a
reverse direction for round-trip validation and interoperability, which is
planned but not yet built.

## Pipeline

The compiler processes source code through four phases:

1. **Parsing** — Build an Abstract Syntax Tree (AST) directly from Crusty source
   text using a PEG grammar (the `rust-peg` crate); there is no separate lexing
   phase
2. **Semantic Analysis** — Validate types, scopes, and language rules
3. **Code Generation** — Emit Rust source code from the AST
4. **Compilation** — Optionally invoke `rustc` to produce binaries

```
Source (.crst) → Parser → Semantic Analyzer → Code Generator → Target (.rs)
                                                                 ↓
                                                               rustc → Binary
```

## Design Principles

- **Shared AST**: A unified AST representation handles both Crusty and Rust constructs
- **Rust Standard Library**: Crusty programs use Rust's std library directly, no wrappers
- **Safety First**: C features that violate Rust's safety guarantees are rejected
- **Familiar Syntax**: C-like syntax maps cleanly to Rust semantics
- **Escape Hatch**: The `rust!` macro allows embedding raw Rust code when needed

## Bidirectional Transpilation (Planned)

Only the **Crusty → Rust** direction is implemented: parse Crusty source, emit
Rust code, optionally compile with `rustc`.

A **Rust → Crusty** direction (parsing Rust via the `syn` crate and emitting
Crusty from the shared AST) is a planned, aspirational capability — it is not
built, and there is no `--from-lang=rust` handling in the compiler. See
[../design/bidirectional-transpilation.md](../design/bidirectional-transpilation.md).
