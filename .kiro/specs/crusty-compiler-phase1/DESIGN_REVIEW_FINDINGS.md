# Design Document Review Findings

## Date: 2026-01-27

## Summary

Reviewed the design document to validate it correctly specifies Crusty's C-like syntax rather than Rust syntax. Found **1 critical issue** that contradicts the requirements document.

## Critical Issues

### Issue 1: `Fn` keyword listed in TokenKind enum

**Location**: Line 408 in design.md, Section "2. Lexer Module"

**Problem**: The TokenKind enum lists `Fn` as a keyword:
```rust
pub enum TokenKind {
    // Keywords
    Fn, Let, Var, Const, Static, If, Else, While, For, Return,
    ...
}
```

**Why this is wrong**: 
- Requirement 6.4 explicitly states: "THE Parser SHALL support C-like function declarations with return types specified before the function name"
- Requirement 6.5: "THE Parser SHALL support void return type for functions that do not return a value"
- Requirement 6.6: "THE Parser SHALL support function declarations with typed parameters in C-style syntax"

**Correct Crusty syntax** (C-style):
```c
int add(int a, int b) { return a + b; }
void print_hello() { println!("Hello"); }
static int helper(int x) { return x * 2; }
```

**NOT Rust syntax**:
```rust
fn add(a: int, b: int) -> int { return a + b; }  // WRONG for Crusty!
```

**Impact**: 
- The lexer was incorrectly implemented with `Fn` as a keyword
- This has been fixed in the implementation (commit 7b7b8b5)
- The design document needs to be updated to remove `Fn` from the TokenKind enum

**Recommendation**: Remove `Fn` from the TokenKind enum in the design document.

## Positive Findings

The rest of the design document correctly specifies C-like syntax:

### ✅ Correct: Function Structure in AST
```rust
pub struct Function {
    pub visibility: Visibility,
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,  // ✅ Optional return type (void = None)
    pub body: Block,
    pub doc_comments: Vec<String>,
}
```
This correctly models C-style functions where:
- Return type comes before function name
- `void` functions have `return_type: None`
- Parameters are typed in C-style

### ✅ Correct: Type Keywords
The design correctly includes C-style type keywords:
- `Int, I32, I64, U32, U64` - integer types
- `Float, F32, F64` - floating-point types
- `Bool, Char, Void` - other primitive types

### ✅ Correct: C-like Control Flow
The design correctly specifies C-like control flow:
- `if/else` statements (not `if let`)
- `while` loops
- `for` loops (both C-style and for-in)
- `switch` statements (not `match`)

### ✅ Correct: Crusty-Specific Syntax
The design correctly documents Crusty's unique syntax features:
- `@Type.method()` for type-scoped calls (translates to `Type::method()`)
- `!` suffix for macros (same as Rust: `println!(...)`)
- `.label:` for labeled loops (translates to `'label:`)
- `#define` for macro definitions (translates to `macro_rules!`)
- `#use` for imports (translates to `use`)

### ✅ Correct: Translation Examples
The design provides correct translation examples throughout:
- C-style casts `(type)expr` → Rust `expr as type`
- `sizeof(type)` → `std::mem::size_of::<type>()`
- `NULL` → `@Option.None` → `Option::None`
- Static functions → private Rust functions (no `pub`)
- Non-static functions → public Rust functions (`pub fn`)

## Recommendations

### Required Changes

1. **Update TokenKind enum** (Line 408):
   ```rust
   pub enum TokenKind {
       // Keywords
       Let, Var, Const, Static, If, Else, While, For, Return,  // Remove Fn
       Break, Continue, Struct, Enum, Typedef, Namespace, Extern,
       ...
   }
   ```

2. **Add clarification** in the "Key Design Principles" section:
   ```markdown
   - **C-like Function Syntax**: Crusty uses C-style function declarations 
     with return types before function names (e.g., `int main()`, `void foo()`),
     NOT Rust's `fn` keyword syntax.
   ```

3. **Add example** in the "Architecture" section showing Crusty function syntax:
   ```crusty
   // Crusty function syntax (C-style)
   int add(int a, int b) {
       return a + b;
   }
   
   void print_message(char* msg) {
       println!("{}", msg);
   }
   
   static int helper(int x) {
       return x * 2;
   }
   ```

### Optional Improvements

1. Add a "Syntax Comparison" section showing Crusty vs C vs Rust side-by-side
2. Add more complete Crusty code examples throughout the document
3. Clarify that Crusty is "C-like" not "C-compatible" (it's a new language with C-inspired syntax)

## Conclusion

The design document is **mostly correct** with only **one critical error**: the inclusion of `Fn` as a keyword. This has already been fixed in the implementation. The design document should be updated to match the corrected implementation and align with the requirements.

**Status**: ✅ Implementation is correct (as of commit 7b7b8b5)
**Action Required**: Update design document to remove `Fn` keyword
