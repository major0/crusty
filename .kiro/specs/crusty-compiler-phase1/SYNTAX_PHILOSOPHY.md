# Crusty Syntax Transpilation Philosophy

## Core Principle

**Crusty is primarily a SYNTAX layer over Rust, with selective semantic enhancements.**

As a general rule, Crusty provides syntax changes over Rust. However, a few C-like semantic constructs are brought over for familiarity, where they can map cleanly to Rust's semantics. Crusty is **C-like**, not C itself.

### What Crusty Is

- A **syntax bridge** for developers comfortable with C-style syntax
- A **learning tool** to help understand Rust through familiar syntax
- A **1:1 compatible** layer that maintains full Rust semantics underneath
- A way to write Rust code with C-like appearance

### What Crusty Is NOT

- **Not a C compiler** - Crusty does not compile native C code
- **Not a C compatibility layer** - No standard C library functions
- **Not a semantic transformation** - Rust semantics are preserved
- **Not a replacement for Rust** - It's an alternative syntax for Rust

Method names, function names, and identifiers pass through unchanged between Crusty and Rust. This preserves:
- Bidirectional transpilation (Crusty ↔ Rust)
- No conflicts with user-defined functions
- Transparent mapping between languages
- Full Rust ecosystem compatibility

## C-Like Semantics Brought Over

Several C-like semantic constructs are supported to provide familiarity. These are semantic transformations because they introduce control flow, scoping, or compile-time behavior that requires scaffolding beyond simple syntax mapping:

### 1. NULL (Semantic Transformation)
- `NULL` → `Option::None` (C keyword with no Rust equivalent)
- `ptr == NULL` → `ptr.is_none()` (NULL comparison)
- `ptr != NULL` → `ptr.is_some()` (NULL comparison)

**Why Semantic:** NULL is a C keyword that maps to a completely different Rust concept (Option type). This requires understanding the semantic intent of null pointer checks.

### 2. C-Style For Loops (Semantic Transformation)
- `for(int i = 0; i < 100; i++) { ... }` → Rust loop with variable scoping
- `for(int i = 0, j = 2; i < 100; i++) { ... }` → Multiple variable initialization
- `for(;;) { ... }` → `loop { ... }` (infinite loop as special case)

**Why Semantic:** C-style for loops introduce variable scoping (variables declared in init are scoped to the loop) and require scaffolding to translate the three-part structure (init; condition; update) into Rust's loop constructs.

### 3. Switch Statements (Semantic Transformation)
- `switch(expr) { case val: ... }` → `match expr { val => ... }`
- C-style switch/case with fall-through → Rust match with exhaustive patterns

**Why Semantic:** Switch statements have different semantics than Rust's match (fall-through vs. exhaustive matching). The transformation must handle break statements, default cases, and pattern matching differently.

### 4. Macro Definitions (Semantic Transformation)
- `#define __MACRO__() body` → `macro_rules! macro { ... }`
- C-style preprocessor macros → Rust declarative macros

**Why Semantic:** C preprocessor macros are compile-time text substitution, while Rust macros are token-based with hygiene. The transformation requires understanding macro expansion semantics and translating between fundamentally different macro systems.

## Pure Syntax Transformations

These are syntax-only changes that maintain Rust semantics:

### 1. Type Syntax
- `Type?` → `Result<Type, Box<dyn std::error::Error>>`
- `int` → `i32`, `float` → `f64`
- `void` → `()` (no return annotation)

### 2. C-Inspired Syntax (Crusty Innovations)
- `@Type.method()` → `Type::method()` (type-scoped calls with @ prefix)
- `.label:` → `'label:` (loop labels with dot prefix)
- `__macro__()` → `macro!()` (double-underscore for macros)

**Note:** The `@` prefix and double-underscore syntax are Crusty innovations in the C spirit, maintaining 1:1 Rust compatibility.

### 3. Operator Syntax (Pass Through)
- `expr?` → `expr?` (error propagation - unchanged)
- Method calls pass through unchanged
- Function calls pass through unchanged

## What is NOT Transformed

### Method Names (Pass Through Unchanged)
- `.is_err()` → `.is_err()` (NOT `.is_error()`)
- `.is_ok()` → `.is_ok()`
- `.unwrap()` → `.unwrap()`
- `.unwrap_or()` → `.unwrap_or()`
- Any other method name passes through unchanged

### Function Names (Pass Through Unchanged)
- `Ok()` → `Ok()` (NOT `ok()`)
- `Err()` → `Err()` (NOT `error()`)
- Any user-defined function passes through unchanged

### Why This Matters

1. **Bidirectional Transpilation**: If we rename methods, reverse transpilation becomes ambiguous
2. **User Functions**: Users might define their own `error()` function - we shouldn't hijack it
3. **Transparency**: Developers should know exactly what Rust code will be generated
4. **Simplicity**: Less magic = fewer surprises

## Updated Requirements

### Error Handling (Requirement 49)

**Syntax Transformations Only:**
- `Type?` → `Result<Type, Box<dyn std::error::Error>>`
- `expr?` → `expr?` (pass through to Rust)

**Users Must Use Rust API Directly:**
- `.is_err()`, `.is_ok()`, `.unwrap()`, `.unwrap_or()`
- `Ok()`, `Err()` constructors

**Removed Semantic Transformations:**
- ~~`error(value)` → `Err(value)`~~ (REMOVED)
- ~~`.is_error()` → `.is_err()`~~ (REMOVED)

### NULL Handling (Requirement 36)

**Special Case Transformations (ONLY Exception):**
- `NULL` → `Option::None`
- `ptr == NULL` → `ptr.is_none()`
- `ptr != NULL` → `ptr.is_some()`

This is the ONLY place where we do semantic transformation, because NULL is a C keyword with no Rust equivalent.

## Implementation Status

✅ **Correctly Implemented:**
- Type? → Result<Type, E>
- expr? → expr? (pass through)
- NULL → Option::None
- Method names pass through unchanged
- Function names pass through unchanged

❌ **Incorrectly Specified in Requirements:**
- Requirements 49.2, 49.4-7, 49.9, 49.11-13 specify semantic transformations
- These should be removed/updated

## Unsupported C Features

Crusty does NOT support the following C features, as they are incompatible with Rust's safety guarantees:

### unions
C unions allow multiple fields to share the same memory location, which violates Rust's type safety. Use Rust's enum types instead for tagged unions.

### goto statements
The `goto` statement enables arbitrary control flow that breaks Rust's structured control flow guarantees. Use loops, match expressions, and early returns instead.

### Unsafe pointer arithmetic
Direct pointer arithmetic (e.g., `ptr + offset`) without bounds checking is unsafe. Use Rust's slice indexing, iterators, or explicit unsafe blocks when necessary.

### #include directives
C's `#include` preprocessor directive is replaced by Rust's module system. Use `#use` directives to import modules.

### Preprocessor conditionals
C preprocessor directives like `#ifdef`, `#ifndef`, `#if` are not supported. Use Rust's `cfg` attributes for conditional compilation.

## Syntax Stability Warning

**IMPORTANT**: Crusty syntax is currently **unstable and evolving**. The language is in active development, and syntax may change between versions without notice.

- Breaking changes may occur in any release
- Code written for one version may not compile in future versions
- The syntax is experimental and subject to refinement
- Production use is not recommended at this time

This is an experimental project to explore C-like syntax over Rust. Use at your own risk.

## Action Items

1. Update requirements.md to remove semantic transformations from Requirement 49
2. Update design.md to clarify syntax-only philosophy
3. Update tasks.md task 16.7 description to remove method renaming
4. Keep implementation as-is (already correct)


## See Also

- [README.md](../../../README.md) - Project overview with syntax examples
- [requirements.md](requirements.md) - Detailed feature requirements
- [design.md](design.md) - Architecture and design decisions
- [tasks.md](tasks.md) - Implementation plan and progress
- [CONTRIBUTING.md](../../../CONTRIBUTING.md) - How to contribute to the project
