# Macros and #define

## Introduction

Crusty supports C-style `#define` macros that translate to Rust `macro_rules!` definitions. Macro names use double-underscore naming (`__name__`) without the `!` suffix used in Rust. Macros have explicit delimiter types determined at definition time.

## Rationale

C developers are familiar with `#define` for macros. Crusty preserves this syntax while generating proper Rust `macro_rules!` definitions. The double-underscore naming convention distinguishes macros from regular functions and avoids the Rust-specific `!` suffix.

## Examples

### Simple Constant Macro
```c
#define __PI__ 3.14159
let area = __PI__ * r * r;
```
Translates to:
```rust
macro_rules! pi { () => { 3.14159 }; }
let area = pi!() * r * r;
```

### Parameterized Macro
```c
#define __MAX__(a, b) ((a) > (b) ? (a) : (b))
let m = __MAX__(x, y);
```
Translates to:
```rust
macro_rules! max { ($a:expr, $b:expr) => { if $a > $b { $a } else { $b } }; }
let m = max!(x, y);
```

### Bracket Delimiters
```c
#define __VEC__[items] items
let v = __VEC__[1, 2, 3];
```
Translates to:
```rust
let v = vec![1, 2, 3];
```

## Delimiter Types

Macros have explicit delimiter types determined at the `#define` declaration:

| Delimiter | Crusty Syntax | Rust Output |
|-----------|--------------|-------------|
| None | `__MACRO__` | `macro!()` |
| Parens | `__MACRO__(args)` | `macro!(args)` |
| Brackets | `__MACRO__[args]` | `macro![args]` |
| Braces | `__MACRO__{args}` | `macro!{args}` |

## Naming Rules

- Macro names must use double-underscore prefix and suffix: `__name__`
- The double-underscores are stripped during translation to Rust
- Rust keywords cannot be used as macro names
- Built-in Rust macros are accessed via their double-underscore equivalents: `__println__`, `__vec__`, `__assert__`, `__format__`

## Formal Grammar

```ebnf
define_stmt   = "#define" "__" IDENT "__" [delimiter_params] macro_body ;
delimiter_params = "(" param_list ")" | "[" param_list "]" | "{" param_list "}" ;
macro_invoke  = "__" IDENT "__" [delimiter_args] ;
delimiter_args  = "(" args ")" | "[" args "]" | "{" args "}" ;
```
