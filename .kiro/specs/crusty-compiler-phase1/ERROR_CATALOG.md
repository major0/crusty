# Crusty Compiler Error Catalog

**Date**: 2026-01-30
**Status**: Complete catalog of all error messages in Phase 1
**Related**: [ERROR_HANDLING.md](ERROR_HANDLING.md) - Error architecture documentation

## Overview

This document catalogs all specific error messages that the Crusty compiler can produce, organized by compilation phase. Each error includes its message, context, and examples of when it occurs.

**See Also**: [ERROR_HANDLING.md](ERROR_HANDLING.md) for error architecture and types.

---

## Table of Contents

1. [Lexical Errors](#lexical-errors)
2. [Parse Errors](#parse-errors)
3. [Semantic Errors](#semantic-errors)
4. [Code Generation Errors](#code-generation-errors)
5. [I/O Errors](#io-errors)
6. [Rustc Invocation Errors](#rustc-invocation-errors)

---

## Lexical Errors

Errors that occur during tokenization of source code.

### LEX-001: Unterminated String Literal

**Message**: `"unterminated string literal"`

**When**: String literal is not closed before end of line or file.

**Example**:
```c
let s = "hello world
// Missing closing quote
```

**Error Output**:
```
Lexical error at 1:9-1:20: unterminated string literal
```

---

### LEX-002: Unterminated Block Comment

**Message**: `"unterminated block comment"`

**When**: Block comment `/* ... */` is not closed before end of file.

**Example**:
```c
/* This comment is never closed
int main() {
    return 0;
}
```

**Error Output**:
```
Lexical error at 1:1-3:1: unterminated block comment
```

---

### LEX-003: Invalid Escape Sequence

**Message**: `"invalid escape sequence"`

**When**: String contains an unrecognized escape sequence.

**Example**:
```c
let s = "hello\xworld";  // \x is not a valid escape
```

**Supported Escapes**: `\n`, `\t`, `\r`, `\\`, `\"`

**Error Output**:
```
Lexical error at 1:9-1:22: invalid escape sequence
```

---

### LEX-004: Unexpected Character

**Message**: `"unexpected character: '{char}'"`

**When**: Character is not valid in Crusty syntax.

**Example**:
```c
let x = 5 $ 3;  // $ is not a valid operator
```

**Error Output**:
```
Lexical error at 1:11-1:12: unexpected character: '$'
```

---

## Parse Errors

Errors that occur during syntax parsing.

### General Parse Errors

#### PARSE-001: Lexical Error During Parsing

**Message**: `"lexical error"`

**When**: Lexer error occurs while parser is requesting tokens.

**Context**: This wraps a lexical error as a parse error.

---

#### PARSE-002: Unexpected Token

**Message**: `"unexpected token"`

**When**: Token doesn't match expected grammar rule.

**Example**:
```c
let x = ;  // Missing expression
```

**Error Output**:
```
Parse error at 1:9-1:10: unexpected token (expected: expression) (found: ;)
```

---

### Item Declaration Errors

#### PARSE-010: Expected Item Declaration

**Message**: `"expected item declaration"`

**When**: Top-level token doesn't start a valid item (function, struct, enum, etc.).

**Example**:
```c
5 + 3;  // Not a valid top-level item
```

---

### Attribute Errors

#### PARSE-020: Expected Attribute Name

**Message**: `"expected attribute name"`

**When**: `#[` is followed by something other than an identifier.

**Example**:
```c
#[123]  // Attributes must be identifiers
struct Foo {}
```

---

#### PARSE-021: Expected Literal in Attribute

**Message**: `"expected literal in attribute"`

**When**: Attribute value is not a valid literal.

**Example**:
```c
#[derive(x + y)]  // Expressions not allowed
struct Foo {}
```

---

### Struct Errors

#### PARSE-030: Expected Struct Name

**Message**: `"expected struct name"`

**When**: `struct` keyword not followed by identifier.

**Example**:
```c
struct 123 {}  // Struct name must be identifier
```

---

#### PARSE-031: Expected Field Name

**Message**: `"expected field name"`

**When**: Struct field doesn't have a valid identifier.

**Example**:
```c
struct Point {
    123: i32,  // Field name must be identifier
}
```

---

### Enum Errors

#### PARSE-040: Expected Enum Name

**Message**: `"expected enum name"`

**When**: `enum` keyword not followed by identifier.

**Example**:
```c
enum 123 {}  // Enum name must be identifier
```

---

#### PARSE-041: Expected Enum Variant Name

**Message**: `"expected enum variant name"`

**When**: Enum variant doesn't have a valid identifier.

**Example**:
```c
enum Color {
    123,  // Variant name must be identifier
}
```

---

#### PARSE-042: Invalid Integer Literal (Enum)

**Message**: `"invalid integer literal"`

**When**: Enum discriminant value cannot be parsed as integer.

**Example**:
```c
enum Status {
    Active = 999999999999999999999,  // Too large
}
```

---

#### PARSE-043: Expected Integer Literal (Enum)

**Message**: `"expected integer literal"`

**When**: Enum discriminant is not an integer.

**Example**:
```c
enum Status {
    Active = "hello",  // Must be integer
}
```

---

### Function Errors

#### PARSE-050: Expected Function Name

**Message**: `"expected function name"`

**When**: Function declaration doesn't have a valid identifier.

**Example**:
```c
fn 123() {}  // Function name must be identifier
```

---

#### PARSE-051: Expected Parameter Name

**Message**: `"expected parameter name"`

**When**: Function parameter doesn't have a valid identifier.

**Example**:
```c
fn foo(123: i32) {}  // Parameter name must be identifier
```

---

#### PARSE-052: Expected 'self' After &

**Message**: `"expected 'self' after &"`

**When**: Method parameter has `&` but not followed by `self`.

**Example**:
```c
impl Foo {
    fn method(&x: i32) {}  // Should be &self
}
```

---

### Typedef Errors

#### PARSE-060: Expected Typedef Name

**Message**: `"expected typedef name"`

**When**: `typedef` not followed by valid identifier.

**Example**:
```c
typedef 123 = i32;  // Typedef name must be identifier
```

---

### Macro Errors

#### PARSE-070: Invalid Macro Name Format

**Message**: `"macro name must start and end with double underscores (e.g., __macro__)"`

**When**: `#define` macro doesn't follow `__name__` convention.

**Example**:
```c
#define MACRO() {}  // Should be __MACRO__()
```

---

#### PARSE-071: Expected Macro Name

**Message**: `"expected macro name"`

**When**: `#define` not followed by valid identifier.

**Example**:
```c
#define 123() {}  // Macro name must be identifier
```

---

#### PARSE-072: Expected Macro Delimiter

**Message**: `"expected (, [, or { after macro name"`

**When**: Macro invocation doesn't use valid delimiter.

**Example**:
```c
__println__<"hello">;  // Should use (), [], or {}
```

---

#### PARSE-073: Macro Delimiter Mismatch

**Message**: `"macro __{name}__ expects {expected} delimiters, but {used} was used"`

**When**: Macro invoked with wrong delimiter type.

**Example**:
```c
#define __foo__() { /* ... */ }
__foo__[];  // Should use ()
```

---

### Variable Declaration Errors

#### PARSE-080: Expected Variable Name

**Message**: `"expected variable name"`

**When**: Variable declaration doesn't have valid identifier.

**Example**:
```c
let 123 = 5;  // Variable name must be identifier
```

---

### Constant Declaration Errors

#### PARSE-090: Expected Constant Name

**Message**: `"expected constant name"`

**When**: Constant declaration doesn't have valid identifier.

**Example**:
```c
const 123: i32 = 5;  // Constant name must be identifier
```

---

### Label Errors

#### PARSE-100: Expected Label Name

**Message**: `"expected label name"`

**When**: Label doesn't have valid identifier.

**Example**:
```c
'123: loop {}  // Label must be identifier
```

---

#### PARSE-101: Expected 'while' or 'loop' After Label

**Message**: `"expected 'while' or 'loop' after label"`

**When**: Label not followed by loop construct.

**Example**:
```c
'outer: if true {}  // Labels only for loops
```

---

### Expression Errors

#### PARSE-110: Invalid Integer Literal

**Message**: `"invalid integer literal"`

**When**: Integer literal cannot be parsed.

**Example**:
```c
let x = 999999999999999999999;  // Too large
```

---

#### PARSE-111: Invalid Float Literal

**Message**: `"invalid float literal"`

**When**: Float literal cannot be parsed.

**Example**:
```c
let x = 1.2.3.4;  // Invalid format
```

---

#### PARSE-112: Invalid Tuple Index

**Message**: `"invalid tuple index"`

**When**: Tuple index is not a valid integer.

**Example**:
```c
let x = tuple.999999999999999999999;  // Too large
```

---

#### PARSE-113: Expected Field Name or Tuple Index

**Message**: `"expected field name or tuple index"`

**When**: Dot operator not followed by field or index.

**Example**:
```c
let x = point.;  // Missing field name
```

---

#### PARSE-114: Expected Field Name After '.'

**Message**: `"expected field name after '.'"`

**When**: Dot operator in expression not followed by identifier.

**Example**:
```c
let x = obj.123;  // Field name must be identifier
```

---

#### PARSE-115: Expected Method Name

**Message**: `"expected method name"`

**When**: Method call doesn't have valid identifier.

**Example**:
```c
impl Foo {
    fn 123() {}  // Method name must be identifier
}
```

---

## Semantic Errors

Errors that occur during semantic analysis when code is syntactically correct but semantically invalid.

### Function Errors

#### SEM-001: Function Name Reserved for Macros

**Message**: `"function name '{name}' uses double-underscore pattern reserved for macros"`

**Kind**: `UnsupportedFeature`

**When**: Function name starts and ends with `__`.

**Example**:
```c
fn __helper__() {}  // Reserved for macros
```

**Error Output**:
```
Semantic error at 0:0-0:0 (unsupported feature): function name '__helper__' uses double-underscore pattern reserved for macros
```

---

#### SEM-002: Duplicate Function Definition

**Message**: `"duplicate definition of function '{name}'"`

**Kind**: `DuplicateDefinition`

**When**: Function with same name defined multiple times.

**Example**:
```c
fn foo() {}
fn foo() {}  // Duplicate
```

---

#### SEM-003: Duplicate Parameter Name

**Message**: `"duplicate parameter '{name}' in function"`

**Kind**: `DuplicateDefinition`

**When**: Function has multiple parameters with same name.

**Example**:
```c
fn foo(x: i32, x: i32) {}  // Duplicate parameter
```

---

### Struct Errors

#### SEM-010: Duplicate Struct Definition

**Message**: `"duplicate definition of struct '{name}'"`

**Kind**: `DuplicateDefinition`

**When**: Struct with same name defined multiple times.

**Example**:
```c
struct Point { x: i32 }
struct Point { y: i32 }  // Duplicate
```

---

### Enum Errors

#### SEM-020: Duplicate Enum Definition

**Message**: `"duplicate definition of enum '{name}'"`

**Kind**: `DuplicateDefinition`

**When**: Enum with same name defined multiple times.

**Example**:
```c
enum Color { Red }
enum Color { Blue }  // Duplicate
```

---

### Typedef Errors

#### SEM-030: Duplicate Typedef Definition

**Message**: `"duplicate definition of typedef '{name}'"`

**Kind**: `DuplicateDefinition`

**When**: Typedef with same name defined multiple times.

**Example**:
```c
typedef MyInt = i32;
typedef MyInt = i64;  // Duplicate
```

---

### Constant Errors

#### SEM-040: Constant Type Mismatch

**Message**: `"constant '{name}' type mismatch: declared as {declared}, initialized with {actual}"`

**Kind**: `TypeMismatch`

**When**: Constant initializer type doesn't match declared type.

**Example**:
```c
const X: i32 = "hello";  // Type mismatch
```

---

#### SEM-041: Duplicate Constant Definition

**Message**: `"duplicate definition of constant '{name}'"`

**Kind**: `DuplicateDefinition`

**When**: Constant with same name defined multiple times.

**Example**:
```c
const X: i32 = 5;
const X: i32 = 10;  // Duplicate
```

---

### Static Variable Errors

#### SEM-050: Static Type Mismatch

**Message**: `"static '{name}' type mismatch: declared as {declared}, initialized with {actual}"`

**Kind**: `TypeMismatch`

**When**: Static initializer type doesn't match declared type.

**Example**:
```c
static X: i32 = "hello";  // Type mismatch
```

---

#### SEM-051: Duplicate Static Definition

**Message**: `"duplicate definition of static '{name}'"`

**Kind**: `DuplicateDefinition`

**When**: Static with same name defined multiple times.

**Example**:
```c
static X: i32 = 5;
static X: i32 = 10;  // Duplicate
```

---

### Macro Errors

#### SEM-060: Invalid Macro Name Format

**Message**: `"macro name '{name}' must start and end with double underscores"`

**Kind**: `UnsupportedFeature`

**When**: Macro definition doesn't follow `__name__` convention.

**Example**:
```c
#define MACRO() {}  // Should be __MACRO__()
```

---

### Variable Errors

#### SEM-070: Variable Type Mismatch (Let)

**Message**: `"variable '{name}' type mismatch: declared as {declared}, initialized with {actual}"`

**Kind**: `TypeMismatch`

**When**: Let variable initializer type doesn't match declared type.

**Example**:
```c
let x: i32 = "hello";  // Type mismatch
```

---

#### SEM-071: Duplicate Variable Definition (Let)

**Message**: `"duplicate definition of variable '{name}'"`

**Kind**: `DuplicateDefinition`

**When**: Let variable with same name defined in same scope.

**Example**:
```c
let x = 5;
let x = 10;  // Duplicate in same scope
```

---

#### SEM-072: Variable Type Mismatch (Var)

**Message**: `"variable '{name}' type mismatch: declared as {declared}, initialized with {actual}"`

**Kind**: `TypeMismatch`

**When**: Var variable initializer type doesn't match declared type.

**Example**:
```c
var x: i32 = "hello";  // Type mismatch
```

---

#### SEM-073: Duplicate Variable Definition (Var)

**Message**: `"duplicate definition of variable '{name}'"`

**Kind**: `DuplicateDefinition`

**When**: Var variable with same name defined in same scope.

**Example**:
```c
var x = 5;
var x = 10;  // Duplicate in same scope
```

---

#### SEM-074: Constant Type Mismatch (Local)

**Message**: `"constant '{name}' type mismatch: declared as {declared}, initialized with {actual}"`

**Kind**: `TypeMismatch`

**When**: Local constant initializer type doesn't match declared type.

**Example**:
```c
fn foo() {
    const X: i32 = "hello";  // Type mismatch
}
```

---

#### SEM-075: Duplicate Constant Definition (Local)

**Message**: `"duplicate definition of constant '{name}'"`

**Kind**: `DuplicateDefinition`

**When**: Local constant with same name defined in same scope.

**Example**:
```c
fn foo() {
    const X: i32 = 5;
    const X: i32 = 10;  // Duplicate
}
```

---

### Control Flow Errors

#### SEM-080: If Condition Type Mismatch

**Message**: `"if condition must be bool, found {actual}"`

**Kind**: `TypeMismatch`

**When**: If condition is not boolean type.

**Example**:
```c
if 5 {  // Condition must be bool
    // ...
}
```

---

#### SEM-081: While Condition Type Mismatch

**Message**: `"while condition must be bool, found {actual}"`

**Kind**: `TypeMismatch`

**When**: While condition is not boolean type.

**Example**:
```c
while 5 {  // Condition must be bool
    // ...
}
```

---

#### SEM-082: Loop Condition Type Mismatch

**Message**: `"loop condition must be bool, found {actual}"`

**Kind**: `TypeMismatch`

**When**: Loop condition is not boolean type.

**Example**:
```c
loop 5 {  // Condition must be bool (if present)
    // ...
}
```

---

#### SEM-083: For Loop Variable Duplicate

**Message**: `"duplicate definition of loop variable '{name}'"`

**Kind**: `DuplicateDefinition`

**When**: For loop variable shadows existing variable.

**Example**:
```c
let x = 5;
for x in 0..10 {  // Shadows outer x
    // ...
}
```

---

#### SEM-084: Switch Case Type Mismatch

**Message**: `"switch case value type mismatch: expected {expected}, found {actual}"`

**Kind**: `TypeMismatch`

**When**: Switch case value type doesn't match switch expression type.

**Example**:
```c
switch x {
    case 5: break;
    case "hello": break;  // Type mismatch
}
```

---

### Expression Errors

#### SEM-090: Undefined Variable

**Message**: `"undefined variable '{name}'"`

**Kind**: `UndefinedVariable`

**When**: Variable used before declaration.

**Example**:
```c
let x = y + 5;  // y is undefined
```

---

#### SEM-091: Assignment Type Mismatch

**Message**: `"assignment type mismatch: cannot assign {right} to {left}"`

**Kind**: `TypeMismatch`

**When**: Assignment right side type incompatible with left side.

**Example**:
```c
let x: i32 = 5;
x = "hello";  // Type mismatch
```

---

#### SEM-092: Dereference Non-Pointer

**Message**: `"cannot dereference non-pointer type {actual}"`

**Kind**: `InvalidOperation`

**When**: Dereference operator used on non-pointer type.

**Example**:
```c
let x = 5;
let y = *x;  // x is not a pointer
```

---

#### SEM-093: Function Call Argument Count Mismatch

**Message**: `"function '{name}' expects {expected} arguments, found {actual}"`

**Kind**: `TypeMismatch`

**When**: Function called with wrong number of arguments.

**Example**:
```c
fn foo(x: i32, y: i32) {}
foo(5);  // Missing argument
```

---

#### SEM-094: Function Call Argument Type Mismatch

**Message**: `"argument {index} type mismatch: expected {expected}, found {actual}"`

**Kind**: `TypeMismatch`

**When**: Function argument type doesn't match parameter type.

**Example**:
```c
fn foo(x: i32) {}
foo("hello");  // Type mismatch
```

---

#### SEM-095: Call Non-Function

**Message**: `"cannot call non-function type {actual}"`

**Kind**: `InvalidOperation`

**When**: Call expression used on non-function type.

**Example**:
```c
let x = 5;
x();  // x is not a function
```

---

#### SEM-096: Field Access on Non-Struct

**Message**: `"cannot access field on non-struct type {actual}"`

**Kind**: `InvalidOperation`

**When**: Field access used on non-struct type.

**Example**:
```c
let x = 5;
let y = x.field;  // x is not a struct
```

---

#### SEM-097: Undefined Field

**Message**: `"struct {struct_name} has no field '{field}'"`

**Kind**: `UndefinedVariable`

**When**: Field doesn't exist on struct.

**Example**:
```c
struct Point { x: i32 }
let p = Point { x: 5 };
let y = p.y;  // No field 'y'
```

---

#### SEM-098: Method Call on Non-Struct

**Message**: `"cannot call method on non-struct type {actual}"`

**Kind**: `InvalidOperation`

**When**: Method call used on non-struct type.

**Example**:
```c
let x = 5;
x.method();  // x is not a struct
```

---

#### SEM-099: Index Non-Array

**Message**: `"cannot index non-array/slice type {actual}"`

**Kind**: `InvalidOperation`

**When**: Index operator used on non-indexable type.

**Example**:
```c
let x = 5;
let y = x[0];  // x is not an array
```

---

#### SEM-100: Index Type Mismatch

**Message**: `"array index must be integer or usize, found {actual}"`

**Kind**: `TypeMismatch`

**When**: Array index is not integer type.

**Example**:
```c
let arr = [1, 2, 3];
let x = arr["hello"];  // Index must be integer
```

---

#### SEM-101: Binary Operation Type Mismatch

**Message**: `"binary operation type mismatch: {left} {op} {right}"`

**Kind**: `TypeMismatch`

**When**: Binary operation operands have incompatible types.

**Example**:
```c
let x = 5 + "hello";  // Type mismatch
```

---

#### SEM-102: Ternary Condition Type Mismatch

**Message**: `"ternary condition must be bool, found {actual}"`

**Kind**: `TypeMismatch`

**When**: Ternary operator condition is not boolean.

**Example**:
```c
let x = 5 ? 10 : 20;  // Condition must be bool
```

---

#### SEM-103: Ternary Branch Type Mismatch

**Message**: `"ternary branches have incompatible types: {then_type} and {else_type}"`

**Kind**: `TypeMismatch`

**When**: Ternary operator branches have different types.

**Example**:
```c
let x = true ? 5 : "hello";  // Branches must match
```

---

## Code Generation Errors

Errors that occur during Rust code generation.

### CODEGEN-001: Unsupported AST Node

**Message**: `"cannot generate code for unsupported AST node"`

**When**: Code generator encounters AST node it doesn't support.

**Context**: This is typically an internal error indicating incomplete implementation.

---

### CODEGEN-002: Invalid Type

**Message**: `"cannot generate code for invalid type"`

**When**: Type cannot be translated to Rust.

**Context**: This may occur with complex or unsupported type constructs.

---

## I/O Errors

Standard Rust I/O errors for file operations.

### IO-001: File Not Found

**Message**: `"No such file or directory"`

**When**: Source file doesn't exist.

**Example**:
```bash
crustyc nonexistent.crst
```

**Error Output**:
```
I/O error: No such file or directory (os error 2)
```

---

### IO-002: Permission Denied

**Message**: `"Permission denied"`

**When**: No permission to read source file or write output.

**Example**:
```bash
crustyc /root/protected.crst
```

**Error Output**:
```
I/O error: Permission denied (os error 13)
```

---

### IO-003: Disk Full

**Message**: `"No space left on device"`

**When**: Cannot write output file due to full disk.

**Error Output**:
```
I/O error: No space left on device (os error 28)
```

---

## Rustc Invocation Errors

Errors from invoking the Rust compiler on generated code.

### RUSTC-001: Rustc Not Found

**Message**: `"rustc not found in PATH"`

**When**: Rust compiler is not installed or not in PATH.

**Example**:
```bash
crustyc --compile example.crst
```

**Error Output**:
```
rustc invocation error: rustc not found in PATH
```

---

### RUSTC-002: Rustc Compilation Failed

**Message**: `"rustc failed with exit code {code}: {stderr}"`

**When**: Generated Rust code fails to compile.

**Context**: This typically indicates a bug in the code generator.

**Example**:
```
rustc invocation error: rustc failed with exit code 1: error[E0425]: cannot find value `x` in this scope
```

---

## Error Message Guidelines

When adding new error messages, follow these guidelines:

### 1. Be Specific
❌ Bad: `"syntax error"`  
✅ Good: `"expected ';' after statement"`

### 2. Provide Context
❌ Bad: `"type error"`  
✅ Good: `"expected i32, found String"`

### 3. Suggest Solutions
❌ Bad: `"undefined variable"`  
✅ Good: `"variable 'count' not found in this scope. Did you mean 'counter'?"`

### 4. Use Consistent Terminology
- Use "expected" and "found" for parse errors
- Use "not found in this scope" for undefined variables
- Use "cannot" for impossible operations
- Use "must be" for type requirements

### 5. Reference Crusty Concepts
❌ Bad: `"use statement not supported"`  
✅ Good: `"#include directives are not supported. Use #import instead"`

---

## Error Statistics

**Total Error Types**: 100+

**By Phase**:
- Lexical: 4 error types
- Parse: 30+ error types
- Semantic: 60+ error types
- Code Generation: 2+ error types
- I/O: 3+ error types
- Rustc: 2+ error types

**Most Common Errors** (estimated):
1. Undefined variable (SEM-090)
2. Type mismatch (various)
3. Unexpected token (PARSE-002)
4. Duplicate definition (various)
5. Invalid operation (various)

---

## Future Enhancements

### Planned Improvements

1. **Error Codes** (Future)
   - Assign unique codes to each error (e.g., E0001, E0002)
   - Enable `--explain` flag for detailed explanations
   - Create error code documentation website

2. **Suggestions** (Future)
   - "Did you mean...?" for typos
   - Suggest fixes for common mistakes
   - Show similar identifiers for undefined variables

3. **Error Recovery** (Task 28.3)
   - Continue parsing after errors
   - Report multiple parse errors in one pass
   - Synchronization points for error recovery

4. **codespan-reporting Integration** (Task 28.1)
   - Beautiful error displays with source code snippets
   - Color-coded error messages
   - Caret indicators pointing to exact error locations

5. **Structured Error Data** (Future)
   - Machine-readable error format (JSON)
   - IDE integration support
   - Language server protocol (LSP) compatibility

---

## See Also

- [ERROR_HANDLING.md](ERROR_HANDLING.md) - Error handling architecture
- [requirements.md](requirements.md) - Requirement 11: Report Compilation Errors
- [tasks.md](tasks.md) - Task 28: Error message improvements
- [src/error.rs](../../src/error.rs) - Error module implementation
- [src/lexer.rs](../../src/lexer.rs) - Lexer error generation
- [src/parser.rs](../../src/parser.rs) - Parser error generation
- [src/semantic.rs](../../src/semantic.rs) - Semantic error generation

---

*This catalog documents all error messages as of Phase 1. Error messages will be expanded and improved as the compiler matures.*
