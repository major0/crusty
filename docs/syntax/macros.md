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
| `#ifdef FOO` … `#else` … `#endif` | `#[cfg(FOO)]` / `#[cfg(not(FOO))]` | Conditional with else branch |
| `#ifdef A` … `#elif B` … `#endif` | `#[cfg(A)]` / `#[cfg(all(not(A), B))]` | Condition chain (cascade) |
| `name!(args)`                 | `name!(args)`              | Macro invocation (bang form)  |

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
`const MIN: i32 = -(5);`).

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
    ($a:expr, $b:expr) => {{
        $a > $b ? $a : $b
    }};
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

### `#else`

An optional `#else` branch selects an alternative set of items. Because Rust has
no block-level preprocessor, `#else` does not become a Rust `else` — it lowers to
a **second cfg-gated group under the inverted predicate**. The `#else` of an
`#ifdef FOO` is gated `#[cfg(not(FOO))]`; the `#else` of an `#ifndef FOO` is gated
`#[cfg(FOO)]`.

```c
#ifdef FOO
void greet() { println!("FOO is set"); }
#else
void greet() { println!("FOO is not set"); }
#endif
```
Translates to:
```rust
#[cfg(FOO)]
fn greet() { println!("FOO is set"); }
#[cfg(not(FOO))]
fn greet() { println!("FOO is not set"); }
```

Both branches may define the same name — they are mutually exclusive at compile
time, so exactly one is active and there is no real duplicate. The condition is
written once; the `#else` predicate is always its inverse.

### `#elif` / `#elifdef` / `#elifndef` — condition chains

A block may carry any number of `#elif` arms between the opener and the optional
`#else`. Crusty accepts the standard C `#ifdef` family:

| Directive            | Meaning                                  |
|----------------------|------------------------------------------|
| `#elif NAME`         | positive defined-test (same as `#elifdef`) |
| `#elifdef NAME`      | positive defined-test (standard-C spelling) |
| `#elifndef NAME`     | negated defined-test                     |

`#if <expr>` / `#elif <expr>` (general constant expressions) are **not**
supported — the family keys off a single `defined`-style NAME, because a Rust
`cfg` predicate is not an expression evaluator. Arithmetic conditions stay in a
`__rust__{}` block.

Because Rust has no `cfg`-level else-if, a chain lowers to **independent
per-item `#[cfg]` groups**: each arm conjoins its own predicate with the
negation of every *prior* arm, so exactly one arm is active for any cfg
combination.

```c
#ifdef A
void which() { println!("A"); }
#elif B
void which() { println!("B"); }
#elifndef C
void which() { println!("not C"); }
#else
void which() { println!("else"); }
#endif
```
Translates to:
```rust
#[cfg(A)]                           fn which() { println!("A"); }
#[cfg(all(not(A), B))]              fn which() { println!("B"); }
#[cfg(all(not(A), not(B), not(C)))] fn which() { println!("not C"); }
#[cfg(all(not(A), not(B), C))]      fn which() { println!("else"); }
```

As with `#else`, every arm may define the same name — the groups are mutually
exclusive, so there is no real duplicate.

## Invocation and delimiter preservation

A macro is invoked with Rust's native `name!(...)` form — the trailing `!` is
required. The invocation delimiter (`()`, `[]`, or `{}`) is **preserved**,
matching Rust conventions such as `vec![]` and `quote!{}`. The argument text is
an opaque span handed to rustc.

| Crusty invocation  | Rust output      |
|--------------------|------------------|
| `println!("hi")`   | `println!("hi")` |
| `vec![1, 2, 3]`    | `vec![1, 2, 3]`  |
| `format!{...}`     | `format!{...}`   |

The `!` is not decoration — it is what makes a macro call unambiguous. A bare
`name(...)` is a function call; `name!(...)` is a macro call. Without the `!`,
an invocation would be lexically identical to a function call, and the only
thing telling them apart would be a naming convention the grammar cannot
enforce — opening the door to vague resolution rules and silent shadowing (a
local binding named the same as a macro). Requiring the `!` keeps the two
grammatically distinct, exactly as Rust does, and is the thin-syntax-layer
choice: a Rust macro call passes through unchanged. (Macros are *defined* with a
sigil — `#define`, `#macro __NAME__` — because a declaration site has no such
ambiguity; only the call site does.)

Built-in Rust macros are called by their Rust names: `println!`, `vec!`,
`assert!`, `format!`.

## Formal Grammar

```ebnf
define_item   = "#define" "__" IDENT "__" const_value ;
const_value   = [ "-" ] literal ;

macro_item    = "#macro" "__" IDENT "__" "(" [ param_list ] ")" macro_body ;
macro_body    = raw_span ;

conditional   = ( "#ifdef" | "#ifndef" ) IDENT item*
                ( ( "#elif" | "#elifdef" | "#elifndef" ) IDENT item* )*
                ( "#else" item* )? "#endif" ;

macro_invoke  = IDENT "!" ( "(" args ")" | "[" args "]" | "{" args "}" ) ;
```

Note that a definition (`#macro`) only ever uses `(params)`, while an invocation
uses the `!` bang form and accepts any of the three delimiters.
