# C Compatibility

## Introduction

Crusty supports a subset of C syntax that maps cleanly to Rust. Some C features are intentionally excluded.

## Supported Features

### Syntax
- C-style function declarations (return type before name)
- C-style variable declarations (type before name)
- C-style control flow (if, while, for with parentheses)
- C-style operators (arithmetic, logical, bitwise)
- C-style struct definitions
- C-style enum definitions
- C-style type casting
- Preprocessor-style macros (#define)

### Types
- Primitive types (int, float, char, bool)
- Sized integers (i32, i64, u32, u64)
- Pointers and references
- Arrays
- Structs and enums

## Excluded Features

### Memory Management
- `malloc`/`free` - Use Rust's ownership system
- Manual pointer arithmetic - Use safe abstractions
- Uninitialized variables - All variables must be initialized

### Unsafe Constructs
- `goto` - Use structured control flow
- `union` - Use Rust enums with variants
- Null pointers - Use Option type
- Undefined behavior - All behavior is defined

### Preprocessor
- `#include` - Use module system
- `#ifdef`/`#ifndef` - Use Rust's cfg attributes
- Complex macro conditionals

## Rationale

Excluding these features ensures that all Crusty programs are memory-safe and have defined behavior. The excluded features either:
1. Cannot be safely expressed in Rust
2. Have better alternatives in Rust's type system
3. Would require unsafe code generation
