# Requirements: C-Style Variable Declarations

## Overview
Implement C-style variable declaration syntax in Crusty, where `Type name = value;` is the primary syntax and `let`/`var` keywords are optional modifiers. This maintains C-like philosophy while preserving Rust's type inference capability.

## Background
Crusty should use C-natural syntax as the primary way to declare variables. The `let` keyword exists specifically for:
1. Explicitly marking immutability when using type inference
2. Allowing Rust's type inference feature (which C doesn't have)

When neither `let` nor `var` is specified, `let` (immutable) is assumed, making `int x = 42;` equivalent to `let int x = 42;`.

## Design Philosophy

### Primary Syntax (C-style)
```c
int x = 42;              // Immutable (implicit let)
var int x = 42;          // Mutable (explicit var)
const int MAX = 100;     // Constant
```

### Alternative Syntax (Type Inference)
```c
let x = 42;              // Immutable with inference
var x = 42;              // Mutable with inference
const MAX = 100;         // Constant with inference
```

### Complete Syntax Options
```c
// All valid immutable declarations:
int x = 42;              // C-style (implicit let)
let int x = 42;          // Explicit let with type
let x = 42;              // Explicit let with inference

// All valid mutable declarations:
var int x = 42;          // Explicit var with type
var x = 42;              // Explicit var with inference

// All valid constant declarations:
const int MAX = 100;     // Explicit type
const MAX = 100;         // Type inference
```

### Key Rule
**If neither `let` nor `var` is specified, `let` (immutable) is assumed.**

## User Stories

### 1. C-Style Immutable Declarations
**As a** C developer  
**I want** to declare immutable variables using C syntax  
**So that** the code feels natural and familiar

**Acceptance Criteria:**
- 1.1: Parser accepts `int x = 42;` (implicit let)
- 1.2: Parser treats `int x = 42;` as immutable
- 1.3: Parser accepts `MyInt x = 32;` with typedef types
- 1.4: Generated Rust code is `let x: i32 = 42;`

### 2. C-Style Mutable Declarations
**As a** C developer  
**I want** to declare mutable variables using `var` prefix  
**So that** mutability is explicit

**Acceptance Criteria:**
- 2.1: Parser accepts `var int x = 42;`
- 2.2: Parser treats `var int x = 42;` as mutable
- 2.3: Generated Rust code is `let mut x: i32 = 42;`

### 3. Explicit Let with Type
**As a** Crusty developer  
**I want** to explicitly use `let` with a type  
**So that** I can be explicit about immutability

**Acceptance Criteria:**
- 3.1: Parser accepts `let int x = 42;`
- 3.2: `let int x = 42;` is equivalent to `int x = 42;`
- 3.3: Generated Rust code is `let x: i32 = 42;`

### 4. Type Inference with Let/Var
**As a** Rust developer  
**I want** to use type inference with `let`/`var`  
**So that** I can leverage Rust's type inference

**Acceptance Criteria:**
- 4.1: Parser accepts `let x = 42;` (inference)
- 4.2: Parser accepts `var x = 42;` (inference)
- 4.3: Type is inferred from initializer
- 4.4: Generated Rust code uses inference

### 5. Const Declarations
**As a** Crusty developer  
**I want** to declare constants with or without explicit types  
**So that** I have flexibility in constant declarations

**Acceptance Criteria:**
- 5.1: Parser accepts `const int MAX = 100;`
- 5.2: Parser accepts `const MAX = 100;` (inference)
- 5.3: Parser rejects `const MAX: int = 100;` (Rust-style)

### 6. Reject Rust-Style Annotations
**As a** Crusty developer  
**I want** Rust-style type annotations to be rejected  
**So that** syntax is consistent

**Acceptance Criteria:**
- 6.1: Parser rejects `let x: int = 42;`
- 6.2: Parser rejects `var x: int = 42;`
- 6.3: Parser rejects `const X: int = 42;`
- 6.4: Error message is clear

### 7. No Casting in Declarations
**As a** Crusty developer  
**I want** casting to not be used in declarations  
**So that** syntax is clean and unambiguous

**Acceptance Criteria:**
- 7.1: Documentation doesn't show `let x = (int)42;`
- 7.2: Examples use C-style or inference, not casting
- 7.3: Code generator doesn't emit casting in declarations

## Syntax Summary

### ✅ Supported Syntax

| Syntax | Mutability | Type | Example |
|--------|-----------|------|---------|
| `Type name = value;` | Immutable | Explicit | `int x = 42;` |
| `let Type name = value;` | Immutable | Explicit | `let int x = 42;` |
| `let name = value;` | Immutable | Inferred | `let x = 42;` |
| `var Type name = value;` | Mutable | Explicit | `var int x = 42;` |
| `var name = value;` | Mutable | Inferred | `var x = 42;` |
| `const Type NAME = value;` | Constant | Explicit | `const int MAX = 100;` |
| `const NAME = value;` | Constant | Inferred | `const MAX = 100;` |

### ❌ NOT Supported

| Syntax | Reason |
|--------|--------|
| `let x: int = 42;` | Rust-style colon annotation |
| `var x: int = 42;` | Rust-style colon annotation |
| `const X: int = 42;` | Rust-style colon annotation |
| `let x = (int)42;` | Casting in declaration (confusing) |

## Non-Functional Requirements

### Parser Complexity
- Parser must handle optional `let`/`var` keywords
- Parser must distinguish between type names and variable names
- Parser must handle both explicit types and inference

### Code Generation
- C-style declarations generate Rust `let` statements
- `var` prefix generates Rust `let mut` statements
- Type information preserved in generated code

### Documentation
- Primary examples use C-style syntax
- Type inference documented as alternative
- Clear explanation of when to use each syntax

## Out of Scope
- Function parameter declarations (already use C-style)
- Struct field declarations (already use C-style)
- Global variable declarations (future feature)

## Success Metrics
- Parser accepts all C-style declaration forms
- Parser rejects Rust-style colon annotations
- All tests pass
- Documentation shows C-style as primary
- Code generation produces correct Rust code

## Migration Guide

### Before (Current - Rust-style rejected)
```c
let x: int = 42;         // ❌ Rejected
let x = 42;              // ✅ Works (inference)
```

### After (C-style primary)
```c
int x = 42;              // ✅ Primary syntax (implicit let)
let int x = 42;          // ✅ Explicit let
let x = 42;              // ✅ Type inference
var int x = 42;          // ✅ Mutable
```

## Implementation Status

### Completed ✅
- [x] Parser rejects Rust-style colon annotations
- [x] Parser accepts `let name = value;` (inference)
- [x] Parser accepts `var name = value;` (inference)
- [x] Parser accepts `const NAME = value;` (inference)

### To Implement 🔨
- [ ] Parser accepts `Type name = value;` (implicit let)
- [ ] Parser accepts `let Type name = value;` (explicit let)
- [ ] Parser accepts `var Type name = value;` (explicit var)
- [ ] Parser accepts `const Type NAME = value;` (explicit type)
- [ ] Update all examples to use C-style syntax
- [ ] Update documentation to show C-style as primary
- [ ] Update code generator to handle all forms correctly
