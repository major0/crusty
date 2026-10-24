# Nested Functions and Anonymous Functions

## Introduction

Crusty has two ways to write a function inside another function, mapping onto
the two Rust constructs that each have different capabilities:

| Crusty form | Rust target | Hoisted | Captures | Recursive |
|---|---|---|---|---|
| named nested function | nested `fn` item | yes | no | yes |
| `__func__(...)` anonymous function | closure `\|…\| {…}` | no | yes | no |

The rule of thumb: **name it to hoist and recurse; `__func__` it to capture.**
In Rust these two abilities are mutually exclusive (a `fn` item is hoisted and
recursive but cannot capture; a closure captures but is neither), so Crusty
offers a distinct surface for each.

## Named nested functions

A function declared inside another function, using ordinary function syntax,
lowers to a Rust **`fn` item**. It is **hoisted** (callable before its textual
definition), may **recurse** (and mutually recurse with sibling nested
functions), and **cannot capture** enclosing locals — a value it needs is
passed as a parameter.

```c
void outer() {
    // `fact` is called before its definition — nested functions are hoisted.
    let f = fact(5);          // 120

    int fact(int n) {
        if (n <= 1) { return 1; }
        return n * fact(n - 1);   // recursion
    }
}
```
Translates to:
```rust
pub fn outer() {
    let f = fact(5);
    fn fact(n: i32) -> i32 {
        if n <= 1 { return 1; }
        return n * fact(n - 1);
    }
}
```

Because a `fn` item cannot capture, a nested function that needs an outer value
takes it as a parameter:
```c
void outer() {
    int add(int base, int x) { return base + x; }   // `base` is a parameter
    let r = add(10, 5);   // 15
}
```

### Scoping rules
1. A nested function is not visible outside its enclosing function.
2. It **is** usable before its textual definition (hoisting).
3. It may call itself (recursion) and sibling nested functions.
4. It cannot capture enclosing locals — pass them as parameters.
5. Nested functions cannot be declared `static`.
6. Multi-level nesting is not supported (a nested function cannot contain one).

## Anonymous functions (`__func__`)

An anonymous function is a normal function declaration with the `__func__`
keyword in the name slot, written in expression position. It is a **callable
value** that **captures** enclosing locals, and lowers to a Rust closure.

```c
void outer() {
    int base = 100;
    let addbase = int __func__(int x) { return x + base; };  // captures base
    let s = addbase(5);   // 105
}
```
Translates to:
```rust
pub fn outer() {
    let base: i32 = 100;
    let addbase = |x: i32| -> i32 { return x + base; };
    let s = addbase(5);
}
```

- Parameters are **typed**, exactly like any function (`int __func__(int x)`).
- The return type is the optional C-style type before `__func__`; omit it (or
  use `void`) for a unit return.
- It is a **callable value**, not a run-once block: the body runs each time the
  value is called, not at the point it is written.
- A `return` inside the body returns from the **anonymous function** to its
  caller, not from the enclosing function. This is unambiguous because a
  `__func__` closure cannot recurse or name itself.
- `__func__` is a keyword, not a name, so any number of anonymous functions
  coexist — even with identical signatures — and none is entered in the symbol
  table.
- A *recursive* function cannot be written as `__func__` (a closure cannot name
  itself); use a named nested function for recursion.

### Storing a function in a struct field
A struct field whose type is a function pointer (see
[functions](functions.md)) can hold a **non-capturing** `__func__` or a named
function:
```c
struct Ops {
    fn(int, int) -> int combine;
}
```
A *capturing* `__func__` is not a bare `fn` pointer and cannot be stored in a
plain function-pointer field; that case (a boxed `dyn Fn`) is not yet supported.

## Formal Grammar

```ebnf
nested_func   = type_expr IDENT "(" [param_list] ")" block ;   (* fn item *)
anon_function = [type_expr | "void"] "__func__" "(" [param_list] ")" block ;
```

A named nested function uses the same syntax as a top-level function but appears
inside a function body. An anonymous function is the same shape with `__func__`
in the name position, used where an expression is expected.
