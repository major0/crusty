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
primary    = literal | IDENT | call | field_access | index | "(" expr ")" ;
```
