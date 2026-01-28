# Expressions

## Introduction

Crusty supports C-style expressions including arithmetic, logical, comparison, and bitwise operators. Operator precedence follows C conventions. Special expressions include type casts, sizeof, and the ternary operator.

## Rationale

Preserving C expression syntax and precedence rules ensures that developers can write expressions naturally without learning new conventions. The transpiler maps these to equivalent Rust expressions.

## Examples

### Arithmetic
```c
int result = a + b * c;
```

### Comparison and Logical
```c
if (x > 0 && y < 100) {
    // ...
}
```

### Ternary Operator
```c
int max = (a > b) ? a : b;
```
Translates to:
```rust
let max = if a > b { a } else { b };
```

### Type Cast
```c
float f = (float)integer_value;
```
Translates to:
```rust
let f = integer_value as f64;
```

### Sizeof
```c
int size = sizeof(int);
```
Translates to:
```rust
let size = std::mem::size_of::<i32>();
```

### Type-Scoped Static Calls
```c
let v = @Vector->new();
let none = @Option->None;
let s = @String->from("hello");
```
Translates to:
```rust
let v = Vector::new();
let none = Option::None;
let s = String::from("hello");
```

The `@` prefix with `->` notation distinguishes type-scoped (static) calls from instance method calls. Instance methods use dot notation as usual: `v.len()`, `v.get(0)`.

### Macro Invocations
```c
__println__!("Hello, world!");
__vec__![1, 2, 3];
__format__!("Value: {}", x);
```
Translates to:
```rust
println!("Hello, world!");
vec![1, 2, 3];
format!("Value: {}", x);
```

Crusty macros use double-underscore naming (`__name__!`) to visually distinguish them from regular function calls. The transpiler strips the underscores when generating Rust code.

## Formal Grammar

```ebnf
expr       = ternary_expr ;
ternary    = logical_or ("?" expr ":" expr)? ;
logical_or = logical_and ("||" logical_and)* ;
logical_and = equality ("&&" equality)* ;
equality   = comparison (("==" | "!=") comparison)* ;
comparison = addition (("<" | ">" | "<=" | ">=") addition)* ;
addition   = multiply (("+" | "-") multiply)* ;
multiply   = unary (("*" | "/" | "%") unary)* ;
unary      = ("!" | "-" | "&" | "*" | "++" | "--") unary | primary ;
primary    = literal | IDENT | call | field_access | index
           | type_scoped_call | macro_call | "(" expr ")" ;
type_scoped_call = "@" IDENT "->" IDENT ["(" [args] ")"] ;
macro_call = "__" IDENT "__" "!" ("(" args ")" | "[" args "]" | "{" args "}") ;
```
