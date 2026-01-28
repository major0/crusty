# Semantic Analyzer

## Introduction

The semantic analyzer is the third phase of the Crusty compiler. It validates the AST for type correctness, scope rules, and language constraints before code generation.

## Rationale

Semantic analysis catches errors that the parser cannot detect, such as type mismatches, undefined variables, and use of unsupported C features. By performing these checks before code generation, the compiler provides clear Crusty-level error messages rather than confusing Rust compiler errors.

## Examples

### Type Checking
```c
int x = "hello";  // Error: cannot assign string to int
```

### Undefined Variable
```c
int y = x + 1;  // Error: undefined variable 'x'
```

### Unsupported Features
```c
union Data { int i; float f; };  // Error: C unions are not supported
goto label;                       // Error: goto is not supported
#include <stdio.h>                // Error: #include is not supported
```

## Checks Performed

- **Type compatibility**: Binary operations, assignments, function arguments
- **Scope validation**: Variables declared before use, proper scoping
- **Function signatures**: Parameter count and types match declarations
- **Unsupported features**: Reject C unions, goto, #include
- **Mutability**: Ensure mutable operations only on `var` bindings
