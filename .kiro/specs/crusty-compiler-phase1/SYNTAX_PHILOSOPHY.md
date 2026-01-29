# Crusty Syntax Transpilation Philosophy

## Core Principle

**Crusty is a SYNTACTIC transpilation layer, not a SEMANTIC one.**

Method names, function names, and identifiers pass through unchanged between Crusty and Rust. This preserves:
- Bidirectional transpilation (Crusty ↔ Rust)
- No conflicts with user-defined functions
- Transparent mapping between languages

## Syntax Transformations (Allowed)

These are pure syntax changes that don't affect semantics:

### 1. Type Syntax
- `Type?` → `Result<Type, Box<dyn std::error::Error>>`
- `int` → `i32`
- `float` → `f64`
- `void` → `()` (no return annotation)

### 2. Operator Syntax
- `expr?` → `expr?` (error propagation - pass through)
- `@Type.method()` → `Type::method()` (type-scoped calls)
- `.label:` → `'label:` (loop labels)

### 3. NULL Special Case (ONLY Exception)
- `NULL` → `Option::None` (special keyword transformation)
- `ptr == NULL` → `ptr.is_none()` (NULL comparison transformation)
- `ptr != NULL` → `ptr.is_some()` (NULL comparison transformation)

**Rationale:** NULL is a special C keyword that has no direct Rust equivalent. This is the ONLY semantic transformation allowed.

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
