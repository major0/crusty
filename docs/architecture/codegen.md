# Code Generator

## Introduction

The code generator transforms the validated AST into target language source code (Rust or Crusty).

## Rationale

Separating code generation from parsing and analysis allows the same AST to be emitted in different formats.

## Output Modes

### Rust Output
Generate valid Rust source code from Crusty AST:
- Transform C-style syntax to Rust syntax
- Convert type names (int → i32, etc.)
- Transform control flow (for loops, labels)
- Generate macro_rules! from #define

### Crusty Output
Generate Crusty source code from Rust AST:
- Transform Rust syntax to C-style syntax
- Convert type names (i32 → int, etc.)
- Transform control flow

### AST Output
Dump AST in human-readable format for debugging.

## Pretty Printer

The pretty printer formats generated code:
- Consistent indentation
- Line wrapping
- Comment preservation
- Configurable style

```rust
pub struct PrettyPrinter {
    indent_size: usize,
    max_line_width: usize,
}

impl PrettyPrinter {
    pub fn print_file(&self, file: &File) -> String;
    pub fn print_item(&self, item: &Item) -> String;
    pub fn print_statement(&self, stmt: &Statement) -> String;
    pub fn print_expression(&self, expr: &Expression) -> String;
}
```

## Syntax Transformations

| Crusty | Rust |
|--------|------|
| `int x = 42;` | `let x: i32 = 42;` |
| `void foo()` | `fn foo()` |
| `int foo()` | `fn foo() -> i32` |
| `.label: loop` | `'label: loop` |
| `break .label` | `break 'label` |
| `#define M(x) ...` | `macro_rules! M { ... }` |
