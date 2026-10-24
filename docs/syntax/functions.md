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

## Function-Pointer Types

A function type is written `fn(T1, T2) -> R`, matching Rust's own spelling. It
denotes a pointer to any function with that signature, and is usable anywhere a
type is — a parameter, a `let` type, or a struct field. A return-less
`fn(T1, T2)` defaults to a unit (`()`) return.

```c
int apply(fn(int) -> int f, int v) {
    return f(v);
}

int double_it(int x) {
    return x * 2;
}

void main() {
    let r = apply(double_it, 21);   // pass a named function; r == 42
}
```
Translates to:
```rust
pub fn apply(f: fn(i32) -> i32, v: i32) -> i32 {
    return f(v);
}
pub fn double_it(x: i32) -> i32 {
    return x * 2;
}
pub fn main() {
    let r = apply(double_it, 21);
}
```

A struct field can hold a function pointer — the C `file_operations` idiom:
```c
struct Ops {
    fn(int, int) -> int combine;
}
```
Assign a named function or a non-capturing anonymous function (see
[nested and anonymous functions](nested-functions.md)) to the field, and call
it through the field. A *capturing* anonymous function is not a bare `fn`
pointer and cannot be stored this way (a boxed `dyn Fn` is not yet supported).

## Formal Grammar

```ebnf
function_decl = [attributes] ["static"] type_expr IDENT "(" [param_list] ")" block ;
param_list    = param ("," param)* ;
param         = type_expr IDENT ;
fn_ptr_type   = "fn" "(" [type_list] ")" ["->" type_expr] ;
type_list     = type_expr ("," type_expr)* ;
```

A `fn_ptr_type` is a `type_expr`, so it is valid in any type position. The
anonymous-function value form (`__func__`) and named nested functions are
documented in [nested and anonymous functions](nested-functions.md).
