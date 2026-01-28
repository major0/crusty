# Crusty Programming Language

[![CI](https://github.com/major0/crusty/workflows/CI/badge.svg)](https://github.com/major0/crusty/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Crusty is a C-like programming language that transpiles to Rust, providing familiar C syntax while guaranteeing Rust's safety and performance. The Crusty transpiler enables bidirectional translation between Crusty and Rust source code, allowing seamless integration with the Rust ecosystem.

## Features

### C-like Syntax with Rust Safety
- **Familiar C syntax**: Write code with C-style function declarations, control flow, and data structures
- **Type-scoped calls**: Use `@Type` prefix with dot notation: `@Vec.new()`, `@Option.None`
- **Macro system**: Double-underscore naming for macros: `__println__("Hello")`, `__vec__[1, 2, 3]`
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
```crusty
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
```crusty
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
```crusty
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
```crusty
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

### Macros and Type-Scoped Calls
```crusty
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
```crusty
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
```crusty
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
```crusty
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
```crusty
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
```crusty
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

#### Implementation Blocks with typedef
```crusty
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

### Specification Documents
- [Requirements](/.kiro/specs/crusty-compiler-phase1/requirements.md) - Detailed requirements and acceptance criteria
- [Design](/.kiro/specs/crusty-compiler-phase1/design.md) - Architecture and component design
- [Implementation Tasks](/.kiro/specs/crusty-compiler-phase1/tasks.md) - Development task breakdown and progress

### Language Reference
- Function declarations and definitions
- Struct and enum types with typedef syntax
- Implementation blocks (typedef struct @Type)
- Trait implementations (typedef default @Type)
- Type-scoped calls with dot notation (`@Type.method()`)
- Macro invocations with double-underscore naming (`__macro_name__`)
- Raw Rust code embedding with `__rust__` escape hatch
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
