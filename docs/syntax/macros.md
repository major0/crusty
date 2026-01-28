# Macros

## Introduction

Crusty supports macro definitions using the `#define` directive, which translates to Rust's `macro_rules!`. Macro invocations use a double-underscore naming convention to distinguish them from regular function calls.

## Rationale

The `#define` syntax is familiar to C programmers while providing access to Rust's powerful macro system. The double-underscore naming convention (`__macro__`) clearly distinguishes macro invocations from regular function calls and type-scoped calls.

## Examples

### Simple Macro Definition

```crusty
#define __MAX__(a, b) ((a) > (b) ? (a) : (b))

int result = __MAX__(10, 20);
```

### Macro with Multiple Statements

```crusty
#define __SWAP__(a, b) { \
    var temp = a; \
    a = b; \
    b = temp; \
}
```

### Rust Macro Invocation

Crusty code calls Rust macros using double-underscore naming:

```crusty
__println__!("Hello, {}!", name);
__vec__![1, 2, 3];
__format__!("{} + {} = {}", a, b, a + b);
```

These translate to Rust (removing double-underscores):
```rust
println!("Hello, {}!", name);
vec![1, 2, 3];
format!("{} + {} = {}", a, b, a + b);
```

### Distinguishing Macros from Type-Scoped Calls

The parser distinguishes between type-scoped calls and macro invocations:
- **Type-scoped call**: `@Type->method()` - uses `@` prefix with `->` separator
- **Macro invocation**: `__macro_name__!(...)` - uses double-underscore prefix/suffix with `!`

Examples:
```crusty
@Vec->new()              // Type-scoped call → Vec::new()
__vec__![1, 2, 3]        // Macro invocation → vec![1, 2, 3]
@Option->None            // Type-scoped call → Option::None
__println__!("hello")    // Macro invocation → println!("hello")
```

## Formal Grammar

```ebnf
macro_def   ::= '#define' identifier '(' params? ')' macro_body
macro_body  ::= token+
macro_call  ::= identifier '!' '(' args? ')'
            |   identifier '!' '[' args? ']'
            |   identifier '!' '{' args? '}'
```
