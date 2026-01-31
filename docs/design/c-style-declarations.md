# C-Style Variable Declarations

## Introduction

Crusty uses C-style variable declarations without Rust-style type annotations. Variables use type inference or C-style casting instead of the `let x: Type = value` pattern. This aligns with Crusty's philosophy of providing a C-like syntax layer over Rust.

## Rationale

Rust-style type annotations (`let x: Type = value`) are inconsistent with Crusty's C-like design philosophy. In C, types are specified through declarations (`int x = 42;`) or casts (`(int)value`), not through post-name annotations. Removing Rust-style annotations makes Crusty more consistent and familiar to C developers.

## Variable Declaration Styles

### Type Inference (Preferred)
```c
let x = 42;           // Inferred as int
let y = 3.14;         // Inferred as float
var count = 0;        // Mutable, inferred as int
```

### C-Style Casting (When Explicit Type Needed)
```c
typedef int MyInt;
let x = (MyInt)42;    // Explicit type via cast
let y = (float)42;    // Cast to float
var z = (MyInt)0;     // Mutable with explicit type
```

### Constants
```c
const MAX = (int)100;     // C-style cast for constants
const PI = 3.14159;      // Type inference for constants
```

### What's Not Supported
```c
let x: int = 42;         // ❌ Rust-style annotation — rejected by parser
var y: float = 3.14;     // ❌ Rust-style annotation — rejected by parser
const MAX: int = 100;    // ❌ Rust-style annotation — rejected by parser
```

## Scope of Change

Type annotations are removed from all variable declarations:
- `let` statements — no type annotations
- `var` statements — no type annotations
- `const` statements — no type annotations

Type annotations are preserved where they are structurally required:
- Function parameters: `int add(int a, int b)` — types are part of the declaration syntax
- Return types: `int add(...)` — the return type precedes the function name
- Struct fields: `typedef struct { int x; int y; } Point;` — fields require types

## Examples

### Before (Rust-style)
```c
let x: int = 42;
let y: MyInt = 10;
var z: float = 3.14;
const MAX: int = 100;
```

### After (C-style)
```c
let x = 42;              // Type inference
let y = (MyInt)10;       // C-style cast
var z = 3.14;            // Type inference
const MAX = (int)100;    // C-style cast
```
