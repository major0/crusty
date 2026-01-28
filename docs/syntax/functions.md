# Functions

## Introduction

Functions in Crusty use C-style syntax with the return type before the function name. This is a key design principle that distinguishes Crusty from Rust's `fn` keyword syntax.

## Rationale

C-style function declarations are familiar to most programmers and clearly indicate the return type at the start of the declaration. This makes Crusty code immediately readable to C, C++, and Java developers.

## Examples

### Basic Function

```crusty
int add(int a, int b) {
    return a + b;
}
```

Translates to Rust:
```rust
pub fn add(a: i32, b: i32) -> i32 {
    return a + b;
}
```

### Void Function

```crusty
void greet(char* name) {
    __println__("Hello, {}!", name);
}
```

Translates to Rust:
```rust
pub fn greet(name: &str) {
    println!("Hello, {}!", name);
}
```

### Static Function

Static functions are module-private (no `pub` in Rust):

```crusty
static int helper(int x) {
    return x * 2;
}
```

Translates to Rust:
```rust
fn helper(x: i32) -> i32 {
    return x * 2;
}
```

### Function with No Parameters

```crusty
int get_value() {
    return 42;
}
```

Translates to Rust:
```rust
pub fn get_value() -> i32 {
    return 42;
}
```

## Formal Grammar

```ebnf
function    ::= visibility? 'static'? type identifier '(' params? ')' block
visibility  ::= 'pub'
type        ::= primitive_type | identifier | pointer_type | array_type
params      ::= param (',' param)*
param       ::= type identifier
block       ::= '{' statement* '}'
```
