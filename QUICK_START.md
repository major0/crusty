# Crusty Quick Start Guide

This guide will help you get started with Crusty in minutes.

## Table of Contents

1. [Installation](#installation)
2. [Your First Program](#your-first-program)
3. [Basic Usage](#basic-usage)
4. [Build Integration](#build-integration)
5. [Example Project](#example-project)
6. [Next Steps](#next-steps)

---

## Installation

### Prerequisites

- **Rust toolchain (stable)** - [Install Rust](https://rustup.rs/)
- **Cargo package manager** (included with Rust)

### Building from Source

```bash
git clone https://github.com/major0/crusty.git
cd crusty
cargo build --release
cargo install --path .
```

### Verify Installation

```bash
crustyc --version
```

---

## Your First Program

### Create a Hello World Program

Create a file named `hello.crst`:

```c
void main() {
    __println__("Hello, Crusty!");
}
```

### Transpile and Run

**Option 1: Transpile to Rust**
```bash
crustyc hello.crst -o hello.rs
rustc hello.rs
./hello
```

**Option 2: Compile Directly to Binary**
```bash
crustyc hello.crst --emit=binary -o hello
./hello
```

**Output:**
```
Hello, Crusty!
```

---

## Basic Usage

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

### Common Commands

**Transpile Crusty to Rust:**
```bash
crustyc input.crst -o output.rs
```

**Transpile and compile to binary:**
```bash
crustyc input.crst --emit=binary -o program
```

**Transpile Rust to Crusty:**
```bash
crustyc input.rs --from-lang=rust -o output.crst
```

**View AST (for debugging):**
```bash
crustyc input.crst --emit=ast
```

---

## Build Integration

### Using Crusty in Cargo Projects

Crusty integrates seamlessly with Cargo through build.rs scripts.

#### Step 1: Add Build Dependency

Add crustyc as a build dependency in `Cargo.toml`:

```toml
[package]
name = "my-project"
version = "0.1.0"
edition = "2021"

[build-dependencies]
# crustyc = "0.1"  # When published to crates.io
```

#### Step 2: Create build.rs

Create `build.rs` in your project root:

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

#### Step 3: Organize Your Project

Place your `.crst` files in `src/`:

```
my-project/
├── Cargo.toml
├── build.rs
└── src/
    ├── main.crst
    ├── lib.crst
    └── utils.crst
```

#### Step 4: Build with Cargo

```bash
cargo build
cargo run
```

The build.rs script automatically transpiles your Crusty code to Rust during the build process.

---

## Example Project

The repository includes a complete working example demonstrating Crusty language features and build system integration.

### Location

See the [example/](example/) directory for:
- [example/Cargo.toml](example/Cargo.toml) - Project configuration
- [example/build.rs](example/build.rs) - Build script that transpiles .crst files
- [example/src/](example/src/) - Sample Crusty programs
- [example/README.md](example/README.md) - Build and run instructions

### What the Example Demonstrates

- Function declarations and control flow
- Struct definitions with methods
- Type-scoped static method calls (`@Type.method()`)
- Macro usage with double-underscore naming (`__println__`, `__vec__`)
- Build system integration with Cargo

### Running the Example

```bash
cd example
cargo build
cargo run
```

The example is automatically built and tested in the CI/CD pipeline to ensure the transpiler works correctly.

---

## Next Steps

### Learn the Syntax

- **[SYNTAX_REFERENCE.md](SYNTAX_REFERENCE.md)** - Complete syntax guide with examples
- **[Philosophy](README.md#philosophy)** - Understand Crusty's design principles
- **[SYNTAX_PHILOSOPHY.md](.kiro/specs/crusty-compiler-phase1/SYNTAX_PHILOSOPHY.md)** - Detailed rationale

### Explore Features

- **Functions and Types** - C-style function declarations
- **Structs and Methods** - Define types with implementation blocks
- **Error Handling** - Using `Type?` for fallible functions
- **NULL Handling** - C-style NULL maps to Rust's Option
- **Macros** - Double-underscore naming convention
- **Type-Scoped Calls** - `@Type.method()` syntax
- **Module System** - `#import` and `#export` directives

### Get Help

- **[GitHub Issues](https://github.com/major0/crusty/issues)** - Report bugs or request features
- **[GitHub Discussions](https://github.com/major0/crusty/discussions)** - Ask questions
- **[Contributing Guide](CONTRIBUTING.md)** - Contribute to the project

### Development Tools

**Run tests:**
```bash
cargo test
```

**Format code:**
```bash
cargo fmt
```

**Lint code:**
```bash
cargo clippy
```

**Install pre-commit hooks:**
```bash
pip install pre-commit
pre-commit install
```

---

## Troubleshooting

### crustyc: command not found

Make sure `~/.cargo/bin` is in your PATH:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Add this to your `~/.bashrc` or `~/.zshrc` to make it permanent.

### Build Errors

If you encounter build errors:

1. Ensure Rust is up to date: `rustup update`
2. Clean and rebuild: `cargo clean && cargo build`
3. Check for syntax errors in your `.crst` files
4. Run with verbose output: `crustyc -v input.crst`

### Generated Rust Code Issues

If the generated Rust code doesn't compile:

1. Check the generated `.rs` file for issues
2. Report bugs at [GitHub Issues](https://github.com/major0/crusty/issues)
3. Use `__rust__` escape hatch for unsupported features

---

## Additional Resources

- **[README.md](README.md)** - Project overview and philosophy
- **[SYNTAX_REFERENCE.md](SYNTAX_REFERENCE.md)** - Complete syntax guide
- **[Build Integration Guide](docs/build-rs-integration.md)** - Comprehensive build.rs guide
- **[Requirements](.kiro/specs/crusty-compiler-phase1/requirements.md)** - Detailed requirements
- **[Design](.kiro/specs/crusty-compiler-phase1/design.md)** - Architecture and design
- **[Tasks](.kiro/specs/crusty-compiler-phase1/tasks.md)** - Implementation progress

---

**Ready to dive deeper?** Check out the [SYNTAX_REFERENCE.md](SYNTAX_REFERENCE.md) for comprehensive syntax examples and transformations.
