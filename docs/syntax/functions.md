# Functions

## Introduction

Crusty uses C-style function declarations where the return type precedes the function name. This provides a familiar syntax for C developers while mapping directly to Rust function definitions.

## Rationale

C-style function syntax was chosen because it is the most widely recognized function declaration format. The transpiler handles the mechanical translation to Rust's `fn name() -> Type` syntax, so developers can write in the style they're comfortable with.

## Examples

### Basic Function
```c
int add(int a, int b) {
    return a + b;
}
```
Translates to:
```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

### Void Functions
```c
void greet() {
    println!("Hello, Crusty!");
}
```
Translates to:
```rust
pub fn greet() {
    println!("Hello, Crusty!");
}
```

### Static (Private) Functions
```c
static int helper(int x) {
    return x * 2;
}
```
Translates to:
```rust
fn helper(x: i32) -> i32 {
    x * 2
}
```

The `static` keyword in Crusty maps to private visibility in Rust (no `pub` modifier).

## Formal Grammar

```ebnf
function_decl = [attributes] ["static"] type_expr IDENT "(" [param_list] ")" block ;
param_list    = param ("," param)* ;
param         = type_expr IDENT ;
```
