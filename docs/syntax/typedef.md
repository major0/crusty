# Typedef, Type Aliases, and Implementation Blocks

## Introduction

Crusty uses C-style `typedef` syntax for three purposes: defining struct types, creating type aliases, and adding implementations to existing types. This provides a familiar syntax for C developers while mapping to Rust's type system.

## Rationale

In Rust, methods are added to types via `impl` blocks, which is unfamiliar to C developers. Crusty repurposes the `typedef` keyword to serve double duty: defining new types (including aliases) and adding implementations to existing types. The `@` prefix distinguishes between type definitions and implementation blocks.

## Type Aliases

Simple type aliases map directly to Rust's `type` keyword:

```c
typedef int MyInt;
typedef float Coordinate;
typedef int* IntPtr;
```

Translates to:
```rust
pub type MyInt = i32;
pub type Coordinate = f64;
pub type IntPtr = *mut i32;
```

Type aliases are fully resolved during semantic analysis — `MyInt` is interchangeable with `int` in all contexts. Circular aliases (e.g., `typedef A B; typedef B A;`) are detected and rejected at compile time.

Use `static typedef` for private (non-public) aliases:
```c
static typedef int InternalId;
```

## Examples

### Basic Struct Definition
```c
typedef struct {
    int x;
    int y;
} Point;
```
Translates to:
```rust
struct Point {
    x: i32,
    y: i32,
}
```

### Implementation Block
```c
typedef struct {
    Point new(int x, int y) {
        return Point { x: x, y: y };
    }

    int distance_squared(&self) {
        return self.x * self.x + self.y * self.y;
    }
} @Point;
```
Translates to:
```rust
impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x: x, y: y }
    }

    pub fn distance_squared(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }
}
```

### Default Trait Implementation
```c
typedef default {
    Point default() {
        return Point { x: 0, y: 0 };
    }
} @Point;
```
Translates to:
```rust
impl Default for Point {
    fn default() -> Self {
        Point { x: 0, y: 0 }
    }
}
```

### Named Implementation Blocks
```c
typedef struct {
    void print(&self) {
        __println__("Point({}, {})", self.x, self.y);
    }
} @Point.display;
```
Named blocks are organizational — multiple `@Type.name` blocks for the same type are merged into a single `impl Type` block in the generated Rust code.

## Formal Grammar

```ebnf
typedef_stmt  = "typedef" typedef_body ;
typedef_body  = type_alias | struct_def | impl_block ;
type_alias    = type IDENT ";" ;
struct_def    = "struct" "{" member_list "}" IDENT ";" ;
impl_block    = typedef_kind "{" member_list "}" typedef_target ";" ;
typedef_kind  = "struct" | "default" ;
typedef_target = "@" IDENT               (* impl block for existing type *)
               | "@" IDENT "." IDENT     (* named impl block *) ;
member_list   = (field_decl | method_decl)* ;
```

## Syntax Rules

- `typedef struct { ... } Type;` — define a new struct type
- `typedef struct { methods } @Type;` — add an impl block for an existing type
- `typedef default { fn default() { ... } } @Type;` — implement the Default trait
- `typedef struct { methods } @Type.name;` — named impl block (for organization)
- The `@` prefix indicates the type already exists
- The `.name` suffix is optional, used for organizing multiple impl blocks
