# Error Handling Architecture

## Introduction

The Crusty compiler uses a structured, hierarchical error handling system that provides clear, actionable error messages with precise source location information. Errors are organized by compilation phase and designed for extensibility.

## Rationale

Precise error reporting is critical for developer experience. The architecture follows Rust best practices with strong typing to prevent error category confusion, and supports reporting multiple errors in a single pass (particularly during semantic analysis).

## Error Hierarchy

```
CompilerError (top-level enum)
├── Lex(LexError)              - Lexical analysis errors
├── Parse(ParseError)          - Syntax parsing errors
├── Semantic(Vec<SemanticError>) - Semantic analysis errors (multiple)
├── CodeGen(CodeGenError)      - Code generation errors
├── Io(std::io::Error)         - File I/O errors
└── RustcInvocation(String)    - Rust compiler invocation errors
```

Each error type carries a `Position` (line and column, 1-indexed) or `Span` (start and end positions) for source location tracking.

## Error Kinds

### Lexer Errors
- `UnexpectedCharacter` — Invalid character in source
- `UnterminatedString` — String literal missing closing quote
- `UnterminatedComment` — Block comment missing closing `*/`
- `InvalidNumber` — Malformed numeric literal

### Parse Errors
- `UnexpectedToken` — Token doesn't match expected grammar
- `ExpectedExpression` — Expression expected but not found
- `ExpectedType` — Type annotation expected
- `UnsupportedFeature` — C feature not supported in Crusty (unions, goto, #include)

### Semantic Errors
- `UndefinedVariable` — Variable not found in scope
- `TypeMismatch` — Expected and actual types differ
- `DuplicateDefinition` — Symbol already defined in scope
- `InvalidOperation` — Operation not valid for given types

## Error Message Guidelines

Error messages follow these principles:
- Be specific: "expected ';' after statement" not "syntax error"
- Provide context: "expected i32, found String" not "type error"
- Reference Crusty concepts: "#include directives are not supported. Use #import instead"
- Use consistent terminology: "expected"/"found" for parse errors, "not found in this scope" for undefined variables
