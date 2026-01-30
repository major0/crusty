# Crusty Syntax Transpilation Philosophy

## Core Principle

**Crusty is a C-like SYNTAX layer over Rust, not a semantic transformation.**

As a general rule, Crusty provides syntax changes over Rust, not semantic ones, though a few C-like semantics are brought over for familiarity. Crusty is **C-like**, not C itself.

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

A few C-like semantic constructs are supported to provide familiarity:

### 1. NULL (Special Semantic Exception)
- `NULL` → `Option::None` (C keyword with no Rust equivalent)
- `ptr == NULL` → `ptr.is_none()` (NULL comparison)
- `ptr != NULL` → `ptr.is_some()` (NULL comparison)

**Rationale:** NULL is a C keyword that has no direct Rust syntax equivalent. This is the ONLY semantic transformation.

### 2. C-Style For Loops
- `for(init; cond; update) { ... }` → Rust loop equivalent
- `for(;;) { ... }` → `loop { ... }` (infinite loop as special case)
- Traditional three-part C for-loop syntax

### 3. Switch Statements
- `switch(expr) { case val: ... }` → `match expr { val => ... }`
- C-style switch/case syntax maps to Rust match expressions

### 4. Macro Definitions
- `#define __MACRO__() body` → `macro_rules! macro { ... }`
- C-style preprocessor macros map to Rust declarative macros

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
