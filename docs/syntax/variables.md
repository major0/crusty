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

`const` is a **statement-level** declaration (inside a function body). For a
module-level (top-level) constant, use `static` instead (see
[statics.md](statics.md)) — a top-level `const` does not parse.

## Formal Grammar

```ebnf
cstyle_decl = type IDENT ["=" expr] ";"             ; C-style, initializer optional
let_decl    = "let" [type] IDENT ["=" expr] ";"     ; initializer optional
var_decl    = "var" [type] IDENT ["=" expr] ";"     ; initializer optional
const_decl  = "const" [type] IDENT "=" expr ";"     ; initializer REQUIRED
```

A `let`/`var`/C-style declaration may omit the initializer (`int x;`, `let x;`,
`var int count;`). A `const` must be initialized.
