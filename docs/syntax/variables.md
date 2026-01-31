# Variables

## Introduction

Crusty supports variable declarations using C-style syntax where the type precedes the variable name. The `let` and `var` keywords are optional modifiers — `let` marks immutability (the default), and `var` marks mutability. Constants use the `const` keyword.

## Rationale

Using C-style `Type name = value;` as the primary syntax provides a familiar experience for C developers. The `var` keyword makes mutability explicit and obvious. Type inference is available through `let`/`var` without a type, leveraging Rust's inference capabilities.

## Examples

### C-Style Declarations (Primary)
```c
int x = 42;              // Immutable (implicit let)
float pi = 3.14;         // Immutable
var int count = 0;       // Mutable
count = count + 1;
```

### Type Inference
```c
let x = 42;              // Inferred as int
var count = 0;           // Mutable, inferred as int
```

### Constants
```c
const int MAX_SIZE = 1024;   // Explicit type
const MAX_SIZE = 1024;       // Type inference
```

## Formal Grammar

```ebnf
let_decl   = ["let"] type IDENT "=" expr ";"
           | "let" IDENT "=" expr ";" ;
var_decl   = "var" [type] IDENT "=" expr ";" ;
const_decl = "const" [type] IDENT "=" expr ";" ;
```
