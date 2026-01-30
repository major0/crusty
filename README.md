# Crusty Programming Language

[![CI](https://github.com/major0/crusty/workflows/CI/badge.svg)](https://github.com/major0/crusty/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Crusty is a C-like programming language that transpiles to Rust, providing familiar C syntax while guaranteeing Rust's safety and performance. The Crusty transpiler enables bidirectional translation between Crusty and Rust source code, allowing seamless integration with the Rust ecosystem.

## Philosophy

**Crusty is primarily a syntax layer over Rust, with selective semantic enhancements.**

As a general rule, Crusty provides syntax changes over Rust. However, a few C-like semantic constructs are brought over for familiarity, where they can map cleanly to Rust's semantics. Crusty is **C-like**, not C itself.

### What Crusty Is

- A **syntax bridge** for developers comfortable with C-style syntax
- A **learning tool** to help understand Rust through familiar syntax
- A **1:1 compatible** layer that maintains full Rust semantics
- A way to write Rust code with C-like appearance

### What Crusty Is NOT

- **Not a C compiler** - Does not compile native C code
- **Not a C compatibility layer** - No standard C library functions
- **Not a semantic transformation** - Rust semantics are preserved
- **Not a replacement for Rust** - It's an alternative syntax for Rust

### Unsupported C Features

Crusty does NOT support the following C features, as they are incompatible with Rust's safety guarantees:

- **unions** - Use Rust enum types for tagged unions instead
- **goto statements** - Use structured control flow (loops, match, early returns)
- **Unsafe pointer arithmetic** - Use Rust's slice indexing or iterators
- **#include directives** - Use `#use` for Rust's module system
- **Preprocessor conditionals** - Use Rust's `cfg` attributes

### Syntax Stability Warning

⚠️ **IMPORTANT**: Crusty syntax is currently **unstable and evolving**. This is an experimental project in active development.

- Breaking changes may occur in any release
- Code written for one version may not compile in future versions
- Production use is not recommended at this time
- Use at your own risk for experimentation and learning

### Design Principles

Crusty transforms only syntax, not semantics. Method names, function names, and identifiers pass through unchanged between Crusty and Rust. This ensures:

- **Transparent transpilation**: You know exactly what Rust code will be generated
- **Bidirectional conversion**: True Crusty ↔ Rust round-trip transpilation
- **No conflicts**: Won't hijack your user-defined functions
- **Simplicity**: Less magic, fewer surprises

### C-Like Semantics (Selective)

Several C-like semantic constructs are supported. These are semantic transformations because they introduce control flow, scoping, or compile-time behavior that requires scaffolding:

- **NULL**: `NULL` → `Option::None` (maps C keyword to Rust's Option type)
- **C-style for loops**: `for(int i = 0; i < 100; i++)` → Rust loop with variable scoping
- **switch/case**: C-style switch statements → Rust match expressions with different semantics
- **#define**: C-style preprocessor macros → Rust declarative macros (different macro systems)

### Pure Syntax Transformations

- `Type?` → `Result<Type, Box<dyn std::error::Error>>` (fallible types)
- `expr?` → `expr?` (error propagation - pass through)
- `@Type.method()` → `Type::method()` (type-scoped calls with @ prefix)
- `.label:` → `'label:` (loop labels with dot prefix)
- `__macro__()` → `macro!()` (double-underscore for macros)

**Note:** The `@` prefix and double-underscore syntax are Crusty innovations in the C spirit, maintaining 1:1 Rust compatibility.

### What is NOT Transformed (Pass Through)

- Method names: `.is_err()`, `.is_ok()`, `.unwrap()` (unchanged)
- Function names: `Ok()`, `Err()` (unchanged)
- User-defined identifiers (unchanged)

### Semantic Transformations

Crusty includes several semantic transformations from C to Rust. These go beyond simple syntax mapping because they require understanding control flow, scoping, and compile-time behavior:

**NULL Handling** - Maps C keyword to Rust's Option type:
```c
void* ptr = NULL;        // → let ptr: Option<&()> = Option::None;
if (ptr == NULL) { }     // → if ptr.is_none() { }
if (ptr != NULL) { }     // → if ptr.is_some() { }
```

**C-Style For Loops** - Introduces variable scoping and control flow scaffolding:
```c
for (int i = 0; i < 100; i++) {  // → Rust loop with scoped variable
    // i is scoped to loop body
}
```

**Switch Statements** - Maps C fall-through semantics to Rust exhaustive matching:
```c
switch (x) {                     // → Rust match expression
    case 1: break;
    case 2: break;
    default: break;
}
```

**See**: [SYNTAX_PHILOSOPHY.md](.kiro/specs/crusty-compiler-phase1/SYNTAX_PHILOSOPHY.md) for detailed rationale.

## Features

### C-like Syntax with Rust Safety
- **Familiar C syntax**: Write code with C-style function declarations, control flow, and data structures
- **Type-scoped calls**: Use `@Type` prefix with dot notation: `@Vec.new()`, `@Option.None`
- **Macro system**: Double-underscore naming for macros: `__println__("Hello")`, `__vec__[1, 2, 3]`
- **Closures**: Nested functions that capture outer scope (transpile to Rust closures)
- **Escape hatch**: Use `__rust__` to embed raw Rust code for advanced features
- **Rust compatibility**: All Crusty code transpiles to safe, idiomatic Rust

### Rust Ecosystem Integration
- **Use Rust crates**: Import and use any existing Rust crate or module
- **Publish crates**: Compile Crusty code into crates that native Rust projects can depend on
- **Bidirectional transpilation**: Convert between Crusty and Rust syntax as needed
- **Build system integration**: Works seamlessly with Cargo through build.rs scripts

### Safety Guarantees
- **Memory safety**: Rust's ownership and borrowing model prevents memory errors
- **Type safety**: Strong static typing catches errors at compile time
- **No null pointers**: Uses Rust's Option type for nullable values
- **No data races**: Rust's concurrency model prevents data races

## Quick Start

### Installation

#### Prerequisites
- Rust toolchain (stable) - [Install Rust](https://rustup.rs/)
- Cargo package manager (included with Rust)

#### Building from Source
```bash
git clone https://github.com/major0/crusty.git
cd crusty
cargo build --release
cargo install --path .
```

### Your First Crusty Program

Create `hello.crst`:
```c
void main() {
    __println__("Hello, Crusty!");
}
```

Transpile and run:
```bash
crustyc hello.crst --emit=binary -o hello
./hello
```

## Syntax Examples

### Functions and Types
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

### Structs and Methods
```c
// Define a struct type
typedef struct {
    int x;
    int y;
} Point;

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

// Implement Default trait
typedef default {
    Point default() {
        return Point { x: 0, y: 0 };
    }
} @Point;

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

### Control Flow
```c
int fibonacci(int n) {
    if (n <= 1) {
        return n;
    }
    return fibonacci(n - 1) + fibonacci(n - 2);
}

void count_to_ten() {
    for (int i = 0; i < 10; i++) {
        __println__("{}", i);
    }
}
```

### Error Handling with Type?
```c
// Fallible return type: Type? → Result<Type, Box<dyn std::error::Error>>
int? parse_number(char* str) {
    // Error propagation: expr? → expr? (pass through)
    let num = str.parse()?;  // Propagates error if parse fails
    return Ok(num);          // Use Rust's Ok() directly (not transformed)
}

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

**Note**: Only `Type?` is transformed to `Result<Type, E>`. The `expr?` operator and method names (`.is_err()`, `.is_ok()`, `.unwrap()`) pass through unchanged to Rust. This preserves transparency and avoids conflicts with user-defined functions.

### NULL Handling (Special Case)
```c
// NULL is the ONLY semantic transformation in Crusty
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

**Note**: NULL is the ONLY exception to Crusty's syntax-only philosophy. It's a C keyword with no direct Rust equivalent, so it requires special handling to map to Rust's `Option` type.

### Macros and Type-Scoped Calls
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

### Advanced Syntax

#### Module Imports with #use
```c
// Import Rust standard library modules
// Dot notation in module paths (no @ prefix for imports)
#use std.collections.HashMap;
#use std.io.Write;

void main() {
    // Type-scoped call with @ prefix uses dot notation
    let map = @HashMap.new();
    map.insert("key", "value");
}
```

#### Explicit Generic Type Parameters
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

#### Defining Macros with #define
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

#### Labeled Loops
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

#### Embedding Raw Rust Code with __rust__
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

**Note**: The `__rust__` macro provides an escape hatch for using Rust features not yet supported by Crusty syntax. The contents are passed directly to rustc without validation by crustyc. Use this when you need access to advanced Rust features like pattern matching, closures, or complex trait bounds.

#### Closures with Nested Functions
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
    
    // Nested functions can be passed as function parameters
    void apply_twice(int (*func)(int), int value) {
        return func(func(value));
    }
    
    int double_it(int x) {
        return x * 2;
    }
    
    let doubled = apply_twice(double_it, 5);  // Returns 20
    
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

**Translation to Rust**: Nested functions are translated to Rust closures:
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

**Scoping Rules**:
- Nested functions can only capture variables defined **before** the nested function declaration
- Variables defined **after** a nested function are not accessible to that function
- Multiple nested functions can capture and share the same outer variables
- Captures can be immutable (read-only) or mutable (read-write)

**Note**: Nested functions provide a familiar C-style syntax for closures. They can capture variables from the enclosing scope and are translated to Rust closures (`Fn`, `FnMut`, or `FnOnce` depending on how they use captured variables).

**Reference**: GNU C supports nested functions as an extension: https://gcc.gnu.org/onlinedocs/gcc/Nested-Functions.html

#### Implementation Blocks with typedef
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

## Usage

### Basic Transpilation

Transpile Crusty to Rust:
```bash
crustyc input.crst -o output.rs
```

Transpile and compile to binary:
```bash
crustyc input.crst --emit=binary -o program
```

Transpile Rust to Crusty:
```bash
crustyc input.rs --from-lang=rust -o output.crst
```

### Command-Line Options

```
crustyc [OPTIONS] <INPUT>

OPTIONS:
    -o, --output <FILE>         Output file path
    --emit <MODE>               Output mode: rust, binary, ast
    --from-lang <LANG>          Source language: crusty, rust
    -v, --verbose               Detailed output
    --no-compile                Generate Rust without invoking rustc
    -h, --help                  Print help information
    --version                   Print version information
```

## Build Integration

### Using Crusty in Cargo Projects

Crusty integrates seamlessly with Cargo through build.rs scripts.

**1. Add crustyc as a build dependency in `Cargo.toml`:**
```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"

[build-dependencies]
# crustyc = "0.1"  # When published to crates.io
```

**2. Create `build.rs` in your project root:**
```rust
use std::process::Command;
use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    
    // Transpile all .crst files to Rust
    Command::new("crustyc")
        .args(&["src/", "--out-dir", &out_dir])
        .status()
        .expect("Failed to run crustyc");
    
    // Rebuild if any .crst file changes
    println!("cargo:rerun-if-changed=src/");
}
```

**3. Place your `.crst` files in `src/`:**
```
my-project/
├── Cargo.toml
├── build.rs
└── src/
    ├── main.crst
    ├── lib.crst
    └── utils.crst
```

**4. Build normally with Cargo:**
```bash
cargo build
cargo run
```

The build.rs script automatically transpiles your Crusty code to Rust during the build process.

### Example Project

The repository includes a complete working example demonstrating Crusty language features and build system integration. See the [example/](example/) directory for:
- [example/Cargo.toml](example/Cargo.toml) - Project configuration
- [example/build.rs](example/build.rs) - Build script that transpiles .crst files
- [example/src/](example/src/) - Sample Crusty programs
- [example/README.md](example/README.md) - Build and run instructions

The example demonstrates:
- Function declarations and control flow
- Struct definitions with methods
- Type-scoped static method calls (`@Type.method()`)
- Macro usage with double-underscore naming (`__println__`, `__vec__`)
- Build system integration with Cargo

To run the example:
```bash
cd example
cargo build
cargo run
```

The example is automatically built and tested in the CI/CD pipeline to ensure the transpiler works correctly.

## Development

### Running Tests
```bash
cargo test
```

### Code Formatting
```bash
cargo fmt
```

### Linting
```bash
cargo clippy
```

### Pre-commit Hooks

Install pre-commit hooks for automatic code quality checks:

```bash
pip install pre-commit
pre-commit install
```

The hooks will automatically run:
- Crusty syntax validation on `.crst` files
- Rust formatting checks on `.rs` files
- Clippy linting on `.rs` files

## Documentation

### Getting Started
- [Quick Start](#quick-start) - Install and run your first Crusty program
- [Syntax Examples](#syntax-examples) - Learn Crusty syntax with examples
- [Example Programs](example/README.md) - Working examples with build integration

### Core Concepts
- [Philosophy](#philosophy) - Understand syntax-only transpilation
- [SYNTAX_PHILOSOPHY.md](.kiro/specs/crusty-compiler-phase1/SYNTAX_PHILOSOPHY.md) - Detailed rationale and design principles
- [Error Handling](#error-handling-with-type) - Using Type? and expr? operator
- [NULL Handling](#null-handling-special-case) - The ONLY semantic exception

### Integration
- [Build System Integration](#build-integration) - Using Crusty with Cargo
- [Build.rs Guide](docs/build-rs-integration.md) - Comprehensive build integration guide
- [Command-Line Usage](#usage) - crustyc command-line options

### Contributing
- [Contributing Guide](CONTRIBUTING.md) - How to contribute to the project
- [Development Workflow](#development) - Running tests, formatting, linting

### Specification Documents
- [Requirements](.kiro/specs/crusty-compiler-phase1/requirements.md) - Detailed requirements and acceptance criteria
- [Design](.kiro/specs/crusty-compiler-phase1/design.md) - Architecture and component design
- [Implementation Tasks](.kiro/specs/crusty-compiler-phase1/tasks.md) - Development task breakdown and progress

### Language Reference
- Function declarations and definitions
- Struct and enum types with typedef syntax
- Implementation blocks (typedef struct @Type)
- Trait implementations (typedef default @Type)
- Type-scoped calls with dot notation (`@Type.method()`)
- Macro invocations with double-underscore naming (`__macro_name__`)
- Raw Rust code embedding with `__rust__` escape hatch
- Closures with nested functions
- Control flow statements
- Memory management and ownership
- Module system and visibility

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Workflow
1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Ensure all tests pass: `cargo test`
5. Format code: `cargo fmt`
6. Check for warnings: `cargo clippy`
7. Commit using [Conventional Commits](https://www.conventionalcommits.org/) format
8. Submit a pull request

### Commit Message Format
```
type(scope): subject

body

footer
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `chore`

Example:
```
feat(parser): add support for labeled loops

Implemented parsing for labeled loops with .label: syntax.
Translates to Rust's 'label: syntax.

Validates: Requirements 6.13, 6.14, 6.15
```

## License

This project is licensed under the MIT License - see the [LICENSE.txt](LICENSE.txt) file for details.

## Project Status

**Phase 1: Core Transpiler** - In Active Development

Current progress:
- ✅ Infrastructure (CI/CD, pre-commit hooks, licensing)
- ✅ Core transpiler (lexer, parser, AST, semantic analysis)
- ✅ Code generation (Crusty → Rust)
- ✅ Advanced parsing (structs, methods, generics, macros)
- ✅ Example directory with working samples ([see example/](example/))
- ✅ Build system integration (build.rs support)
- 🚧 Bidirectional transpilation (Rust → Crusty)
- 🚧 Documentation generator (crustydoc)

See [tasks.md](/.kiro/specs/crusty-compiler-phase1/tasks.md) for detailed implementation progress.

## Roadmap

### Phase 1 (Current)
- Core transpiler infrastructure
- Crusty → Rust transpilation
- Basic language features
- Build system integration

### Phase 2 (Planned)
- Enhanced macro system
- Generic function definitions
- Trait definitions with ergonomic syntax
- Pattern matching syntax
- Async/await support

### Phase 3 (Future)
- IDE integration (LSP support)
- Debugger integration
- Procedural macros
- Advanced optimization passes

## Community

- **Issues**: [GitHub Issues](https://github.com/major0/crusty/issues)
- **Discussions**: [GitHub Discussions](https://github.com/major0/crusty/discussions)
- **Repository**: [github.com/major0/crusty](https://github.com/major0/crusty)

## Acknowledgments

Crusty builds on the excellent work of:
- The Rust programming language and its ecosystem
- The Rust compiler (rustc) for code generation
- The syn crate for Rust parsing
- The proptest crate for property-based testing
