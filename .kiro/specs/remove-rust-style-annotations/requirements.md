# Requirements: Remove Rust-Style Type Annotations

## Overview
Remove Rust-style type annotations (`let x: Type = value`) from Crusty to maintain C-like philosophy. Only support C-style casting and type inference.

## Background
Crusty currently supports Rust-style type annotations which is inconsistent with its C-like design philosophy. The language should use:
- **Type inference**: `let x = 42;` (infer type from initializer)
- **C-style casting**: `let x = (MyInt)42;` (explicit type via cast)

## User Stories

### 1. Remove Type Annotations from Let Statements
**As a** Crusty developer  
**I want** `let` statements to not support type annotations  
**So that** the language is more C-like

**Acceptance Criteria:**
- 1.1: Parser rejects `let x: Type = value` syntax
- 1.2: Parser accepts `let x = value` (type inference)
- 1.3: Parser accepts `let x = (Type)value` (C-style cast)
- 1.4: Error message is clear when type annotation is attempted

### 2. Remove Type Annotations from Var Statements
**As a** Crusty developer  
**I want** `var` statements to not support type annotations  
**So that** mutable variables use C-style syntax

**Acceptance Criteria:**
- 2.1: Parser rejects `var x: Type = value` syntax
- 2.2: Parser accepts `var x = value` (type inference)
- 2.3: Parser accepts `var x = (Type)value` (C-style cast)

### 3. Update All Examples
**As a** Crusty user  
**I want** all examples to use C-style syntax  
**So that** I learn the correct idioms

**Acceptance Criteria:**
- 3.1: All `.crst` files in `example/` directory updated
- 3.2: No Rust-style type annotations remain
- 3.3: Examples compile successfully

### 4. Update All Tests
**As a** Crusty maintainer  
**I want** all tests to use C-style syntax  
**So that** tests validate correct behavior

**Acceptance Criteria:**
- 4.1: All test files updated to use C-style or type inference
- 4.2: All tests pass
- 4.3: No Rust-style type annotations in test code

### 5. Update Documentation
**As a** Crusty user  
**I want** documentation to reflect C-style only  
**So that** I understand the correct syntax

**Acceptance Criteria:**
- 5.1: SYNTAX_REFERENCE.md shows only C-style and type inference
- 5.2: README.md updated if needed
- 5.3: All spec documents updated

## Non-Functional Requirements

### Breaking Change Management
- This is a **major breaking change**
- All existing Crusty code using type annotations will break
- Clear migration guide needed

### Type Inference
- Type inference must work correctly for common cases
- Semantic analyzer must infer types from initializers
- Clear error messages when type cannot be inferred

### Const Statements
- Decision needed: Should `const` keep type annotations?
- Constants may need explicit types for clarity
- Consider: `const X = (int)42;` vs `const X: int = 42;`

## Out of Scope
- Function parameter type annotations (these are required, not optional)
- Return type annotations (these are required, not optional)
- Struct field type annotations (these are required, not optional)

## Success Metrics
- Parser rejects all Rust-style type annotations
- All examples compile with C-style syntax
- All tests pass
- Documentation is consistent
- Type inference works for common cases

## Migration Guide

### Before (Rust-style)
```c
let x: int = 42;
let y: MyInt = 10;
var z: float = 3.14;
```

### After (C-style)
```c
let x = 42;              // Type inference
let y = (MyInt)10;       // C-style cast
var z = 3.14;            // Type inference
```

## Implementation Status

### Completed ✅
- [x] Parser updated to reject type annotations in `let` statements
- [x] Parser updated to reject type annotations in `var` statements
- [x] C-style casting implemented and working
- [x] Updated all test files (412 tests passing)
- [x] Updated all example files (typedef_demo.crst, main.crst)
- [x] Updated code generator to emit C-style syntax
- [x] Verified type inference works correctly
- [x] Migration guide included in requirements
- [x] Error messages clear ("expected Semicolon, found Colon")
- [x] SYNTAX_REFERENCE.md already uses C-style syntax
- [x] Property test generator fixed to exclude type names

### Decision: Const Statements ✅
- `const` statements **keep** type annotations (required for clarity)
- Syntax: `const X: int = 42;` (unchanged)
- Rationale: Constants need explicit types for documentation and clarity
