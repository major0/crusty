# Types

## Introduction

Crusty provides C-like type syntax that maps to Rust's type system.

## Rationale

Familiar type names reduce cognitive load while providing access to Rust's rich type system.

## Examples

### Primitive Types

```crusty
int x = 42;        // i32
i64 big = 1000000; // i64
u32 unsigned = 10; // u32
float f = 3.14;    // f32
f64 precise = 3.14159265359; // f64
bool flag = true;  // bool
char c = 'a';      // char
```

### Pointer Types

```crusty
int* ptr;          // *const i32
int& ref;          // &i32
int& mut mref;     // &mut i32
```

### Array Types

```crusty
int[10] arr;       // [i32; 10]
int[] slice;       // &[i32]
```

### Struct Types

```crusty
struct Point {
    int x;
    int y;
}

Point p = { .x = 10, .y = 20 };
```

### Enum Types

```crusty
enum Color {
    Red,
    Green,
    Blue
}
```

### Type Aliases

```crusty
typedef int MyInt;
typedef Point* PointPtr;
```

## Formal Grammar

```ebnf
type        ::= primitive_type | identifier | pointer_type | reference_type | array_type
primitive_type ::= 'int' | 'i32' | 'i64' | 'u32' | 'u64' | 'float' | 'f32' | 'f64' | 'bool' | 'char' | 'void'
pointer_type ::= type '*'
reference_type ::= type '&' 'mut'?
array_type  ::= type '[' expression? ']'
```
