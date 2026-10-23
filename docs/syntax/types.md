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
| `int8`, `int16`, `int32`, `int64`, `int128` | `i8`, `i16`, `i32`, `i64`, `i128` |
| `uint8`, `uint16`, `uint32`, `uint64`, `uint128` | `u8`, `u16`, `u32`, `u64`, `u128` |
| `float` | `f64` |
| `f32`, `f64` | `f32`, `f64` |
| `bool` | `bool` |
| `char` | `char` |
| `void` | `()` |

`int` and `float` are C-friendly aliases for `i32` and `f64`. The fixed-width
integer types use the C-traditional `int<size>` / `uint<size>` spelling
(`int8`…`int128`, `uint8`…`uint128`) and lower to Rust's `i8`…`i128` /
`u8`…`u128`. They may be used anywhere a type is expected — including function
parameters and return types.

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
int[10] arr;           // array of 10 ints
(int, float) pair;     // tuple
```

## Formal Grammar

```ebnf
type_expr  = primitive_type | struct_type | enum_type | pointer_type
           | reference_type | array_type | tuple_type | generic_type ;
primitive  = "int" | "int8" | "int16" | "int32" | "int64" | "int128"
           | "uint8" | "uint16" | "uint32" | "uint64" | "uint128"
           | "float" | "f32" | "f64" | "bool" | "char" | "void" ;
```
