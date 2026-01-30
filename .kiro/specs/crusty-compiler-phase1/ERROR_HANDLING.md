# Crusty Compiler Error Handling Architecture

**Date**: 2026-01-30
**Module**: `src/error.rs`
**Status**: Core infrastructure complete, specific errors added as features are implemented

## Overview

The Crusty compiler uses a structured, hierarchical error handling system that provides clear, actionable error messages with precise source location information. The architecture follows Rust best practices and is designed to support future integration with tools like `codespan-reporting` for beautiful error displays.

## Design Principles

1. **Precise Location Tracking**: Every error includes exact source position (line and column)
2. **Hierarchical Structure**: Errors are organized by compilation phase
3. **Multiple Error Reporting**: Semantic analysis can report multiple errors in one pass
4. **Type Safety**: Strong typing prevents error category confusion
5. **Extensibility**: Easy to add new error types and kinds
6. **User-Friendly**: Clear messages with context about what was expected

## Error Type Hierarchy

```
CompilerError (top-level enum)
├── Lex(LexError)              - Lexical analysis errors
├── Parse(ParseError)          - Syntax parsing errors
├── Semantic(Vec<SemanticError>) - Semantic analysis errors (multiple)
├── CodeGen(CodeGenError)      - Code generation errors
├── Io(std::io::Error)         - File I/O errors
└── RustcInvocation(String)    - Rust compiler invocation errors
```

## Core Types

### Position

Represents a single point in source code.

```rust
pub struct Position {
    pub line: usize,
    pub column: usize,
}
```

**Usage**:
```rust
let pos = Position::new(10, 5);  // Line 10, column 5
println!("{}", pos);              // Output: "10:5"
```

**Properties**:
- 1-indexed (line 1, column 1 is the start of the file)
- Immutable after creation
- Implements `Display`, `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

---

### Span

Represents a range in source code from start to end position.

```rust
pub struct Span {
    pub start: Position,
    pub end: Position,
}
```

**Usage**:
```rust
let span = Span::new(
    Position::new(1, 1),
    Position::new(1, 10)
);
println!("{}", span);  // Output: "1:1-1:10"
```

**Properties**:
- Represents a contiguous range of source code
- Used by all error types for location tracking
- Implements `Display`, `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`

---

## Error Types by Phase

### 1. LexError (Lexical Analysis)

Errors that occur during tokenization of source code.

```rust
pub struct LexError {
    pub span: Span,
    pub message: String,
}
```

**When Used**: During lexical analysis when invalid characters or malformed tokens are encountered.

**Current Error Cases**:
- Unterminated string literals
- Unterminated block comments
- Invalid escape sequences
- Unexpected characters

**Example**:
```rust
let error = LexError::new(
    Span::new(Position::new(5, 10), Position::new(5, 11)),
    "unexpected character: '@'"
);
// Output: "Lexical error at 5:10-5:11: unexpected character: '@'"
```

**Display Format**:
```
Lexical error at {span}: {message}
```

---

### 2. ParseError (Syntax Analysis)

Errors that occur during parsing when syntax rules are violated.

```rust
pub struct ParseError {
    pub span: Span,
    pub message: String,
    pub expected: Vec<String>,  // What tokens were expected
    pub found: String,           // What token was actually found
}
```

**When Used**: During parsing when the token stream doesn't match Crusty's grammar.

**Features**:
- Lists expected tokens to help users fix syntax errors
- Shows what was actually found
- Precise location of the syntax error

**Example**:
```rust
let error = ParseError::new(
    Span::new(Position::new(10, 5), Position::new(10, 6)),
    "unexpected token",
    vec!["identifier".to_string(), ";".to_string()],
    "}"
);
// Output: "Parse error at 10:5-10:6: unexpected token (expected: identifier, ;) (found: })"
```

**Display Format**:
```
Parse error at {span}: {message} (expected: {expected_list}) (found: {found})
```

---

### 3. SemanticError (Semantic Analysis)

Errors that occur during semantic analysis when code is syntactically correct but semantically invalid.

```rust
pub struct SemanticError {
    pub span: Span,
    pub kind: SemanticErrorKind,
    pub message: String,
}
```

**Error Kinds**:
```rust
pub enum SemanticErrorKind {
    UndefinedVariable,      // Variable used before declaration
    TypeMismatch,           // Type incompatibility
    DuplicateDefinition,    // Symbol defined multiple times
    InvalidOperation,       // Operation not valid for types
    UnsupportedFeature,     // C feature not supported in Crusty
}
```

**When Used**: During semantic analysis after successful parsing.

**Features**:
- Categorized by error kind for better error handling
- Multiple errors can be collected and reported together
- Provides context about what went wrong semantically

**Example**:
```rust
let error = SemanticError::new(
    Span::new(Position::new(15, 10), Position::new(15, 15)),
    SemanticErrorKind::UndefinedVariable,
    "variable 'count' not found in this scope"
);
// Output: "Semantic error at 15:10-15:15 (undefined variable): variable 'count' not found in this scope"
```

**Display Format**:
```
Semantic error at {span} ({kind}): {message}
```

**Multiple Errors**:
Semantic errors are collected in a `Vec<SemanticError>` to report all issues in one pass:
```rust
CompilerError::Semantic(vec![error1, error2, error3])
```

---

### 4. CodeGenError (Code Generation)

Errors that occur during Rust code generation.

```rust
pub struct CodeGenError {
    pub message: String,
}
```

**When Used**: During code generation when the AST cannot be translated to valid Rust.

**Note**: Code generation errors typically don't include spans because they represent internal compiler issues rather than user code problems.

**Example**:
```rust
let error = CodeGenError::new("cannot generate code for unsupported AST node");
// Output: "Code generation error: cannot generate code for unsupported AST node"
```

**Display Format**:
```
Code generation error: {message}
```

---

### 5. Io(std::io::Error)

Standard Rust I/O errors for file operations.

**When Used**: 
- Reading source files
- Writing generated code
- File not found
- Permission denied
- Disk full

**Example**:
```rust
CompilerError::Io(std::io::Error::new(
    std::io::ErrorKind::NotFound,
    "source file not found"
))
// Output: "I/O error: source file not found"
```

---

### 6. RustcInvocation(String)

Errors from invoking the Rust compiler on generated code.

**When Used**: When `rustc` fails to compile the generated Rust code.

**Example**:
```rust
CompilerError::RustcInvocation(
    "rustc failed with exit code 1: error[E0425]: cannot find value `x`"
)
// Output: "rustc invocation error: rustc failed with exit code 1: error[E0425]: cannot find value `x`"
```

---

## CompilerError (Top-Level)

The main error type that encompasses all compiler errors.

```rust
pub enum CompilerError {
    Lex(LexError),
    Parse(ParseError),
    Semantic(Vec<SemanticError>),
    CodeGen(CodeGenError),
    Io(std::io::Error),
    RustcInvocation(String),
}
```

**Features**:
- Implements `std::error::Error` trait
- Provides `source()` method for error chaining
- Automatic conversion from specific error types via `From` trait
- Custom `Display` implementation for user-friendly output

**Type Alias**:
```rust
pub type Result<T> = std::result::Result<T, CompilerError>;
```

This allows concise error handling throughout the compiler:
```rust
fn compile(source: &str) -> Result<String> {
    let tokens = lex(source)?;
    let ast = parse(tokens)?;
    let validated_ast = analyze(ast)?;
    let rust_code = generate(validated_ast)?;
    Ok(rust_code)
}
```

---

## Error Conversion

The error system uses Rust's `From` trait for automatic error conversion:

```rust
impl From<LexError> for CompilerError { ... }
impl From<ParseError> for CompilerError { ... }
impl From<Vec<SemanticError>> for CompilerError { ... }
impl From<CodeGenError> for CompilerError { ... }
impl From<std::io::Error> for CompilerError { ... }
```

**Usage**:
```rust
// Automatic conversion with ? operator
fn lex_source(source: &str) -> Result<Vec<Token>> {
    let lexer = Lexer::new(source);
    lexer.tokenize()?  // LexError automatically converts to CompilerError
}
```

---

## Error Display Examples

### Single Error
```
Lexical error at 5:10-5:11: unexpected character: '@'
```

### Parse Error with Context
```
Parse error at 10:5-10:6: unexpected token (expected: identifier, ;) (found: })
```

### Multiple Semantic Errors
```
Semantic errors:
  Semantic error at 15:10-15:15 (undefined variable): variable 'count' not found in this scope
  Semantic error at 20:5-20:10 (type mismatch): expected i32, found String
  Semantic error at 25:1-25:8 (duplicate definition): function 'main' defined multiple times
```

---

## Usage Patterns

### Pattern 1: Simple Error Creation
```rust
// Lexical error
return Err(LexError::new(
    span,
    "unterminated string literal"
).into());

// Parse error
return Err(ParseError::new(
    span,
    "unexpected token",
    vec!["identifier".to_string()],
    "number"
).into());
```

### Pattern 2: Collecting Multiple Errors
```rust
let mut errors = Vec::new();

// Collect all semantic errors
for item in ast.items {
    if let Err(e) = validate_item(&item) {
        errors.push(e);
    }
}

// Return all errors at once
if !errors.is_empty() {
    return Err(CompilerError::Semantic(errors));
}
```

### Pattern 3: Error Propagation
```rust
fn compile_file(path: &Path) -> Result<String> {
    let source = std::fs::read_to_string(path)?;  // Io error
    let tokens = lex(&source)?;                    // Lex error
    let ast = parse(tokens)?;                      // Parse error
    let validated = analyze(ast)?;                 // Semantic errors
    let code = generate(validated)?;               // CodeGen error
    Ok(code)
}
```

---

## Future Enhancements

### Planned Improvements

1. **codespan-reporting Integration** (Task 28.1)
   - Beautiful error displays with source code snippets
   - Color-coded error messages
   - Caret indicators pointing to exact error locations
   - Multi-line error spans

2. **Error Codes** (Future)
   - Unique error codes (e.g., E0001, E0002)
   - Detailed error explanations
   - Links to documentation

3. **Suggestions and Fixes** (Future)
   - "Did you mean...?" suggestions
   - Automatic fix suggestions
   - Quick-fix code actions for IDEs

4. **Error Recovery** (Task 28.3)
   - Continue parsing after errors
   - Report multiple parse errors in one pass
   - Synchronization points for error recovery

5. **Structured Error Data** (Future)
   - Machine-readable error format (JSON)
   - IDE integration support
   - Language server protocol (LSP) compatibility

---

## Error Message Guidelines

When adding new error messages, follow these guidelines:

### 1. Be Specific
❌ Bad: "syntax error"  
✅ Good: "expected ';' after statement"

### 2. Provide Context
❌ Bad: "type error"  
✅ Good: "expected i32, found String"

### 3. Suggest Solutions
❌ Bad: "undefined variable"  
✅ Good: "variable 'count' not found in this scope. Did you mean 'counter'?"

### 4. Use Consistent Terminology
- Use "expected" and "found" for parse errors
- Use "not found in this scope" for undefined variables
- Use "cannot" for impossible operations

### 5. Reference Crusty Concepts
❌ Bad: "use statement not supported"  
✅ Good: "#include directives are not supported. Use #import instead"

---

## Testing

The error module includes comprehensive unit tests:

```rust
#[test]
fn test_position_display() { ... }

#[test]
fn test_span_display() { ... }

#[test]
fn test_lex_error() { ... }

#[test]
fn test_parse_error() { ... }

#[test]
fn test_semantic_error() { ... }

#[test]
fn test_compiler_error_conversion() { ... }

#[test]
fn test_semantic_error_kinds() { ... }
```

**Test Coverage**: All error types and conversions are tested.

---

## Implementation Status

### ✅ Complete
- Core error types (Position, Span, all error structs)
- Error conversion traits
- Display implementations
- Unit tests
- Basic error messages

### ⏳ In Progress
- Specific error messages for all language features
- Error recovery in parser
- Multiple error reporting in parser

### ⏳ Not Started
- codespan-reporting integration
- Error codes and documentation
- Suggestion system
- IDE integration support

---

## See Also

- [requirements.md](requirements.md) - Requirement 11: Report Compilation Errors
- [tasks.md](tasks.md) - Task 3: Error handling infrastructure (complete)
- [tasks.md](tasks.md) - Task 28: Error message improvements (future)
- [src/error.rs](../../src/error.rs) - Error module implementation

---

*This document describes the error handling architecture as of Phase 1. Error messages and features will be expanded as the compiler matures.*
