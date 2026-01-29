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

## Nested Functions (Closures)

Crusty supports defining functions within functions (nested functions) that can capture variables from the enclosing scope. This provides a familiar C-style syntax for closures.

### Rationale

Nested functions allow developers to write closures using familiar function syntax rather than Rust's closure syntax. The transpiler automatically determines the appropriate Rust closure trait (Fn, FnMut, FnOnce) based on how captured variables are used.

### Examples

#### Immutable Capture

When a nested function only reads a captured variable, it becomes an `Fn` closure:

```crusty
void outer_function() {
    int captured_value = 42;
    
    int add_value(int x) {
        return x + captured_value;  // Immutable capture
    }
    
    let result = add_value(10);  // Returns 52
}
```

Translates to Rust:
```rust
pub fn outer_function() {
    let captured_value = 42;
    
    let add_value = |x: i32| -> i32 {
        x + captured_value
    };
    
    let result = add_value(10);
}
```

#### Mutable Capture

When a nested function modifies a captured variable, it becomes an `FnMut` closure:

```crusty
void counter_example() {
    int counter = 0;
    
    void increment() {
        counter = counter + 1;  // Mutable capture
    }
    
    increment();
    increment();
    // counter is now 2
}
```

Translates to Rust:
```rust
pub fn counter_example() {
    let mut counter = 0;
    
    let mut increment = || {
        counter = counter + 1;
    };
    
    increment();
    increment();
}
```

#### Passing Nested Functions as Parameters

Nested functions can be passed to other functions using function pointer syntax:

```crusty
void apply(int (*func)(int), int value) {
    return func(value);
}

void outer() {
    int base = 10;
    
    int add_base(int x) {
        return x + base;
    }
    
    let result = apply(add_base, 5);  // Returns 15
}
```

Translates to Rust:
```rust
fn apply<F>(func: F, value: i32) -> i32 
where F: Fn(i32) -> i32 
{
    func(value)
}

pub fn outer() {
    let base = 10;
    
    let add_base = |x: i32| -> i32 {
        x + base
    };
    
    let result = apply(add_base, 5);
}
```

### Scoping Rules

1. Nested functions can only capture variables declared **before** the nested function
2. Variables declared **after** a nested function are not accessible to that function
3. Multiple nested functions can capture and share the same outer variables
4. Captures are automatically classified based on usage:
   - **Immutable (Fn)**: Variable is only read
   - **Mutable (FnMut)**: Variable is modified
   - **Move (FnOnce)**: Variable ownership is transferred

### Restrictions

- Nested functions cannot be declared `static`
- Multi-level nesting is not supported (nested functions cannot contain nested functions)

## Formal Grammar

```ebnf
function        ::= visibility? 'static'? type identifier '(' params? ')' block
nested_function ::= type identifier '(' params? ')' block
visibility      ::= 'pub'
type            ::= primitive_type | identifier | pointer_type | array_type
params          ::= param (',' param)*
param           ::= type identifier
block           ::= '{' statement* '}'
```
