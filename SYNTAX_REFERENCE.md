# Crusty Syntax Reference

Complete reference for Crusty syntax, transformations, and language features.

## Table of Contents

1. [Functions and Types](#functions-and-types)
2. [Type Aliases](#type-aliases)
3. [Structs and Methods](#structs-and-methods)
4. [Control Flow](#control-flow)
5. [Error Handling](#error-handling)
6. [NULL Handling](#null-handling)
7. [Macros and Type-Scoped Calls](#macros-and-type-scoped-calls)
8. [Module System](#module-system)
9. [Generic Type Parameters](#generic-type-parameters)
10. [Defining Macros](#defining-macros)
11. [Labeled Loops](#labeled-loops)
12. [Raw Rust Code](#raw-rust-code)
13. [Closures with Nested Functions](#closures-with-nested-functions)
14. [Implementation Blocks](#implementation-blocks)

---

## Functions and Types

### Basic Function Declarations

```c
// C-style function declarations
int add(int a, int b) {
    return a + b;
}

// Void return type
void print_sum(int x, int y) {
    __println__("Sum: {}", add(x, y));
}

// Static functions (private in Rust)
static int helper(int n) {
    return n * 2;
}
```

**Translates to Rust:**
```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn print_sum(x: i32, y: i32) {
    println!("Sum: {}", add(x, y));
}

fn helper(n: i32) -> i32 {
    n * 2
}
```

---

## Type Aliases

Crusty supports general type aliases using `typedef`, allowing you to create alternative names for any type.

### Simple Type Aliases

```c
// Create aliases for primitive types
typedef int MyInt;
typedef float MyFloat;
typedef bool Flag;

// Use type aliases with C-like casting
void example() {
    let x = (MyInt)42;        // Cast to MyInt type
    let y = (int)x;           // Cast back to int
    let z = (MyFloat)3.14;
}
```

**Translates to Rust:**
```rust
pub type MyInt = i32;
pub type MyFloat = f64;
pub type Flag = bool;

pub fn example() {
    let x = 42 as MyInt;
    let y = x as i32;
    let z = 3.14 as MyFloat;
}
```

### Pointer and Reference Type Aliases

```c
// Pointer type aliases (prefix * syntax)
typedef *int IntPtr;
typedef *char CharPtr;

// Reference type aliases
// Immutable references (implicit or explicit let)
typedef &int IntRef;           // Implicit let (default)
typedef let &int IntRefAlt;    // Explicit let (equivalent)

// Mutable references (explicit var)
typedef var &int MutIntRef;

void pointer_example() {
    let value = 100;
    let ptr = (IntPtr)&value;
    let ref = (IntRef)&value;
}
```

**Translates to Rust:**
```rust
pub type IntPtr = *mut i32;
pub type CharPtr = *mut i8;
pub type IntRef = &i32;
pub type MutIntRef = &mut i32;

pub fn pointer_example() {
    let value = 100;
    let ptr = &value as IntPtr;
    let ref_val = &value as IntRef;
}
```

### Custom Type Aliases

```c
// Alias for struct types
struct Point {
    int x;
    int y;
}

typedef Point PointAlias;
typedef Point* PointPtr;

void custom_type_example() {
    let p = (PointAlias)Point { x: 10, y: 20 };
    let ptr = (PointPtr)&p;
}
```

**Translates to Rust:**
```rust
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub type PointAlias = Point;
pub type PointPtr = *mut Point;

pub fn custom_type_example() {
    let p = Point { x: 10, y: 20 } as PointAlias;
    let ptr = &p as PointPtr;
}
```

### Chained Type Aliases

```c
// Type aliases can reference other aliases
typedef int Integer;
typedef Integer Number;
typedef Number Count;

// All three are compatible with int through casting
void chained_example() {
    let a = 1;
    let b = (Integer)a;
    let c = (Number)b;
    let d = (Count)c;
}
```

**Translates to Rust:**
```rust
pub type Integer = i32;
pub type Number = Integer;
pub type Count = Number;

pub fn chained_example() {
    let a = 1;
    let b = a as Integer;
    let c = b as Number;
    let d = c as Count;
}
```

### Generic Type Aliases

```c
// Aliases for generic types
typedef Vec[int] IntVec;
typedef HashMap[String, int] StringIntMap;

void generic_example() {
    let numbers = (IntVec)@Vec(int).new();
    let map = (StringIntMap)@HashMap(String, int).new();
}
```

**Translates to Rust:**
```rust
pub type IntVec = Vec<i32>;
pub type StringIntMap = HashMap<String, i32>;

pub fn generic_example() {
    let numbers = Vec::<i32>::new() as IntVec;
    let map = HashMap::<String, i32>::new() as StringIntMap;
}
```

### Type Alias Best Practices

- **Use meaningful names**: `UserId` instead of `MyInt`
- **Document purpose**: Add comments explaining why the alias exists
- **Avoid circular references**: `typedef A B; typedef B A;` will cause an error
- **Keep it simple**: Don't create unnecessarily long alias chains

---

## Structs and Methods

### Struct Definition

```c
// Define a struct type
typedef struct {
    int x;
    int y;
} Point;
```

**Translates to Rust:**
```rust
pub struct Point {
    pub x: i32,
    pub y: i32,
}
```

### Implementation Blocks

```c
// Add implementation block with methods
typedef struct {
    // Static method (constructor)
    Point new(int x, int y) {
        return Point { x: x, y: y };
    }
    
    // Instance method
    int distance_squared(&self) {
        return self.x * self.x + self.y * self.y;
    }
} @Point;
```

**Translates to Rust:**
```rust
impl Point {
    pub fn new(x: i32, y: i32) -> Point {
        Point { x, y }
    }
    
    pub fn distance_squared(&self) -> i32 {
        self.x * self.x + self.y * self.y
    }
}
```

### Trait Implementations

```c
// Implement Default trait
typedef default {
    Point default() {
        return Point { x: 0, y: 0 };
    }
} @Point;
```

**Translates to Rust:**
```rust
impl Default for Point {
    fn default() -> Point {
        Point { x: 0, y: 0 }
    }
}
```

### Using Structs

```c
void main() {
    // Type-scoped call with @ prefix and dot notation
    // Dot (.) replaces Rust's :: for type-scoped access
    let p1 = @Point.new(3, 4);
    
    // Use Default trait
    let origin = @Point.default();
    
    // Instance method call (no @ prefix)
    __println__("Distance²: {}", p1.distance_squared());
    
    // Nested type paths: dot replaces :: for type-scoped access
    // @std.collections.HashMap.new()
    // Translates to: std::collections::HashMap::new()
    
    // Method calls on type-scoped values use arrow
    // @Foo.BAR->boo()  where BAR is a constant, boo() is a method
    // Translates to: Foo::BAR.boo()
}
```

---

## Control Flow

### If Statements

```c
int fibonacci(int n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}
```

### For Loops

```c
void count_to_ten() {
    for (int i = 0; i < 10; i++) {
        __println__("{}", i);
    }
}
```

**Translates to Rust:**
```rust
pub fn count_to_ten() {
    for i in 0..10 {
        println!("{}", i);
    }
}
```

### While Loops

```c
void countdown(int n) {
    while (n > 0) {
        __println__("{}", n);
        n = n - 1;
    }
}
```

### Loop Statement

```c
void infinite_loop() {
    loop {
        __println__("Forever!");
        break;
    }
}
```

---

## Error Handling

### Fallible Return Types

```c
// Fallible return type: Type? → Result<Type, Box<dyn std::error::Error>>
int? parse_number(char* str) {
    // Error propagation: expr? → expr? (pass through)
    let num = str.parse()?;  // Propagates error if parse fails
    return Ok(num);          // Use Rust's Ok() directly (not transformed)
}
```

**Translates to Rust:**
```rust
pub fn parse_number(str: &str) -> Result<i32, Box<dyn std::error::Error>> {
    let num = str.parse()?;
    Ok(num)
}
```

### Using Result Types

```c
void main() {
    let result = parse_number("42");
    
    // Use Rust's Result API directly (method names NOT transformed)
    if (result.is_err()) {              // NOT .is_error()
        __println__("Parse failed");
        return;
    }
    
    let value = result.unwrap();        // Pass through unchanged
    __println__("Parsed: {}", value);
}
```

**Important Notes:**
- Only `Type?` is transformed to `Result<Type, E>`
- The `expr?` operator passes through unchanged to Rust
- Method names (`.is_err()`, `.is_ok()`, `.unwrap()`) pass through unchanged
- This preserves transparency and avoids conflicts with user-defined functions

---

## NULL Handling

NULL is the **ONLY semantic transformation** in Crusty. It's a C keyword with no direct Rust equivalent, so it requires special handling to map to Rust's `Option` type.

### NULL Syntax

```c
// NULL → Option::None
void process_optional(int* ptr) {
    // NULL → Option::None
    if (ptr == NULL) {              // → if ptr.is_none()
        __println__("No value");
        return;
    }
    
    if (ptr != NULL) {              // → if ptr.is_some()
        __println__("Has value");
    }
}

void main() {
    int* ptr = NULL;                // → let ptr: Option<&i32> = Option::None;
    process_optional(ptr);
}
```

**Translates to Rust:**
```rust
pub fn process_optional(ptr: Option<&i32>) {
    if ptr.is_none() {
        println!("No value");
        return;
    }
    
    if ptr.is_some() {
        println!("Has value");
    }
}

pub fn main() {
    let ptr: Option<&i32> = Option::None;
    process_optional(ptr);
}
```

---

## Macros and Type-Scoped Calls

### Macro Invocations

```c
void main() {
    // Macros use double-underscore naming (no ! suffix in Crusty)
    __println__("Creating a vector...");
    
    // Type-scoped calls use @ prefix with dot notation
    let v = @Vec.new();
    v.push(1);
    v.push(2);
    v.push(3);
    
    // Macro with formatting
    __println__("Vector: {:?}", v);
}
```

**Translates to Rust:**
```rust
pub fn main() {
    println!("Creating a vector...");
    
    let v = Vec::new();
    v.push(1);
    v.push(2);
    v.push(3);
    
    println!("Vector: {:?}", v);
}
```

**Syntax Rules:**
- Macros use double-underscore naming: `__macro_name__()`
- No `!` suffix in Crusty (added during transpilation)
- Type-scoped calls use `@` prefix: `@Type.method()`
- Dot notation replaces `::` for type-scoped access

---

## Module System

### Importing Modules

```c
// Import modules into current context (private)
// Translates to: use std::collections::HashMap;
#import std.collections.HashMap;

// Import entire module
// Translates to: use std::fs;
#import std.fs;

void main() {
    // Type-scoped call with @ prefix uses dot notation
    let map = @HashMap.new();
    map.insert("key", "value");
}
```

### Exporting Symbols

```c
// Export symbols for public API (public re-export)
// Translates to: pub use std::io::Write;
#export std.io.Write;
```

**Import vs Export:**
- `#import module;` → `use module;` (private import into current context)
- `#export module.symbol;` → `pub use module::symbol;` (public re-export for API)
- Use `#import` to bring modules/symbols into scope for internal use
- Use `#export` to make imported symbols part of your public API

---

## Generic Type Parameters

### Explicit Type Parameters

```c
void main() {
    // Explicit type parameters with parentheses/brackets syntax
    let v = @Vec(i32).new();
    v.push(42);
    
    // Nested generics alternate parentheses and brackets
    // Dot notation for type-scoped access
    let opt = @Option(Result[String, std.io.Error]).None;
    
    // Type inference when parameters omitted
    let v2 = @Vec.new();  // Type inferred from usage
}
```

**Translates to Rust:**
```rust
pub fn main() {
    let v = Vec::<i32>::new();
    v.push(42);
    
    let opt = Option::<Result<String, std::io::Error>>::None;
    
    let v2 = Vec::new();
}
```

**Syntax Rules:**
- Use parentheses `()` for first level of generics
- Use brackets `[]` for nested generics
- Alternate between `()` and `[]` for deeper nesting
- Type inference works when parameters are omitted

---

## Defining Macros

### Macro Definitions

```c
// Define macros with double-underscore naming
#define __MAX__(a, b) ((a) > (b) ? (a) : (b))
#define __SQUARE__(x) ((x) * (x))

void main() {
    let max_val = __MAX__(10, 20);
    let squared = __SQUARE__(5);
    __println__("Max: {}, Squared: {}", max_val, squared);
}
```

**Translates to Rust:**
```rust
macro_rules! MAX {
    ($a:expr, $b:expr) => {
        if $a > $b { $a } else { $b }
    };
}

macro_rules! SQUARE {
    ($x:expr) => {
        $x * $x
    };
}

pub fn main() {
    let max_val = MAX!(10, 20);
    let squared = SQUARE!(5);
    println!("Max: {}, Squared: {}", max_val, squared);
}
```

**Syntax Rules:**
- Macro names must start and end with double underscores: `__NAME__`
- Translates to Rust's `macro_rules!` declarative macros
- Invocations use double-underscore syntax without `!`

---

## Labeled Loops

### Loop Labels

```c
void main() {
    // Labels use dot prefix (. is not part of the label name)
    .outer: loop {
        .inner: loop {
            if (condition) {
                break outer;  // Break to outer loop (no dot in break)
            }
            continue inner;  // Continue inner loop (no dot in continue)
        }
    }
}
```

**Translates to Rust:**
```rust
pub fn main() {
    'outer: loop {
        'inner: loop {
            if condition {
                break 'outer;
            }
            continue 'inner;
        }
    }
}
```

**Syntax Rules:**
- Labels use dot prefix: `.label:`
- Break/continue reference labels without dot: `break label;`
- Translates to Rust's lifetime-style labels: `'label:`

---

## Raw Rust Code

### Using __rust__ Escape Hatch

The `__rust__` macro provides an escape hatch for using Rust features not yet supported by Crusty syntax.

```c
void main() {
    // Use __rust__ as an escape hatch for Rust-specific features
    // The contents are passed directly to the Rust compiler
    
    // In expression context
    let result = __rust__{ Some(42) };
    
    // In statement context
    __rust__{
        println!("This is raw Rust code");
        let x = vec![1, 2, 3];
    };
    
    // For complex Rust patterns not yet supported in Crusty
    __rust__{
        match value {
            Some(x) if x > 10 => println!("Large: {}", x),
            Some(x) => println!("Small: {}", x),
            None => println!("Nothing"),
        }
    };
    
    // In type context (for complex Rust types)
    let callback: __rust__{ Box<dyn Fn(i32) -> i32> } = __rust__{ Box::new(|x| x * 2) };
}
```

**Important Notes:**
- Contents are passed directly to rustc without validation by crustyc
- Use when you need access to advanced Rust features
- Useful for pattern matching, closures, or complex trait bounds
- No syntax checking by Crusty compiler

---

## Closures with Nested Functions

Crusty supports nested functions as closures, providing a familiar C-style syntax.

### Basic Nested Functions

```c
void main() {
    // Crusty supports nested functions as closures
    // Functions defined within functions can capture variables from outer scope
    
    int outer_value = 42;
    
    // Define a nested function that captures outer scope
    // Can only capture variables defined BEFORE the nested function
    int add_to_outer(int x) {
        return x + outer_value;  // Captures outer_value (defined above)
    }
    
    // Use the nested function
    let result = add_to_outer(10);  // Returns 52
    __println__("Result: {}", result);
    
    // Variables defined after the nested function are NOT accessible
    int later_value = 100;  // add_to_outer cannot access this
}
```

**Translates to Rust:**
```rust
pub fn main() {
    let outer_value = 42;
    
    // Becomes a closure
    let add_to_outer = |x: i32| -> i32 {
        x + outer_value
    };
    
    let result = add_to_outer(10);
    println!("Result: {}", result);
}
```

### Passing Nested Functions as Parameters

```c
void main() {
    // Nested functions can be passed as function parameters
    void apply_twice(int (*func)(int), int value) {
        return func(func(value));
    }
    
    int double_it(int x) {
        return x * 2;
    }
    
    let doubled = apply_twice(double_it, 5);  // Returns 20
}
```

### Mutable Captures

```c
void main() {
    // Mutable captures work too
    int counter = 0;
    
    void increment() {
        counter = counter + 1;  // Mutably captures counter
    }
    
    increment();
    increment();
    __println__("Counter: {}", counter);  // Prints 2
    
    // Multiple nested functions can capture the same variables
    void reset() {
        counter = 0;
    }
    
    reset();
    __println__("Counter after reset: {}", counter);  // Prints 0
}
```

### Scoping Rules

**Important:**
- Nested functions can only capture variables defined **before** the nested function declaration
- Variables defined **after** a nested function are not accessible to that function
- Multiple nested functions can capture and share the same outer variables
- Captures can be immutable (read-only) or mutable (read-write)

**Translation:**
- Nested functions are translated to Rust closures
- Closure type depends on usage: `Fn`, `FnMut`, or `FnOnce`

**Reference:** GNU C supports nested functions as an extension: https://gcc.gnu.org/onlinedocs/gcc/Nested-Functions.html

---

## Implementation Blocks

### Struct with Multiple Implementation Blocks

```c
// Define a struct type
typedef struct {
    int width;
    int height;
} Rectangle;

// Add implementation block
typedef struct {
    Rectangle new(int w, int h) {
        return Rectangle { width: w, height: h };
    }
    
    int area(&self) {
        return self.width * self.height;
    }
} @Rectangle;

// Implement Default trait
typedef default {
    Rectangle default() {
        return Rectangle { width: 0, height: 0 };
    }
} @Rectangle;

// Named implementation block (for organization)
typedef struct {
    void print(&self) {
        __println__("Rectangle: {}x{}", self.width, self.height);
    }
} @Rectangle.display;

void main() {
    // Type-scoped call with @ prefix and dot notation
    let rect = @Rectangle.new(10, 20);
    __println__("Area: {}", rect.area());
    rect.print();
}
```

**Translates to Rust:**
```rust
pub struct Rectangle {
    pub width: i32,
    pub height: i32,
}

impl Rectangle {
    pub fn new(w: i32, h: i32) -> Rectangle {
        Rectangle { width: w, height: h }
    }
    
    pub fn area(&self) -> i32 {
        self.width * self.height
    }
}

impl Default for Rectangle {
    fn default() -> Rectangle {
        Rectangle { width: 0, height: 0 }
    }
}

impl Rectangle {
    pub fn print(&self) {
        println!("Rectangle: {}x{}", self.width, self.height);
    }
}

pub fn main() {
    let rect = Rectangle::new(10, 20);
    println!("Area: {}", rect.area());
    rect.print();
}
```

---

## Syntax Transformation Summary

### Pure Syntax Transformations

| Crusty | Rust | Description |
|--------|------|-------------|
| `Type?` | `Result<Type, Box<dyn std::error::Error>>` | Fallible types |
| `expr?` | `expr?` | Error propagation (pass through) |
| `@Type.method()` | `Type::method()` | Type-scoped calls |
| `.label:` | `'label:` | Loop labels |
| `__macro__()` | `macro!()` | Macro invocations |
| `#import module;` | `use module;` | Module imports |
| `#export module;` | `pub use module;` | Public re-exports |

### Semantic Transformations

| Crusty | Rust | Description |
|--------|------|-------------|
| `NULL` | `Option::None` | NULL keyword mapping |
| `ptr == NULL` | `ptr.is_none()` | NULL comparison |
| `ptr != NULL` | `ptr.is_some()` | Non-NULL comparison |
| `for(int i=0; i<n; i++)` | `for i in 0..n` | C-style for loops |
| `switch/case` | `match` | Switch statements |
| `#define __M__()` | `macro_rules! M` | Macro definitions |

### Pass Through (Unchanged)

- Method names: `.is_err()`, `.is_ok()`, `.unwrap()`
- Function names: `Ok()`, `Err()`
- User-defined identifiers
- Rust operators and expressions

---

## Additional Resources

- **[README.md](README.md)** - Project overview and philosophy
- **[QUICK_START.md](QUICK_START.md)** - Installation and getting started
- **[SYNTAX_PHILOSOPHY.md](.kiro/specs/crusty-compiler-phase1/SYNTAX_PHILOSOPHY.md)** - Detailed design rationale
- **[Example Programs](example/)** - Working code examples
- **[Build Integration Guide](docs/build-rs-integration.md)** - Cargo integration

---

**Questions or feedback?** Open an issue at [GitHub Issues](https://github.com/major0/crusty/issues) or start a discussion at [GitHub Discussions](https://github.com/major0/crusty/discussions).
