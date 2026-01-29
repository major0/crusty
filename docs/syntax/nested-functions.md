# Nested Functions (Closures)

## Introduction

Crusty supports defining functions within functions (nested functions) that can capture variables from the enclosing scope. This provides familiar C-style function syntax for what Rust implements as closures.

## Rationale

Rust closures use a unique `|args| body` syntax that is unfamiliar to C developers. Crusty allows nested functions to be written using standard function declaration syntax, and the transpiler automatically translates them to Rust closures with the appropriate capture semantics.

## Examples

### Basic Nested Function
```c
void outer_function() {
    int captured_value = 42;

    int add_value(int x) {
        return x + captured_value;
    }

    let result = add_value(10);  // Returns 52
}
```
Translates to:
```rust
pub fn outer_function() {
    let captured_value = 42;

    let add_value = |x: i32| -> i32 {
        x + captured_value
    };

    let result = add_value(10);
}
```

### Mutable Capture
```c
void counter_example() {
    int counter = 0;

    void increment() {
        counter = counter + 1;
    }

    increment();
    increment();
    // counter is now 2
}
```
Translates to a `FnMut` closure since `counter` is modified.

### Passing as Function Parameter
```c
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

## Scoping Rules

1. Nested functions can only capture variables defined before the nested function declaration
2. Variables defined after a nested function are not accessible to that function
3. Multiple nested functions can capture and share the same outer variables
4. Nested functions cannot be declared `static`
5. Multi-level nesting is not supported (nested functions cannot contain nested functions)

## Capture Classification

The transpiler automatically determines the capture mode:
- Immutable (Fn): variable is only read
- Mutable (FnMut): variable is modified
- Move (FnOnce): variable ownership is transferred

## Formal Grammar

```ebnf
nested_func   = type_spec IDENT "(" [param_list] ")" block ;
func_ptr_type = type_spec "(" "*" IDENT ")" "(" [type_list] ")" ;
```

Nested functions use the same syntax as top-level functions but appear within a function body.
