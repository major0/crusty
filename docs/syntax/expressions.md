# Expressions

## Introduction

Crusty supports C-like expressions including arithmetic, logical, and comparison operators.

## Rationale

Standard C operators provide familiar syntax for common operations.

## Examples

### Arithmetic Operators

```crusty
int a = 10 + 5;    // addition
int b = 10 - 5;    // subtraction
int c = 10 * 5;    // multiplication
int d = 10 / 5;    // division
int e = 10 % 3;    // modulo
```

### Comparison Operators

```crusty
bool eq = a == b;  // equal
bool ne = a != b;  // not equal
bool lt = a < b;   // less than
bool gt = a > b;   // greater than
bool le = a <= b;  // less or equal
bool ge = a >= b;  // greater or equal
```

### Logical Operators

```crusty
bool and = a && b; // logical and
bool or = a || b;  // logical or
bool not = !a;     // logical not
```

### Bitwise Operators

```crusty
int band = a & b;  // bitwise and
int bor = a | b;   // bitwise or
int bxor = a ^ b;  // bitwise xor
int bnot = ~a;     // bitwise not
int shl = a << 2;  // shift left
int shr = a >> 2;  // shift right
```

### Assignment Operators

```crusty
a = 10;            // assignment
a += 5;            // add and assign
a -= 5;            // subtract and assign
a *= 2;            // multiply and assign
a /= 2;            // divide and assign
```

### Ternary Operator

```crusty
int max = (a > b) ? a : b;
```

### Type Cast

```crusty
float f = (float)42;
int i = (int)3.14;
```

### Field Access

```crusty
Point p = { .x = 10, .y = 20 };
int x = p.x;
```

### Method Call

```crusty
str.len();
vec.push(42);
```

### Type-Scoped Calls

Type-scoped calls (static methods/associated functions) use the `@` prefix:

```crusty
// Type-scoped calls - ALWAYS require @ prefix
let v = @Vec->new();
let none = @Option->None;
let s = @String->from("hello");

// With explicit generic parameters
let opt = @Option(i32)->None;
let v = @Vec(i32)->new();
```

**Arrow vs Dot Notation**:

After the `@` prefix, you can use either:
- **Arrow notation (`->`)**: For simple type-scoped calls
- **Dot notation (`.`)**: For nested type paths (matching Rust's `::` followed by `.`)

```crusty
// Arrow notation for simple calls
@Vec->new()
@Option->None

// Dot notation for nested type paths
@Foo.Bar.boo()                    // → Foo::Bar.boo()
@std.collections.HashMap->new()   // → std::collections::HashMap::new()
```

This syntax makes it immediately clear whether a call is:
- **Type-scoped** (`@Type->method()` or `@Type.method()`): Calling a static method on the type itself
- **Instance-scoped** (`obj.method()`): Calling a method on an instance

Translation to Rust:
- `@Vec->new()` → `Vec::new()`
- `@Option->None` → `Option::None`
- `@String->from("hello")` → `String::from("hello")`
- `@Foo.Bar.boo()` → `Foo::Bar.boo()`
- `@std.collections.HashMap->new()` → `std::collections::HashMap::new()`

## Formal Grammar

```ebnf
expression  ::= assignment_expr
assignment_expr ::= ternary_expr (assign_op ternary_expr)?
ternary_expr ::= logical_or_expr ('?' expression ':' ternary_expr)?
logical_or_expr ::= logical_and_expr ('||' logical_and_expr)*
logical_and_expr ::= equality_expr ('&&' equality_expr)*
equality_expr ::= relational_expr (('==' | '!=') relational_expr)*
relational_expr ::= additive_expr (('<' | '>' | '<=' | '>=') additive_expr)*
additive_expr ::= multiplicative_expr (('+' | '-') multiplicative_expr)*
multiplicative_expr ::= unary_expr (('*' | '/' | '%') unary_expr)*
unary_expr  ::= ('!' | '-' | '~' | '*' | '&') unary_expr | postfix_expr
postfix_expr ::= primary_expr (call_expr | index_expr | field_expr)*
```
