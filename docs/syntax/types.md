# Types

## Introduction

Crusty supports a rich type system that maps to Rust types. Primitive types use C-style names where possible, and compound types follow familiar C conventions for structs, enums, pointers, and references.

## Rationale

Using C-style type names (e.g., `int` instead of `i32`) reduces the learning curve for C developers. The transpiler handles the mapping to Rust's type system, including safety checks for pointer usage.

## Examples

### Primitive Types
| Crusty | Rust |
|--------|------|
| `int` | `i32` |
| `i32`, `i64` | `i32`, `i64` |
| `u32`, `u64` | `u32`, `u64` |
| `float` | `f64` |
| `f32`, `f64` | `f32`, `f64` |
| `bool` | `bool` |
| `char` | `char` |
| `void` | `()` |

### Structs
```c
struct Point {
    int x;
    int y;
}
```

### Enums
```c
enum Color {
    Red = 0,
    Green = 1,
    Blue = 2,
}
```

### Pointers and References
```c
&int x;       // immutable reference
var &int x;   // mutable reference
```

### Arrays and Tuples
```c
int arr[10];           // array of 10 ints
(int, float) pair;     // tuple
```

## Formal Grammar

```ebnf
type_expr  = primitive_type | struct_type | enum_type | pointer_type
           | reference_type | array_type | tuple_type | generic_type ;
primitive  = "int" | "i32" | "i64" | "u32" | "u64" | "float"
           | "f32" | "f64" | "bool" | "char" | "void" ;
```
