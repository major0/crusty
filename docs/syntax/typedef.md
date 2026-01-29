# Typedef and Implementation Blocks

## Introduction

Crusty uses C-style `typedef` syntax for defining structs and their implementations. This provides a familiar syntax for C programmers while mapping cleanly to Rust's `impl` blocks.

## Rationale

The typedef syntax was chosen because:
- Familiar to C programmers
- Clear distinction between type definition and implementation
- Supports trait implementations with minimal new syntax
- Maps cleanly to Rust's `impl` and `impl Trait for Type` patterns

## Examples

### Basic Struct Definition

```crusty
// Define a struct type
typedef struct {
    int x;
    int y;
} Point;
```

Translates to Rust:
```rust
struct Point {
    x: i32,
    y: i32,
}
```

### Implementation Blocks (@Type)

Use the `@` prefix to add methods to an existing type:

```crusty
// Add methods to an existing type
typedef struct {
    Point new(int x, int y) {
        return Point { x: x, y: y };
    }
    
    int distance_squared(&self) {
        return self.x * self.x + self.y * self.y;
    }
} @Point;
```

Translates to Rust:
```rust
impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        return Point { x: x, y: y };
    }
    
    pub fn distance_squared(&self) -> i32 {
        return self.x * self.x + self.y * self.y;
    }
}
```

### Default Trait Implementation

Use `typedef default` to implement the Default trait:

```crusty
// Implement Default trait
typedef default {
    Point default() {
        return Point { x: 0, y: 0 };
    }
} @Point;
```

Translates to Rust:
```rust
impl Default for Point {
    fn default() -> Self {
        return Point { x: 0, y: 0 };
    }
}
```

### Named Implementation Blocks (@Type.name)

Use named impl blocks to organize methods:

```crusty
// Named impl block for display methods
typedef struct {
    void print(&self) {
        __println__("Point({}, {})", self.x, self.y);
    }
} @Point.display;

// Another named impl block for debug methods
typedef struct {
    void debug(&self) {
        __println__("Point {{ x: {}, y: {} }}", self.x, self.y);
    }
} @Point.debug;
```

Both translate to Rust (merged into single impl):
```rust
impl Point {
    pub fn print(&self) {
        println!("Point({}, {})", self.x, self.y);
    }
    
    pub fn debug(&self) {
        println!("Point {{ x: {}, y: {} }}", self.x, self.y);
    }
}
```

## Syntax Rules

| Syntax | Meaning |
|--------|---------|
| `typedef struct { ... } Type;` | Define a new struct type |
| `typedef struct { methods } @Type;` | Add impl block for existing type |
| `typedef default { fn default() { ... } } @Type;` | Implement Default trait |
| `typedef struct { methods } @Type.name;` | Named impl block (for organization) |

Key points:
- The `@` prefix indicates the type already exists
- The `.name` suffix is optional and used for organizing multiple impl blocks
- All named impl blocks for the same type are merged in the generated Rust code

## Translation Rules

| Crusty | Rust |
|--------|------|
| `typedef struct @Type` | `impl Type` |
| `typedef default @Type` | `impl Default for Type` |
| `typedef struct @Type.name` | `impl Type` (name is for organization only) |

## Formal Grammar

```ebnf
typedef_stmt    ::= 'typedef' typedef_kind '{' typedef_body '}' typedef_target ';'
typedef_kind    ::= 'struct' | 'default'
typedef_body    ::= field_list | method_list
typedef_target  ::= identifier | '@' identifier impl_name?
impl_name       ::= '.' identifier
```
