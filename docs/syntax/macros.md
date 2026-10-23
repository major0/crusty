# Macros, Constants, and Conditional Compilation

## Introduction

Crusty is a syntax layer over Rust, not a reimplementation of the C
preprocessor. Rather than emulating a weaker `#define`-does-everything model, it
provides a small set of unambiguous C-flavoured shims and otherwise passes
Rust's macro system through unchanged. See [ADR-0001](../adr/0001-macro-handling.md)
for the decision and rationale.

The complete macro surface is four directives:

| Crusty syntax                 | Emits                      | Role                          |
|-------------------------------|----------------------------|-------------------------------|
| `#define __NAME__ value`      | `const NAME: T = value;`   | Named constant (no parameters) |
| `#macro __NAME__(a, b) body`  | `macro_rules! name { … }`  | Parameterised macro           |
| `#ifdef FOO` … `#endif`       | `#[cfg(FOO)]`              | Conditional compilation       |
| `__name__(args)`              | `name!(args)`             | Macro invocation              |

Anything else — raw `macro_rules!`, `#[derive(...)]`, attribute macros,
procedural macros — is not modelled by Crusty. Those already work through
attributes and the `__rust__` escape hatch, so Crusty does not get in the way.

## `#define` — constants only

A `#define` binds a name to a literal value and lowers to a Rust `const`. It is
**constants only**; it takes no parameters, and the type is inferred from the
literal (Crusty does not text-substitute the way C does).

```c
#define __PI__ 3.14159
```
Translates to:
```rust
const PI: f64 = 3.14159;
```

The double-underscores are stripped and the name is emitted verbatim (upper-case
by convention for constants). Type inference follows the literal:

| Literal        | Inferred Rust type |
|----------------|--------------------|
| `42`           | `i32`              |
| `3.14159`      | `f64`              |
| `"text"`       | `&str`             |
| `'c'`          | `char`             |
| `true`/`false` | `bool`             |

A leading `-` on a numeric literal is accepted (`#define __MIN__ -5` →
`const MIN: i32 = -5;`).

### `#define` with parameters is an error

A `#define` with a parameter list is a **parse error** directing you to
`#macro`:

```c
#define __MAX__(a, b) ...   // error: parameterised #define — use #macro
```

This split is deliberate: `#define` cannot take `(`, so its grammar is trivial
and unambiguous, and `#macro` carries all parameterised-macro behaviour.

## `#macro` — parameterised macros

A `#macro` is the function-like shim and always uses `(params)`. It lowers to a
Rust `macro_rules!` definition. The body is a **raw source span**: Crusty does
not tokenise or inspect it. Parameter references in the body are rewritten to
their `$name` form by a textual identifier substitution; everything else is
passed through for rustc to parse.

```c
#macro __MAX__(a, b) a > b ? a : b
```
Translates to:
```rust
macro_rules! max {
    ($a:expr, $b:expr) => {
        $a > $b ? $a : $b
    };
}
```

The macro name is lower-cased in the generated Rust (`__MAX__` → `max`); a name
that collides with a Rust keyword is suffixed with `_macro`.

Because the body is opaque to Crusty, Crusty cannot report Crusty-level errors
*inside* a macro body — any such error surfaces from rustc on the generated
Rust. This is consistent with Crusty being a syntax layer, not a semantic
reimplementation.

## `#ifdef` / `#ifndef` — conditional compilation

`#ifdef FOO` … `#endif` wraps the enclosed items and emits a bare `#[cfg(FOO)]`
attribute on each. `#ifndef FOO` emits `#[cfg(not(FOO))]`.

```c
#ifdef DEBUG
void log_state() { }
#endif
```
Translates to:
```rust
#[cfg(DEBUG)]
fn log_state() { }
```

Only bare cfg flags are shimmed. `#[cfg(feature = "…")]`, target predicates, and
other cfg forms are out of scope — write a raw attribute or use `__rust__` for
those.

## Invocation and delimiter preservation

A macro is invoked with the `__name__` form; the double-underscores are stripped
and a `!` is appended, producing the Rust call. The invocation delimiter
(`()`, `[]`, or `{}`) is **preserved**, matching Rust conventions such as
`vec![]` and `quote!{}`. The argument text is an opaque span handed to rustc.

| Crusty invocation   | Rust output   |
|---------------------|---------------|
| `__println__("hi")` | `println!("hi")` |
| `__vec__[1, 2, 3]`  | `vec![1, 2, 3]`  |
| `__format__{...}`   | `format!{...}`   |

Built-in Rust macros are reached through their double-underscore equivalents:
`__println__`, `__vec__`, `__assert__`, `__format__`.

## Formal Grammar

```ebnf
define_item   = "#define" "__" IDENT "__" const_value ;
const_value   = [ "-" ] literal ;

macro_item    = "#macro" "__" IDENT "__" "(" [ param_list ] ")" macro_body ;
macro_body    = raw_span ;

conditional   = ( "#ifdef" | "#ifndef" ) IDENT item* "#endif" ;

macro_invoke  = "__" IDENT "__" ( "(" args ")" | "[" args "]" | "{" args "}" ) ;
```

Note that a definition (`#macro`) only ever uses `(params)`, while an invocation
accepts any of the three delimiters.
