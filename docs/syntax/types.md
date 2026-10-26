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

### C / POSIX Type Aliases
For compatibility with existing C source, the standard `<stdint.h>` / POSIX
integer names are recognised as aliases and lower to their Rust equivalents, so
basic C code transpiles with little or no modification:

| C / POSIX | Rust |
|-----------|------|
| `int8_t`, `int16_t`, `int32_t`, `int64_t` | `i8`, `i16`, `i32`, `i64` |
| `uint8_t`, `uint16_t`, `uint32_t`, `uint64_t` | `u8`, `u16`, `u32`, `u64` |
| `size_t`, `uintptr_t` | `usize` |
| `ssize_t`, `ptrdiff_t`, `intptr_t` | `isize` |
| `byte` | `u8` |

A `typedef` of the same name takes precedence over the built-in alias.

### Multi-word C Primitives
The multi-word C integer spellings are recognised and lowered with the **LP64**
width model (the Linux/macOS convention):

| C spelling | Rust |
|------------|------|
| `signed char` | `i8` |
| `unsigned char` | `u8` |
| `short`, `short int`, `signed short [int]` | `i16` |
| `unsigned short [int]` | `u16` |
| `signed`, `signed int` | `i32` |
| `unsigned`, `unsigned int` | `u32` |
| `long`, `long int`, `long long [int]`, `signed long [long] [int]` | `i64` |
| `unsigned long [int]`, `unsigned long long [int]` | `u64` |

Caveats:
- **`long` is 64-bit here** (LP64). Code that relies on LLP64 (Windows, where
  `long` is 32-bit) should use a fixed-width name (`int32`) or `int32_t`.
- **Bare `char` stays Rust `char`** (a Unicode scalar), not an 8-bit integer.
  For a byte, use `signed char` / `unsigned char` (→ `i8`/`u8`) or `byte`.
- `long double` is not supported (Rust has no equivalent wide float).

### Structs

A struct has one of three shapes (ADR-0024), distinguished by what follows the
name. A braced body is the named-field form; `( … )` is a tuple struct with
positional, unnamed fields; and a bare `;` is a unit struct with no fields.

```c
struct Point {          // named-field struct
    int x;
    int y;
}

struct Pair(int, int);  // tuple struct      -> struct Pair(i32, i32);
struct Meters(f64);     // newtype           -> struct Meters(f64);
struct Marker;          // unit struct       -> struct Marker;
```

A tuple struct's fields are accessed positionally (`p.0`, `p.1`) and constructed
by calling the type name (`Point(3, 4)`); a unit struct is constructed by its
bare name (`Marker`). Both lower to the exact Rust form and inherit its
semantics (ADR-0001).

### Enums

A variant may be a plain name, a C-style discriminant (`= n`), a tuple payload
`(T, …)`, or a struct payload `{ T name; … }` (ADR-0023). A single variant
carries exactly one of these — a discriminant and a payload are mutually
exclusive (Rust forbids combining them).

```c
enum Color {            // C-style discriminant enum
    Red = 0,
    Green = 1,
    Blue = 2,
}

enum Shape {            // data-carrying enum
    Unit,                   // unit variant      -> Unit,
    Code = 7,               // discriminant      -> Code = 7,
    Circle(f64),            // tuple variant     -> Circle(f64),
    Rect { f64 w; f64 h; }, // struct variant    -> Rect { w: f64, h: f64 },
}
```

A bare variant (`Unit`) emits as a bare Rust variant — Rust auto-assigns it the
next sequential discriminant, so `as int` casts keep C semantics, and it stays
valid in an enum that also has payload variants. Only an explicit `= n` emits a
discriminant. Variants are matched with `match` and constructed with the
`@Type.Variant` form — tuple: `@Shape.Circle(2.0)`, unit: `@Shape.Unit`, and
struct-payload: `@Shape.Rect { w: 3, h: 4 }` (functional update `..base`
supported). All lower to the exact Rust form (`Shape::Rect { w: 3, h: 4 }`, etc.;
ADR-0001).

### Pointers and References
```c
&int x;       // immutable reference
var &int x;   // mutable reference
```

### Trait Objects and `impl Trait`

`dyn <Trait>` (a trait object) and `impl <Trait>` (an opaque type) are valid in
any type position and compose with the other type operators (ADR-0027):

```c
Box<dyn Error> boxed();            // trait object in a generic argument
void log(&dyn Display sink);       // trait object behind a reference
impl Iterator<Item = i32> seq();   // impl Trait return
```

Crusty does not model traits as types — the trait spec after `dyn` / `impl` is
carried **verbatim** to rustc (the same escape-hatch as generic bounds and
`where` clauses). Write the spec in Rust spelling (`i32`, not `int`), because it
is passed through unchanged; `dyn Fn(i32) -> i32` and `impl A + B` are supported.
`dyn` / `impl` are keywords only at the head of a type, so identifiers like
`dynamic` are unaffected.

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
