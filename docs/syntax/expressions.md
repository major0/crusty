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
let max: i32 = if a > b { a } else { b };
```

### Prefix Increment/Decrement
```c
++i;    // Increment before use
--i;    // Decrement before use
```
Translates to:
```rust
{ i += 1; i }
{ i -= 1; i }
```

### Postfix Increment/Decrement
```c
i++;    // Increment after use (yields the old value)
i--;    // Decrement after use (yields the old value)
```
Translates to a block expression that captures the prior value, then mutates:
```rust
{ let __old = i; i += 1; __old }
{ let __old = i; i -= 1; __old }
```

Both prefix (`++i`, `--i`) and postfix (`i++`, `i--`) forms are supported. Prefix
yields the new value; postfix yields the value from before the mutation, matching
C semantics.

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
let v = @Vector.new();
let none = @Option.None;
let s = @String.from("hello");
```
Translates to:
```rust
let v = Vector::new();
let none = Option::None;
let s = String::from("hello");
```

The `@` prefix distinguishes type-scoped (static) calls from instance method
calls. A single-segment `@Type.method(args)` call and the no-parens
associated-access `@Type.MEMBER` form (for a unit enum variant or an associated
constant) are supported:
```c
@Vector.new()       // Vector::new()
@Option.None        // Option::None  (no-parens associated access)
@String.from("hi")  // String::from("hi")
```

**Planned (not yet implemented):** multi-segment paths and the arrow form are
documented for direction but do not parse today — use a `__rust__ { … }` block
for these until they land:
```c
@std.collections.HashMap.new()   // PLANNED → std::collections::HashMap::new()
@Foo.Bar.boo()                   // PLANNED → Foo::Bar::boo()
@Foo.BAR->boo()                  // PLANNED → Foo::BAR.boo() (method on a constant)
```

Instance methods use dot notation as usual: `v.len()`, `v.get(0)`.

### Macro Invocations
```c
println!("Hello, world!");
vec![1, 2, 3];
format!("Value: {}", x);
```
Translates to:
```rust
println!("Hello, world!");
vec![1, 2, 3];
format!("Value: {}", x);
```

Macros are invoked with Rust's native `name!(...)` bang form. The `!` keeps a
macro call grammatically distinct from a function call (`name(...)`), so an
invocation can never be confused with a call or shadowed by a same-named
binding. Macros are *defined* with a sigil (`#define`, `#macro`); see
[macros](macros.md).

### Comma Operator
```c
for (int i = 1, j = 2; i < 100; i++, j += 2) {
    // ...
}
```

The comma operator has the lowest precedence and is left-associative. It evaluates its left operand, discards the result, then evaluates and returns its right operand. This is particularly important in for-loop initializers and increments where multiple expressions need to be evaluated in sequence.

## Formal Grammar

```ebnf
expr       = comma_expr ;
comma_expr = ternary_expr ("," ternary_expr)* ;
ternary    = logical_or ("?" expr ":" expr)? ;
logical_or = logical_and ("||" logical_and)* ;
logical_and = equality ("&&" equality)* ;
equality   = comparison (("==" | "!=") comparison)* ;
comparison = addition (("<" | ">" | "<=" | ">=") addition)* ;
addition   = multiply (("+" | "-") multiply)* ;
multiply   = unary (("*" | "/" | "%") unary)* ;
unary      = ("!" | "-" | "&" | "*" | "++" | "--") unary | postfix ;
postfix    = primary ("++" | "--" | "." IDENT | "." IDENT "(" [args] ")"
                       | "[" expr "]" | "(" [args] ")")* ;
primary    = literal | IDENT | call | field_access | index
           | type_scoped_call | macro_call | "(" expr ")" ;
type_scoped_call = "@" IDENT "." IDENT "(" [args] ")" ;
macro_call = IDENT "!" ("(" args ")" | "[" args "]" | "{" args "}") ;
```

> Note: `type_scoped_call` accepts the single-segment `@Type.method(args)` form,
> and `@Type.MEMBER` (no-parens associated access) is supported separately
> (PS-15). Multi-segment paths (`@a.b.c.m()`) and the arrow form
> (`@Type.CONST->method()`) are **Planned** and do not parse yet.
