# Macros

## Introduction

Crusty supports macro definitions using the `#define` directive, which translates to Rust's `macro_rules!`.

## Rationale

The `#define` syntax is familiar to C programmers while providing access to Rust's powerful macro system.

## Examples

### Simple Macro

```crusty
#define MAX(a, b) ((a) > (b) ? (a) : (b))

int result = MAX(10, 20);
```

### Macro with Multiple Statements

```crusty
#define SWAP(a, b) { \
    var temp = a; \
    a = b; \
    b = temp; \
}
```

### Rust Macro Invocation

Crusty code can call Rust macros directly:

```crusty
println!("Hello, {}!", name);
vec![1, 2, 3];
format!("{} + {} = {}", a, b, a + b);
```

## Formal Grammar

```ebnf
macro_def   ::= '#define' identifier '(' params? ')' macro_body
macro_body  ::= token+
macro_call  ::= identifier '!' '(' args? ')'
            |   identifier '!' '[' args? ']'
            |   identifier '!' '{' args? '}'
```
