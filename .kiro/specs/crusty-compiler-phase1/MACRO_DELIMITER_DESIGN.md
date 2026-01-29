# Macro Delimiter Type Design

## Problem Statement

The initial implementation of `#define` macros in Crusty had a design flaw: macro invocations were being parsed as regular function calls (`Expression::Call`), which caused the semantic analyzer to incorrectly validate them as undefined functions.

## Root Cause

When the parser encountered `__MACRO__(args)`, it was:
1. Parsing `__MACRO__` as an identifier (`Expression::Ident`)
2. Parsing `(args)` as a function call (`Expression::Call`)

This meant the semantic analyzer tried to validate macro invocations as function calls, leading to "undefined variable" and "cannot call non-function type" errors.

## Solution: Explicit Delimiter Types

Macros in Crusty now have **explicit delimiter types** that are determined at the `#define` declaration, not at invocation time.

### Delimiter Types

```rust
pub enum MacroDelimiter {
    None,       // __MACRO__ (no parameters, no delimiters)
    Parens,     // __MACRO__(args)
    Brackets,   // __MACRO__[args]
    Braces,     // __MACRO__{args}
}
```

### Examples

```crusty
// Define macros with different delimiter types
#define __PI__ 3.14159                    // MacroDelimiter::None
#define __MAX__(a, b) ((a) > (b) ? (a) : (b))  // MacroDelimiter::Parens
#define __VEC__[items] { vec![items] }    // MacroDelimiter::Brackets
#define __BLOCK__{code} { code }          // MacroDelimiter::Braces

// Invocations must match the delimiter type
let pi = __PI__;                          // OK - no delimiter
let max_val = __MAX__(10, 20);            // OK - parentheses
let my_vec = __VEC__[1, 2, 3];            // OK - brackets
let result = __BLOCK__{ x + y };          // OK - braces

// ERROR: Wrong delimiter type
let wrong = __MAX__[10, 20];              // ERROR - __MAX__ expects parentheses
```

## Implementation Changes

### 1. AST Updates

Added `MacroDelimiter` enum and `delimiter` field to `MacroDefinition`:

```rust
pub struct MacroDefinition {
    pub name: Ident,
    pub params: Vec<Ident>,
    pub body: Vec<crate::lexer::Token>,
    pub delimiter: MacroDelimiter,  // NEW
}
```

### 2. Parser Updates

#### `parse_define` Method

The parser now detects the delimiter type when parsing `#define`:

```rust
let delimiter = if self.check(&TokenKind::LParen) {
    // Parse parameters in parentheses
    MacroDelimiter::Parens
} else if self.check(&TokenKind::LBracket) {
    // Parse parameters in brackets
    MacroDelimiter::Brackets
} else if self.check(&TokenKind::LBrace) {
    // Parse parameters in braces
    MacroDelimiter::Braces
} else {
    // No delimiter - constant macro
    MacroDelimiter::None
};
```

#### `parse_postfix` Method (TODO - Task 15.2)

The parser needs to be updated to:
1. Recognize double-underscore pattern as potential macro invocations
2. Build a macro registry during parsing to track delimiter types
3. Check the macro's delimiter type when parsing invocations
4. Create `Expression::MacroCall` instead of `Expression::Call`
5. Report error if invoked with wrong delimiter type

### 3. Semantic Analyzer

The semantic analyzer already handles `Expression::MacroCall` correctly:

```rust
Expression::MacroCall { .. } => {
    // Macro calls are not type-checked at this stage
    Type::Auto
}
```

This means macros are treated as compile-time constructs that will be expanded by the Rust compiler, not as runtime function calls.

### 4. Code Generator

The code generator needs to be updated to handle different delimiter types when translating to Rust `macro_rules!`.

## Benefits

1. **Type Safety**: Macros are no longer confused with functions
2. **Clear Syntax**: The delimiter type makes the macro's purpose clear
3. **Rust Compatibility**: Maps naturally to Rust's macro invocation syntax
4. **Error Detection**: Can detect when a macro is invoked with the wrong delimiter

## Remaining Work (Task 15.2)

The parser still needs to be updated to:
1. Build a macro registry during the first pass
2. Check delimiter types when parsing macro invocations
3. Create `Expression::MacroCall` for all macro invocations
4. Report errors for delimiter type mismatches

Currently, the parser change in `parse_postfix` only checks for double-underscore pattern with parentheses. It needs to handle all delimiter types.

## Testing

New tests are needed for:
- Parsing macros with brackets delimiter
- Parsing macros with braces delimiter
- Parsing macros with no delimiter
- Error when macro invoked with wrong delimiter type
- Code generation for different delimiter types

## Documentation Updates

Updated:
- `requirements.md`: Requirement 26 now includes delimiter types
- `design.md`: Added section on macro delimiter types with examples
- `tasks.md`: Split task 15 to separate parsing from invocation handling
