# Macro Syntax Correction

## Issue

The documentation incorrectly shows Crusty macros with the `!` suffix. This is incorrect.

## Correct Syntax

### Crusty Macro Invocation
```crusty
__println__("Hello, world!")
__vec__[1, 2, 3]
__format__("Value: {}", x)
__assert__(x > 0)
```

**Key Points:**
- Crusty macros use double-underscore prefix and suffix: `__macro_name__`
- Crusty macros do NOT use the `!` suffix
- The `!` is added during transpilation to Rust

### Rust Translation
```rust
println!("Hello, world!")
vec![1, 2, 3]
format!("Value: {}", x)
assert!(x > 0)
```

**Translation Rules:**
- Remove double-underscore prefix and suffix
- Add `!` suffix for Rust macro invocation
- `__println__()` → `println!()`
- `__vec__[]` → `vec![]`

### Reserved Pattern

The double-underscore pattern (leading AND trailing) is **reserved for macros only**.

**Prohibited:**
```crusty
// INVALID - functions cannot use double-underscore pattern
void __my_function__() {  // ERROR
    // ...
}

int __helper__() {  // ERROR
    return 42;
}
```

**Allowed:**
```crusty
// Valid - single leading underscore for private functions
void _helper() {
    // ...
}

// Valid - normal function names
void my_function() {
    // ...
}
```

## Semantic Analysis Rule

The Semantic_Analyzer SHALL:
1. Detect function definitions with leading AND trailing double-underscores
2. Report an error: "Function names cannot use double-underscore pattern (reserved for macros)"
3. Suggest removing the double-underscores or using single underscore for private functions

## Files to Update

All documentation files need correction:
- README.md
- CONTRIBUTING.md
- requirements.md
- design.md
- tasks.md
- REVIEW_FINDINGS.md
- COMPREHENSIVE_REVIEW_2.md
- FIXES_COMPLETED.md

## Correct Examples

### Hello World
```crusty
void main() {
    __println__("Hello, Crusty!");
}
```

### Macros and Vectors
```crusty
void main() {
    __println__("Creating a vector...");
    let v = @Vec->new();
    v.push(1);
    v.push(2);
    __println__("Vector: {:?}", v);
}
```

### Macro Definitions
```crusty
#define __MAX__(a, b) ((a) > (b) ? (a) : (b))
#define __SQUARE__(x) ((x) * (x))
#define __DEBUG_PRINT__(msg) __println__("DEBUG: {}", msg)
```

## Summary of Changes

**Incorrect (OLD):**
- `__println__!("text")`
- `__vec__![1, 2, 3]`
- `__macro_name__!(args)`

**Correct (NEW):**
- `__println__("text")`
- `__vec__[1, 2, 3]`
- `__macro_name__(args)` or `__macro_name__[args]` or `__macro_name__{args}`

The `!` is a Rust-specific syntax that is added during transpilation, not part of Crusty syntax.
