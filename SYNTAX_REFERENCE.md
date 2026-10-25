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
    println!("Sum: {}", add(x, y));
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

// Use type aliases with type inference
void example() {
    let x = 42;        // Type inferred as int
    let y = x;         // Type inferred from x
    let z = 3.14;      // Type inferred as float
}
```

**Translates to Rust:**
```rust
pub type MyInt = i32;
pub type MyFloat = f64;
pub type Flag = bool;

pub fn example() {
    let x = 42;
    let y = x;
    let z = 3.14;
}
```

**Note:** C-style variable declarations with explicit types (e.g., `MyInt x = 42;`) are supported and lower to a typed `let` (`let x: MyInt = 42;`). Type inference with `let` or `var` also works.

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
    // Pointer and reference usage with type inference
    let ptr = &value;
    let ref = &value;
}
```

**Translates to Rust:**
```rust
pub type IntPtr = *mut i32;
pub type CharPtr = *mut char;
pub type IntRef = &i32;
pub type MutIntRef = &mut i32;

pub fn pointer_example() {
    let value = 100;
    let ptr = &value;
    let ref_val = &value;
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
    // Use struct initializer with type inference
    let p = Point { x: 10, y: 20 };
    let ptr = &p;
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
    let p = Point { x: 10, y: 20 };
    let ptr = &p;
}
```

### Chained Type Aliases

```c
// Type aliases can reference other aliases
typedef int Integer;
typedef Integer Number;
typedef Number Count;

// All three are compatible through type inference
void chained_example() {
    let a = 1;
    let b = a;
    let c = b;
    let d = c;
}
```

**Translates to Rust:**
```rust
pub type Integer = i32;
pub type Number = Integer;
pub type Count = Number;

pub fn chained_example() {
    let a = 1;
    let b = a;
    let c = b;
    let d = c;
}
```

### Generic Type Aliases

```c
// Aliases for generic types
typedef Vec[int] IntVec;
typedef HashMap[String, int] StringIntMap;

void generic_example() {
    // Use type-scoped calls with type inference
    let numbers = @Vec(int).new();
    let map = @HashMap(String, int).new();
}
```

**Translates to Rust:**
```rust
pub type IntVec = Vec<i32>;
pub type StringIntMap = HashMap<String, i32>;

pub fn generic_example() {
    let numbers = Vec::<i32>::new();
    let map = HashMap::<String, i32>::new();
}
```

### Type Alias Best Practices

- **Use meaningful names**: `UserId` instead of `MyInt`
- **Document purpose**: Add comments explaining why the alias exists
- **Avoid circular references**: `typedef A B; typedef B A;` will cause an error
- **Keep it simple**: Don't create unnecessarily long alias chains

**Current Implementation Note:**
- Type aliases are fully supported with `typedef`
- Variable declarations may use type inference (`let x = 42;`) or an explicit C-style type (`int x = 42;` → `let x: i32 = 42;`)
- C-style declarations with explicit types (e.g., `MyInt x = 42;`) are supported
- Both forms are available; pick inference or an explicit type as you prefer

---

## Structs and Methods

### Struct Definition

```c
// Define a struct type
struct Point {
    int x;
    int y;
}
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
    println!("Distance²: {}", p1.distance_squared());
    
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
    for (int i = 0; i < 10; ++i) {
        println!("{}", i);
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
        println!("{}", n);
        n = n - 1;
    }
}
```

### Loop Statement

```c
void infinite_loop() {
    loop {
        println!("Forever!");
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
        println!("Parse failed");
        return;
    }
    
    let value = result.unwrap();        // Pass through unchanged
    println!("Parsed: {}", value);
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
        println!("No value");
        return;
    }
    
    if (ptr != NULL) {              // → if ptr.is_some()
        println!("Has value");
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
    // Macro invocations use Rust's bang form: `name!(...)`
    println!("Creating a vector...");
    
    // Type-scoped calls use @ prefix with dot notation
    let v = @Vec.new();
    v.push(1);
    v.push(2);
    v.push(3);
    
    // Macro with formatting
    println!("Vector: {:?}", v);
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
- Macro invocations use Rust's bang form: `name!(...)` (e.g. `println!("{}", x)`) — passed through unchanged
- Double-underscore naming (`__NAME__`) applies only to `#define`/`#macro` *definitions*, not to invocations
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

Crusty has two definition directives, both requiring a double-underscore `__NAME__` name:

- `#define __NAME__ <literal>` — a **constant** (a single literal value); lowers to a Rust `const`.
- `#macro __NAME__(params) body` — a **parameterised macro**; lowers to `macro_rules!`.

`#define` does **not** take a parameter list — use `#macro` for anything with parameters.

### Constant Definitions (`#define`)

```c
#define __PI__ 3
#define __GREETING__ "hello"

void main() {
    println!("{} {}", __PI__, __GREETING__);
}
```

**Translates to Rust:**
```rust
const PI: i32 = 3;
const GREETING: &str = "hello";

pub fn main() {
    println!("{} {}", PI, GREETING);
}
```

### Parameterised Macros (`#macro`)

```c
// Parameterised macros use #macro with double-underscore naming
#macro __MAX__(a, b) ((a) > (b) ? (a) : (b))
#macro __SQUARE__(x) ((x) * (x))

void main() {
    let max_val = max!(10, 20);
    let squared = square!(5);
    println!("Max: {}, Squared: {}", max_val, squared);
}
```

**Translates to Rust:**
```rust
macro_rules! max {
    ($a:expr, $b:expr) => {{
        if $a > $b { $a } else { $b }
    }};
}

macro_rules! square {
    ($x:expr) => {{
        $x * $x
    }};
}

pub fn main() {
    let max_val = max!(10, 20);
    let squared = square!(5);
    println!("Max: {}, Squared: {}", max_val, squared);
}
```

**Syntax Rules:**
- Definition names must start and end with double underscores: `__NAME__`; the generated Rust name is lower-cased with the underscores dropped (`__MAX__` → `max`)
- `#define` defines a literal constant; `#macro` defines a parameterised `macro_rules!` macro
- Invocations use Rust's bang form with the generated (lower-cased) name (`max!(10, 20)`), not the double-underscore form

---

## Labeled Loops

### Loop Labels

```c
void main() {
    // Labels use dot prefix (. is not part of the label name)
    .outer: loop {
        .inner: loop {
            if (condition) {
                break .outer;  // Break to outer loop (dot prefixes the label)
            }
            continue .inner;  // Continue inner loop (dot prefixes the label)
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
- Break/continue reference labels WITH the dot prefix: `break .label;` / `continue .label;`
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

Crusty has two nested-callable forms, split by Rust's own rule that hoisting and capture are mutually exclusive (see [docs/syntax/nested-functions.md](docs/syntax/nested-functions.md) for the full treatment):

- A **named nested function** lowers to a Rust **`fn` item** — hoisted (callable before its definition), recursive, and **non-capturing** (it cannot read enclosing locals; pass them as parameters, C-style).
- A **`__func__` anonymous function** lowers to a Rust **closure** — it **captures** enclosing locals but is not hoisted and cannot recurse.

### Named Nested Functions (hoisted `fn` items)

```c
void main() {
    // A named nested function is a hoisted `fn` item: recursive and
    // forward-referenceable, but it does NOT capture outer locals.
    let f = fact(5);          // callable before its definition (hoisted)
    println!("fact(5)={}", f);

    int fact(int n) {
        if (n <= 1) { return 1; }
        return n * fact(n - 1);   // recursion is fine
    }
}
```

**Translates to Rust:**
```rust
pub fn main() {
    let f = fact(5);
    println!("fact(5)={}", f);
    fn fact(n: i32) -> i32 {
        if n <= 1 { return 1; }
        return n * fact(n - 1);
    }
}
```

An outer value a nested function needs is passed as a parameter (a `fn` item cannot capture):

```c
void main() {
    int base = 10;
    int add_base(int x, int base) { return x + base; }   // base is a parameter
    let r = add_base(5, base);   // 15
    println!("{}", r);
}
```

### Capturing: `__func__` (closures)

When you need to capture an enclosing local, use a `__func__` anonymous function — it lowers to a Rust closure:

```c
void main() {
    int outer_value = 42;
    let add_to_outer = int __func__(int x) { return x + outer_value; };  // captures
    println!("{}", add_to_outer(10));   // 52
}
```

**Translates to Rust:**
```rust
pub fn main() {
    let outer_value = 42;
    let add_to_outer = |x: i32| -> i32 { return x + outer_value; };
    println!("{}", add_to_outer(10));
}
```

### Passing a function by pointer

A named function can be passed by a `fn(T) -> R` function-pointer parameter:

```c
int apply_twice(fn(int) -> int func, int value) {
    return func(func(value));
}
int double_it(int x) { return x * 2; }

void main() {
    let doubled = apply_twice(double_it, 5);   // 20
    println!("{}", doubled);
}
```

### Validation Rules

1. **No Multi-Level Nesting**: a nested function cannot contain another nested function (one level only).
   ```c
   void outer() {
       void middle() {
           void inner() { return; }  // ❌ ERROR: multi-level nesting not supported
       }
   }
   ```

2. **No Static Nested Functions**: a nested function cannot be `static` (the `NestedFunction` AST node has no `static` modifier).

3. **Type Compatibility**: assigning or passing a nested function enforces function-signature compatibility (parameter types + return type).

**Reference:** GNU C supports nested functions as an extension: https://gcc.gnu.org/onlinedocs/gcc/Nested-Functions.html

---

## Implementation Blocks

### Struct with Multiple Implementation Blocks

```c
// Define a struct type
struct Rectangle {
    int width;
    int height;
}

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
        println!("Rectangle: {}x{}", self.width, self.height);
    }
} @Rectangle.display;

void main() {
    // Type-scoped call with @ prefix and dot notation
    let rect = @Rectangle.new(10, 20);
    println!("Area: {}", rect.area());
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
| `name!(...)` | `name!(...)` | Macro invocations (bang form, pass through) |
| `Type name = v;` | `let name: Type = v;` | C-style typed declaration |
| `#import module;` | `use module;` | Module imports |
| `#export module;` | `pub use module;` | Public re-exports |

### Semantic Transformations

| Crusty | Rust | Description |
|--------|------|-------------|
| `NULL` | `Option::None` | NULL keyword mapping |
| `ptr == NULL` | `ptr.is_none()` | NULL comparison |
| `ptr != NULL` | `ptr.is_some()` | Non-NULL comparison |
| `for(int i=0; i<n; ++i)` | `for i in 0..n` | C-style for loops |
| `switch/case` | `match` | Switch statements |
| `#define __C__ lit` | `const C = lit;` | Constant definitions |
| `#macro __M__(a) body` | `macro_rules! M` | Parameterised macro definitions |

### Pass Through (Unchanged)

- Method names: `.is_err()`, `.is_ok()`, `.unwrap()`
- Function names: `Ok()`, `Err()`
- User-defined identifiers
- Rust operators and expressions

---

## Additional Resources

- **[README.md](README.md)** - Project overview and philosophy
- **[QUICK_START.md](QUICK_START.md)** - Installation and getting started
- **[docs/design/philosophy.md](docs/design/philosophy.md)** - Detailed design rationale
- **[Example Programs](example/)** - Working code examples
- **[Build Integration Guide](docs/build-rs-integration.md)** - Cargo integration

---

**Questions or feedback?** Open an issue at [GitHub Issues](https://github.com/major0/crusty/issues) or start a discussion at [GitHub Discussions](https://github.com/major0/crusty/discussions).
