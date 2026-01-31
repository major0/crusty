# C-Style Variable Declarations

## Introduction

Crusty uses C-style variable declaration syntax where `Type name = value;` is the primary form. The `let` and `var` keywords are optional modifiers — when neither is specified, `let` (immutable) is assumed. This makes `int x = 42;` equivalent to `let int x = 42;`.

## Rationale

Rust-style type annotations (`let x: Type = value`) are inconsistent with Crusty's C-like design philosophy. In C, types precede variable names in declarations (`int x = 42;`). Crusty adopts this as the primary syntax while preserving Rust's type inference capability through the `let`/`var` keywords.

## Declaration Styles

### C-Style (Primary)
```c
int x = 42;              // Immutable (implicit let)
var int x = 42;          // Mutable (explicit var)
const int MAX = 100;     // Constant with explicit type
```

### Explicit Let with Type
```c
let int x = 42;          // Equivalent to int x = 42;
```

### Type Inference
```c
let x = 42;              // Immutable, type inferred
var x = 42;              // Mutable, type inferred
const MAX = 100;         // Constant, type inferred
```

### With Typedef Aliases
```c
typedef int MyInt;
MyInt x = 32;            // C-style with typedef
let MyInt y = 32;        // Explicit let with typedef
var MyInt z = 32;        // Mutable with typedef
```

## What's Not Supported
```c
let x: int = 42;         // ❌ Rust-style colon annotation
var y: float = 3.14;     // ❌ Rust-style colon annotation
const MAX: int = 100;    // ❌ Rust-style colon annotation
```

## Scope of Change

Type annotations are removed from all variable declarations:
- `let` statements — no colon annotations
- `var` statements — no colon annotations
- `const` statements — no colon annotations

Type annotations are preserved where structurally required:
- Function parameters: `int add(int a, int b)` — types are part of the declaration syntax
- Return types: `int add(...)` — the return type precedes the function name
- Struct fields: `typedef struct { int x; int y; } Point;` — fields require types

## Key Design Rule

When neither `let` nor `var` is specified, `let` (immutable) is assumed. This means:
- `int x = 42;` is immutable (same as `let int x = 42;`)
- `var int x = 42;` is mutable
- Mutability must always be explicitly requested with `var`

## Parser Disambiguation

The parser uses lookahead to distinguish between:
- `int x = 42;` — declaration (pattern: Type Identifier `=`)
- `int(x)` — cast expression
- `int + 5` — expression

## Examples

### Migration from Rust-style
```c
// Before (rejected)
let x: int = 42;
var y: float = 3.14;

// After (C-style primary)
int x = 42;
var float y = 3.14;

// After (inference alternative)
let x = 42;
var y = 3.14;
```
