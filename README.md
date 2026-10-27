# Crusty Programming Language

[![CI](https://github.com/major0/crusty/workflows/CI/badge.svg)](https://github.com/major0/crusty/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Crusty is a C-like programming language that transpiles to Rust, providing familiar C syntax while guaranteeing Rust's safety and performance. Crusty code compiles into crates that native Rust projects can depend on, for seamless integration with the Rust ecosystem.

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
- **#include directives** - Use `#import` for Rust's module system

### Supported Conditional Compilation

Crusty supports the C `#ifdef` directive family, lowering each arm to Rust's `cfg` attributes:

- **#ifdef / #ifndef** - open a conditional block, testing whether a name is defined
- **#elif / #elifdef / #elifndef** - else-if arms chained after the opener
- **#else / #endif** - fallback arm and block terminator
- Each arm lowers to a per-item `#[cfg(all(not(prior…), this))]` attribute
- Bare `#if <expr>` is deliberately not supported — Crusty never evaluates conditions, only tests definedness

### Syntax Stability Warning

⚠️ **IMPORTANT**: Crusty syntax is currently **unstable and evolving**. This is an experimental project in active development.

- Breaking changes may occur in any release
- Code written for one version may not compile in future versions
- Production use is not recommended at this time
- Use at your own risk for experimentation and learning

### Design Principles

Crusty transforms only syntax, not semantics. Method names, function names, and identifiers pass through unchanged from Crusty to Rust. This ensures:

- **Transparent transpilation**: You know exactly what Rust code will be generated
- **No conflicts**: Won't hijack your user-defined functions
- **Simplicity**: Less magic, fewer surprises

### C-Like Semantics (Selective)

Several C-like semantic constructs are supported. These are semantic transformations because they introduce control flow, scoping, or compile-time behavior that requires scaffolding:

- **NULL**: `NULL` → `Option::None` (maps C keyword to Rust's Option type)
- **C-style for loops**: `for(int i = 0; i < 100; ++i)` → Rust loop with variable scoping
- **switch/case**: C-style switch statements → Rust match expressions with different semantics
- **#define / #macro**: `#define` names a constant (→ a Rust `const`); `#macro` defines a parameterised macro (→ `macro_rules!`)

### Pure Syntax Transformations

- `Type?` → `Result<Type, Box<dyn std::error::Error>>` (fallible types)
- `expr?` → `expr?` (error propagation - pass through)
- `@Type.method()` → `Type::method()` (type-scoped calls with @ prefix)
- `.label:` → `'label:` (loop labels with dot prefix)
- `name!()` → `name!()` (macro calls use Rust's bang form and pass through)

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
for (int i = 0; i < 100; ++i) {  // → Rust loop with scoped variable
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

**See**: [docs/design/philosophy.md](docs/design/philosophy.md) for detailed rationale.

## Features

### C-like Syntax with Rust Safety
- **Familiar C syntax**: Write code with C-style function declarations, control flow, and data structures
- **Type-scoped calls**: Use `@Type` prefix with dot notation: `@Vec.new()`, `@Option.None`
- **Macro system**: define constants with `#define`, parameterised macros with `#macro`, and invoke with Rust's bang form: `println!("Hello")`, `vec![1, 2, 3]`
- **Closures**: Nested functions that capture outer scope (transpile to Rust closures)
- **Escape hatch**: Use `__rust__` to embed raw Rust code for advanced features
- **Rust compatibility**: All Crusty code transpiles to safe, idiomatic Rust

### Rust Ecosystem Integration
- **Use Rust crates**: Import and use any existing Rust crate or module
- **Publish crates**: Compile Crusty code into crates that native Rust projects can depend on
- **Build system integration**: Works seamlessly with Cargo through build.rs scripts

### Safety Guarantees
- **Memory safety**: Rust's ownership and borrowing model prevents memory errors
- **Type safety**: Strong static typing catches errors at compile time
- **No null pointers**: Uses Rust's Option type for nullable values
- **No data races**: Rust's concurrency model prevents data races

## Quick Start

Get up and running with Crusty in minutes.

### Installation

```bash
git clone https://github.com/major0/crusty.git
cd crusty
cargo build --release
cargo install --path .
```

### Your First Program

Create `hello.crst`:
```c
void main() {
    println!("Hello, Crusty!");
}
```

Transpile and run:
```bash
crustyc hello.crst --emit=binary -o hello
./hello
```

**📖 For detailed installation instructions and usage, see [QUICK_START.md](QUICK_START.md)**

## Syntax Overview

Crusty provides C-like syntax that transpiles to Rust:

```c
// Functions with C-style declarations
int add(int a, int b) {
    return a + b;
}

// Structs with implementation blocks
typedef struct {
    int x;
    int y;
} Point;

typedef struct {
    Point new(int x, int y) {
        return Point { x: x, y: y };
    }
    
    int distance_squared(&self) {
        return self.x * self.x + self.y * self.y;
    }
} @Point;

// Type-scoped calls with @ prefix
void main() {
    let p = @Point.new(3, 4);
    println!("Distance²: {}", p.distance_squared());
}
```

### Key Syntax Features

- **Type-scoped calls**: `@Type.method()` → `Type::method()`
- **Macros**: `println!()` → `println!()` (pass through unchanged)
- **Error handling**: `Type?` → `Result<Type, E>`
- **NULL handling**: `NULL` → `Option::None`
- **Module system**: `#import` and `#export`
- **Nested functions**: C-style closures
- **Raw Rust**: `__rust__{}` escape hatch

**📖 For the complete syntax reference — grammar, examples, and the emitted Rust for every feature — see [docs/syntax/](docs/syntax/README.md). The [Transformation Summary](docs/syntax/transformations.md) is a one-page map of every form to its Rust output.**

## Usage

### Basic Commands

```bash
# Transpile to Rust
crustyc input.crst -o output.rs

# Compile to binary
crustyc input.crst --emit=binary -o program
```

### Build Integration

Integrate Crusty with Cargo projects using build.rs:

```rust
// build.rs
use std::process::Command;
use std::env;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    Command::new("crustyc")
        .args(&["src/", "--out-dir", &out_dir])
        .status()
        .expect("Failed to run crustyc");
    println!("cargo:rerun-if-changed=src/");
}
```

**📖 For detailed usage and build integration, see [QUICK_START.md](QUICK_START.md)**

## Example Project

The repository includes a complete working example in the [example/](example/) directory:

```bash
cd example
cargo build
cargo run
```

The example demonstrates all major Crusty features and is automatically tested in CI/CD.

## Development

Developer setup, the full test/format/lint workflow, pre-commit hooks, the
CI/CD workflow architecture, and the release process are documented in
**[CONTRIBUTING.md](CONTRIBUTING.md)**.

Quick start for contributors:

```bash
cargo test          # run the test suite
cargo fmt           # format
cargo clippy        # lint
crusty test         # run a Crusty project's tests via the cargo harness
```

## Documentation

### Getting Started
- **[QUICK_START.md](QUICK_START.md)** - Installation, first program, and basic usage
- **[Syntax Reference](docs/syntax/README.md)** - Per-topic syntax guide with grammar and emitted-Rust examples
- **[Example Programs](example/README.md)** - Working examples with build integration

### Core Concepts
- **[Philosophy](#philosophy)** - Understand syntax-only transpilation
- **[docs/design/philosophy.md](docs/design/philosophy.md)** - Detailed rationale and design principles

### Integration
- **[Build System Integration](QUICK_START.md#build-integration)** - Using Crusty with Cargo
- **[Build.rs Guide](docs/build-rs-integration.md)** - Comprehensive build integration guide

### Contributing
- **[Contributing Guide](CONTRIBUTING.md)** - How to contribute to the project
- **[Development Workflow](CONTRIBUTING.md#development-workflow)** - Running tests, formatting, linting

### Design &amp; Architecture
- **[Architecture Overview](docs/architecture/overview.md)** - Compiler pipeline and component design
- **[Design Notes](docs/design/README.md)** - Language design decisions
- **[Syntax Reference](docs/syntax/README.md)** - Per-feature syntax documentation
- **[Architecture Decision Records](docs/adr/)** - Numbered, durable design decisions

## Contributing

Contributions are welcome! See **[CONTRIBUTING.md](CONTRIBUTING.md)** for the
development workflow, branch and commit conventions, testing and CI
expectations, and the review and release process.

## License

This project is licensed under the MIT License - see the [LICENSE.txt](LICENSE.txt) file for details.

## Project Status

**Phase 1: Core Transpiler** - In Active Development

Current progress:
- ✅ Infrastructure (CI/CD, pre-commit hooks, licensing)
- ✅ Core transpiler (PEG parser, AST, semantic analysis)
- ✅ Code generation (Crusty → Rust)
- ✅ Advanced parsing (structs, methods, generics, macros)
- ✅ Example directory with working samples ([see example/](example/))
- ✅ Build system integration (build.rs support)
- 🚧 Bidirectional transpilation (Rust → Crusty) — planned, not yet implemented
- 🚧 Documentation generator (crustydoc)

## Roadmap

### Phase 1 (Current)
- Core transpiler infrastructure
- Crusty → Rust transpilation
- Basic language features
- Build system integration

### Phase 2 — Landed
- Generic definitions (functions, structs, enums) with bounds and `where`
- Trait definitions and `impl`/trait-impl via the `typedef … @Type` surface
- Pattern matching (`match (scrutinee) { … }`)
- `unsafe` blocks / `unsafe fn`
- Concurrency primitives (`__thread__`, `__scope__`)
- Enum data variants (tuple, struct, and discriminant payloads)
- Tuple and unit structs (`struct P(int, int);`, `struct Marker;`)
- Verbatim attribute pass-through (`#[...]`, `#![...]`)
- Trait-object and `impl Trait` type positions (`dyn`, `impl`)
- C99-style conditional binding (`if (decl; cond)`, `while (decl; cond)`)
- Full `#ifdef` directive family (`#elif`/`#elifdef`/`#elifndef`/`#else`)

### Phase 3 (Planned)
- Enhanced macro system
- `async`/`await` support (currently available via the `__rust__` escape hatch)
- Procedural macros

### Phase 4 (Future)
- IDE integration (LSP support)
- Debugger integration
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
