# Code Generator

## Introduction

The code generator is the fourth phase of the Crusty compiler. It traverses the AST and emits Rust source code, translating Crusty syntax to equivalent Rust constructs.

## Rationale

Generating Rust source code (rather than LLVM IR or machine code) leverages the Rust compiler's optimization and safety checking. The generated code is formatted with `prettyplease` to ensure it follows Rust style conventions.

## Examples

### Function Translation
```c
// Crusty
static int double(int x) {
    return x * 2;
}
```
```rust
// Generated Rust
fn double(x: i32) -> i32 {
    x * 2
}
```

### Struct Translation
```c
// Crusty
struct Point {
    int x;
    int y;
}
```
```rust
// Generated Rust
pub struct Point {
    pub x: i32,
    pub y: i32,
}
```

## Translation Rules

| Crusty | Rust |
|--------|------|
| `void` return | No return annotation |
| `static` function | Private (no `pub`) |
| Non-static function | `pub fn` |
| `.label:` loops | `'label:` loops |
| `break .label` | `break 'label` |
| C-style cast `(Type)expr` | `expr as Type` |
| `sizeof(Type)` | `std::mem::size_of::<Type>()` |
| `NULL` | `Option::None` |
| Ternary `a ? b : c` | `if a { b } else { c }` |
