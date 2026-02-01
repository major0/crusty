# Tooling Architecture

## Introduction

Crusty includes two developer tools beyond the core transpiler: crustydoc for documentation generation and crustyfmt for code formatting. These tools share the parser and AST infrastructure with the transpiler.

## Rationale

A complete developer experience requires more than just compilation. Documentation generation and code formatting are essential for maintaining code quality and consistency in any project. By building these tools on the same parser infrastructure, they stay in sync with language evolution.

## crustydoc

crustydoc generates documentation from Crusty source code, similar to rustdoc. It leverages rustdoc directly by transpiling to Rust first, then running rustdoc on the generated code. This ensures documentation output is compatible with the Rust ecosystem.

## crustyfmt

crustyfmt formats Crusty source code for consistent style. It uses the parser to build an AST, then the pretty-printer to emit formatted code. This round-trip through the AST ensures that formatting is always syntactically valid.

## Shared Infrastructure

Both tools reuse the core transpiler components:
- Lexer for tokenization
- Parser for AST construction
- Pretty-printer for code emission (crustyfmt)
- Code generator for Rust output (crustydoc)
