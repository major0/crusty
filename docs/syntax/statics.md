# Static Variables

## Introduction

A `static` item at the top level declares a global variable with a fixed memory
location for the whole program run. It maps directly to a Rust `static`.

## Syntax

```c
static int COUNTER = 0;
static mut int TICKS = 100;
```

Translates to:

```rust
pub static COUNTER: i32 = 0;
pub static mut TICKS: i32 = 100;
```

- A plain `static` is immutable.
- `static mut` marks a mutable global. As in Rust, *reading or writing* a
  `mut` static in the generated code requires an `unsafe` context.
- A top-level static is public (`pub static`).
- An initializer is required.

## `static` on a variable vs. a function

Crusty's `static` keyword is overloaded by position, and the two uses are
disambiguated by what follows the name:

| Form                              | Meaning                    | Rust                      |
|-----------------------------------|----------------------------|---------------------------|
| `static Type name = value;`       | static **variable** (public) | `pub static name: Type = value;` |
| `static Type name(params) { … }`  | **private function**       | `fn name(…) { … }`        |

A `(` after the name makes it a function declaration (see
[Functions](functions.md)); an `=` makes it a static variable. The parser tries
the variable form first and falls through to the function form on the `(`.

## Formal Grammar

```ebnf
static_item = "static" ["mut"] type IDENT "=" expr ";" ;
```
