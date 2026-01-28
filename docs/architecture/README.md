# Architecture Documentation

This section documents the Crusty compiler architecture.

## Topics

- [Overview](overview.md) - Compiler architecture overview
- [Lexer](lexer.md) - Lexical analysis
- [Parser](parser.md) - Syntax parsing
- [AST](ast.md) - Abstract Syntax Tree
- [Semantic Analyzer](semantic.md) - Semantic analysis
- [Code Generator](codegen.md) - Code generation

## Compiler Pipeline

```
Source Code → Lexer → Parser → AST → Semantic Analyzer → Code Generator → Output
```
