# Variables

## Introduction

Crusty supports variable declarations using `let` (immutable) and `var` (mutable) keywords, along with `const` for compile-time constants. These map directly to Rust's `let`, `let mut`, and `const` bindings.

## Rationale

Using `var` for mutable variables provides a clear visual distinction from immutable `let` bindings, following the principle that mutability should be explicit and obvious.

## Examples

### Immutable Binding
```c
let x = 42;
```

### Mutable Binding
```c
var count = 0;
count = count + 1;
```

### Constants
```c
const MAX_SIZE = 1024;
```

## Formal Grammar

```ebnf
let_decl   = "let" IDENT "=" expr ";" ;
var_decl   = "var" IDENT "=" expr ";" ;
const_decl = "const" IDENT "=" expr ";" ;
```
