# Parser

## Introduction

The parser transforms a token stream into an Abstract Syntax Tree (AST) representing the program structure.

## Rationale

A recursive descent parser provides clear, maintainable code that closely mirrors the grammar.

## Parsing Strategy

The parser uses recursive descent with lookahead for disambiguation. Key parsing functions:

- `parse_file()` - Parse a complete source file
- `parse_item()` - Parse top-level items (functions, structs, etc.)
- `parse_statement()` - Parse statements within blocks
- `parse_expression()` - Parse expressions with operator precedence

## Interface

```rust
pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(source: &'a str) -> Self;
    pub fn parse_file(&mut self) -> Result<File, ParseError>;
}
```

## Error Handling

Parse errors include:
- Location (line, column)
- Expected token(s)
- Actual token found
- Context (what was being parsed)

## Rust Parser

For bidirectional transpilation, a separate Rust parser uses the `syn` library:

```rust
pub struct RustParser;

impl RustParser {
    pub fn parse_file(source: &str) -> Result<File, syn::Error>;
}
```

The Rust parser converts `syn` AST nodes to the unified Crusty AST representation.
