# Design: C-Style Variable Declarations

**Status**: In Progress  
**Related Requirements**: requirements.md

## Overview

Implement C-style variable declaration syntax where `Type name = value;` is the primary form and `let`/`var` are optional keywords. When neither is specified, `let` (immutable) is assumed.

## Design Goals

1. **C-Natural Syntax**: Make `int x = 42;` the primary, documented way to declare variables
2. **Preserve Type Inference**: Keep `let x = 42;` for Rust's type inference feature
3. **Explicit Mutability**: Use `var` prefix for mutable variables
4. **Clean Syntax**: No mixing of casting with declarations
5. **Backward Compatible**: Existing `let`/`var` syntax continues to work

## Syntax Design

### Grammar Rules

```
variable_declaration ::= [let | var] type identifier '=' expression ';'
                      | let identifier '=' expression ';'
                      | var identifier '=' expression ';'

const_declaration ::= const [type] identifier '=' expression ';'
```

### Parsing Logic

**When parsing a statement starting with a type token:**
1. Check if current token is `let` or `var`
   - If yes: consume keyword, parse type, parse name
   - If no: assume `let`, parse type, parse name
2. Expect `=` token
3. Parse initializer expression
4. Expect `;` token

**Key Insight**: The parser must distinguish between:
- `int x = 42;` (type `int`, name `x`) - implicit let
- `let x = 42;` (no type, name `x`) - explicit let with inference
- `let int x = 42;` (type `int`, name `x`) - explicit let with type

## Parser Implementation

### Current State

The parser currently:
- ✅ Accepts `let name = value;` (inference)
- ✅ Accepts `var name = value;` (inference)
- ✅ Rejects `let name: Type = value;` (Rust-style)
- ❌ Does NOT accept `Type name = value;` (implicit let)
- ❌ Does NOT accept `let Type name = value;` (explicit let with type)
- ❌ Does NOT accept `var Type name = value;` (explicit var with type)

### Required Changes

#### 1. Update `parse_let_statement()`

**Current Logic:**
```rust
fn parse_let_statement() {
    expect(Let);
    name = parse_identifier();
    // Reject colon
    expect(Assign);
    init = parse_expression();
}
```

**New Logic:**
```rust
fn parse_let_statement() {
    expect(Let);
    
    // Check if next token is a type
    if is_type_token() {
        ty = parse_type();
        name = parse_identifier();
    } else {
        // Type inference
        name = parse_identifier();
        ty = None;
    }
    
    expect(Assign);
    init = parse_expression();
}
```

#### 2. Update `parse_var_statement()`

**Current Logic:**
```rust
fn parse_var_statement() {
    expect(Var);
    name = parse_identifier();
    expect(Assign);
    init = parse_expression();
}
```

**New Logic:**
```rust
fn parse_var_statement() {
    expect(Var);
    
    // Check if next token is a type
    if is_type_token() {
        ty = parse_type();
        name = parse_identifier();
    } else {
        // Type inference
        name = parse_identifier();
        ty = None;
    }
    
    expect(Assign);
    init = parse_expression();
}
```

#### 3. Update `parse_statement()` to Handle Implicit Let

**Current Logic:**
```rust
fn parse_statement() {
    match current_token {
        Let => parse_let_statement(),
        Var => parse_var_statement(),
        // ... other statements
    }
}
```

**New Logic:**
```rust
fn parse_statement() {
    match current_token {
        Let => parse_let_statement(),
        Var => parse_var_statement(),
        
        // Check for implicit let (Type name = value)
        _ if is_type_token() => {
            // Look ahead to check if this is a declaration
            if looks_like_declaration() {
                parse_implicit_let_statement()
            } else {
                parse_expression_statement()
            }
        }
        
        // ... other statements
    }
}
```

#### 4. Add `parse_implicit_let_statement()`

```rust
fn parse_implicit_let_statement() {
    // Parse type
    ty = parse_type();
    
    // Parse name
    name = parse_identifier();
    
    // Expect assignment
    expect(Assign);
    
    // Parse initializer
    init = parse_expression();
    
    expect(Semicolon);
    
    // Create Let statement with type
    return Statement::Let {
        name,
        ty: Some(ty),
        init: Some(init),
        mutable: false,
    };
}
```

#### 5. Add `looks_like_declaration()` Helper

```rust
fn looks_like_declaration() -> bool {
    // Save current position
    // Look ahead: Type Identifier '='
    // If we see this pattern, it's a declaration
    // Otherwise, it's an expression
    
    if !is_type_token() {
        return false;
    }
    
    // Peek ahead past the type
    let next = peek_ahead(1);
    if !matches!(next, Identifier) {
        return false;
    }
    
    // Peek ahead past the identifier
    let next_next = peek_ahead(2);
    matches!(next_next, Assign)
}
```

### Lookahead Strategy

The parser needs lookahead to distinguish:
- `int x = 42;` (declaration)
- `int(x)` (cast expression)
- `int + 5` (expression with identifier named `int`)

**Strategy**: Look for pattern `Type Identifier '='`
- If found: Parse as declaration
- If not: Parse as expression

## Code Generator Updates

### Current Behavior

The code generator currently:
- Generates `let x = value;` for Crusty target (no type)
- Wraps in cast if type present: `let x = (Type)value;`

### Required Behavior

The code generator should:
- Generate `Type name = value;` for C-style declarations
- Generate `let name = value;` for inference
- NOT generate casting in declarations

**Update Logic:**
```rust
fn generate_let_statement(stmt: &Statement::Let) {
    match target {
        Rust => {
            write("let ");
            if mutable { write("mut "); }
            write(name);
            if let Some(ty) = ty {
                write(": ");
                write(generate_type(ty));
            }
            write(" = ");
            write(generate_expr(init));
        }
        Crusty => {
            // If type is present, use C-style
            if let Some(ty) = ty {
                write(generate_type(ty));
                write(" ");
                write(name);
            } else {
                // Use let for inference
                write("let ");
                write(name);
            }
            write(" = ");
            write(generate_expr(init));
        }
    }
}
```

## AST Representation

No changes needed to AST. The `Statement::Let` already has:
```rust
Statement::Let {
    name: Ident,
    ty: Option<Type>,  // Some for explicit type, None for inference
    init: Option<Expression>,
    mutable: bool,
}
```

## Examples

### Input Crusty Code → AST → Output Crusty Code

```c
// Input: C-style (implicit let)
int x = 42;

// AST:
Statement::Let {
    name: "x",
    ty: Some(Type::Primitive(Int)),
    init: Some(Literal(42)),
    mutable: false,
}

// Output: C-style
int x = 42;
```

```c
// Input: Explicit let with type
let int x = 42;

// AST: (same as above)
Statement::Let {
    name: "x",
    ty: Some(Type::Primitive(Int)),
    init: Some(Literal(42)),
    mutable: false,
}

// Output: C-style (preferred)
int x = 42;
```

```c
// Input: Type inference
let x = 42;

// AST:
Statement::Let {
    name: "x",
    ty: None,
    init: Some(Literal(42)),
    mutable: false,
}

// Output: Type inference
let x = 42;
```

## Edge Cases

### 1. Type Name vs Variable Name

**Problem**: How to distinguish `int x = 42;` from `x = 42;` where `x` is a variable?

**Solution**: Use lookahead. If pattern is `Type Identifier '='`, it's a declaration.

### 2. Cast vs Declaration

**Problem**: How to distinguish `int(x)` (cast) from `int x = 42;` (declaration)?

**Solution**: Look for `'='` after identifier. Cast has `(`, declaration has `=`.

### 3. Typedef Names

**Problem**: Custom type names like `MyInt x = 42;`

**Solution**: `is_type_token()` returns true for identifiers (could be typedef). Use lookahead to confirm declaration pattern.

### 4. Pointer/Reference Types

**Problem**: `int* ptr = NULL;` or `int& ref = x;`

**Solution**: `parse_type()` already handles pointer/reference syntax.

## Testing Strategy

### Unit Tests

1. **Parse C-style declarations**
   - `int x = 42;`
   - `MyInt x = 32;`
   - `int* ptr = NULL;`

2. **Parse explicit let with type**
   - `let int x = 42;`
   - `let MyInt x = 32;`

3. **Parse var with type**
   - `var int x = 42;`
   - `var MyInt x = 32;`

4. **Parse type inference**
   - `let x = 42;`
   - `var x = 42;`

5. **Reject Rust-style**
   - `let x: int = 42;` → error
   - `var x: int = 42;` → error

6. **Code generation**
   - C-style input → C-style output
   - Inference input → inference output

### Integration Tests

1. Update all typedef tests to use C-style
2. Update all nested function tests to use C-style
3. Verify roundtrip: parse → generate → parse

## Documentation Updates

### SYNTAX_REFERENCE.md

Update variable declaration section:

```markdown
## Variable Declarations

### Immutable Variables (Primary Syntax)

```c
int x = 42;              // C-style (recommended)
MyInt y = 32;            // With typedef
let int x = 42;          // Explicit let (equivalent)
let x = 42;              // Type inference
```

### Mutable Variables

```c
var int x = 42;          // C-style mutable
var x = 42;              // Mutable with inference
```

### Constants

```c
const int MAX = 100;     // Explicit type
const MAX = 100;         // Type inference
```
```

### Examples

Update all `.crst` files to use C-style as primary:

```c
// typedef_demo.crst
int calculate(MyInt a, MyFloat b) {
    Number result = a + 10;  // C-style
    return result;
}

void main() {
    MyInt a = 10;            // C-style
    MyFloat b = 3.14;        // C-style
    int result = calculate(a, b);
}
```

## Implementation Plan

1. ✅ Update requirements.md
2. ✅ Create design.md
3. 🔨 Update parser to support `let Type name = value;`
4. 🔨 Update parser to support `var Type name = value;`
5. 🔨 Update parser to support `Type name = value;` (implicit let)
6. 🔨 Update code generator for C-style output
7. 🔨 Update all tests to use C-style syntax
8. 🔨 Update all examples to use C-style syntax
9. 🔨 Update SYNTAX_REFERENCE.md
10. 🔨 Run full test suite and validate

## Success Criteria

- ✅ Parser accepts `int x = 42;` (implicit let)
- ✅ Parser accepts `let int x = 42;` (explicit let)
- ✅ Parser accepts `var int x = 42;` (explicit var)
- ✅ Parser rejects `let x: int = 42;` (Rust-style)
- ✅ Code generator emits C-style for explicit types
- ✅ Code generator emits inference for no type
- ✅ All tests pass
- ✅ Documentation shows C-style as primary

## Risks and Mitigations

### Risk: Ambiguity with Expressions

**Risk**: `int x = 42;` could be confused with expression `int(x) = 42;`

**Mitigation**: Use lookahead to check for declaration pattern. If pattern doesn't match, parse as expression.

### Risk: Breaking Existing Code

**Risk**: Existing code using `let x = 42;` might break

**Mitigation**: This syntax continues to work. Only adding new syntax, not removing old.

### Risk: Parser Complexity

**Risk**: Lookahead adds complexity to parser

**Mitigation**: Implement robust lookahead helper. Add comprehensive tests.

## Future Enhancements

1. **Global Variables**: Extend to support global variable declarations
2. **Multiple Declarations**: Support `int x = 1, y = 2;` (C-style)
3. **Uninitialized Variables**: Support `int x;` (declare without init)
4. **Array Declarations**: Support `int arr[10];` syntax

---

**Author**: Kiro AI Assistant  
**Date**: 2026-01-31  
**Status**: Ready for Implementation
