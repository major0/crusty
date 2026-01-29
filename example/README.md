# Crusty Example Programs

This directory contains example Crusty programs that demonstrate the language features and syntax.

## Overview

The examples showcase:
- **main.crst**: Basic program with arithmetic
- **functions.crst**: Various function declaration styles and patterns
- **structs.crst**: Struct definitions
- **methods.crst**: Struct methods with self parameters
- **generics.crst**: Generic type syntax (documentation only)
- **attributes.crst**: Attribute usage (#[derive], #[test], etc.)
- **macros.crst**: Macro syntax documentation
- **defines.crst**: #define macro definitions (translates to macro_rules!)
- **ranges.crst**: Range syntax (planned feature)
- **slices.crst**: Slice types (planned feature)
- **closures.crst**: Nested functions (planned feature)
- **rust_escape.crst**: __rust__ escape hatch (planned feature)

## Current Implementation Status

### ✅ Fully Implemented
- C-style function declarations (`int add(int a, int b)`)
- Struct definitions
- Struct methods with `&self` parameters
- Attributes (#[derive], #[test], #[cfg], etc.)
- #define macro definitions (translates to Rust macro_rules!)
- Variable declarations (let, var, const)
- Control flow (if/else, while, for loops)
- Basic expressions and operators

### 📋 Planned Features
- Macro invocations (__println__, __vec__, etc.) - requires semantic analyzer support
- Range syntax (start..end, start..=end, etc.)
- Slice operations with ranges
- Nested functions (closures)
- __rust__ escape hatch for raw Rust code
- Type-scoped calls with @ prefix (@Type.method())
- Explicit generic parameters

## Building the Examples

The example project uses Cargo with a `build.rs` script that automatically transpiles `.crst` files to Rust before compilation.

### Prerequisites

- Rust toolchain (rustc, cargo)
- crustyc transpiler installed and available in PATH

### Installing crustyc

First, install the crustyc transpiler:

```bash
# From the repository root
cargo install --path .
```

This will install `crustyc` to your Cargo bin directory (typically `~/.cargo/bin`), which should be in your PATH.

Alternatively, for development:

```bash
# Build crustyc in release mode
cargo build --release

# Add to PATH temporarily
export PATH="$PWD/target/release:$PATH"
```

### Build Steps

Once crustyc is installed and in your PATH:

```bash
cd example
cargo build
```

The build process will:
1. Discover all `.crst` files in `src/`
2. Transpile each `.crst` file to Rust using crustyc
3. Compile the generated Rust code
4. Link everything into a binary

### Running the Examples

After building, run the example binary:

```bash
cargo run
```

Or run the compiled binary directly:

```bash
./target/debug/crusty-example
```

## How It Works

### build.rs Script

The `build.rs` script is executed before the main compilation:

1. **Discovery**: Recursively finds all `.crst` files in `src/`
2. **Transpilation**: Invokes `crustyc` for each file with `--emit=rust`
3. **Output**: Writes generated `.rs` files to `OUT_DIR`
4. **Incremental Builds**: Uses `cargo:rerun-if-changed` to only retranspile modified files

### Cargo Integration

The `Cargo.toml` specifies crustyc as a build-dependency:

```toml
[build-dependencies]
crustyc = { path = ".." }
```

This ensures crustyc is available during the build process.

## Example Code Walkthrough

### main.crst

Demonstrates:
- C-style function declarations (`int add(int a, int b)`)
- Variable declarations with type annotations
- Function calls and return values

### functions.crst

Demonstrates:
- Public functions (default)
- Static (private) functions
- Void functions
- Multiple parameters
- Early returns
- Boolean return types

### structs.crst

Demonstrates:
- Basic struct definitions
- Field declarations with types
- Simple functions

### methods.crst

Demonstrates:
- Struct methods with &self parameters
- Instance method definitions within structs

### generics.crst

Demonstrates (syntax documentation only):
- Generic type parameter syntax with parentheses
- Type-scoped calls with @ prefix
- Nested generic parameters

### attributes.crst

Demonstrates:
- #[derive(...)] for automatic trait implementations
- #[test] for unit tests
- #[allow(...)] for suppressing warnings
- Attributes on structs and functions
- Multiple attributes on the same item

### macros.crst

Demonstrates (syntax documentation only):
- Macro invocation syntax with double-underscore naming
- Translation from __name__ to Rust name! syntax
- Note: Macro invocations require semantic analyzer support

### defines.crst

Demonstrates:
- #define macro definitions with double-underscore naming
- Simple constant macros
- Macros with parameters
- Ternary operator in macro bodies
- Translation to Rust macro_rules!

### ranges.crst, slices.crst, closures.crst, rust_escape.crst

These files document planned features with syntax examples in comments.

## Crusty Syntax Highlights

### Function Declarations

Crusty uses C-style function syntax:

```crusty
// Return type comes before function name
int add(int a, int b) {
    return a + b;
}

// void for no return value
void print_message() {
    // function body
}

// static for private functions
static int helper(int x) {
    return x * 2;
}
```

### #define Macros

Crusty supports #define for macro definitions with double-underscore naming:

```crusty
// Simple constant macro
#define __MAX_SIZE__ 100

// Macro with parameters
#define __ADD__(a, b) ((a) + (b))

// Macro with ternary operator
#define __MAX__(x, y) ((x) > (y) ? (x) : (y))
```

These transpile to Rust macro_rules!:

```rust
macro_rules! max_size {
    () => {{ 100 }};
}

macro_rules! add {
    ($a:expr, $b:expr) => {{ (($a) + ($b)) }};
}

macro_rules! max {
    ($x:expr, $y:expr) => {{
        if $x > $y { $x } else { $y }
    }};
}
```

### Attributes

Crusty supports Rust-style attributes:

```crusty
#[derive(Debug, Clone)]
struct Point {
    int x;
    int y;
}

#[test]
void test_addition() {
    let result = 2 + 2;
}
```

### Control Flow

Standard C-like control flow:

```crusty
if (condition) {
    // then branch
} else {
    // else branch
}

while (condition) {
    // loop body
}

for (int i = 0; i < 10; i = i + 1) {
    // loop body
}
```

## Adding More Examples

To add new examples:

1. Create a new `.crst` file in `src/`
2. Write your Crusty code
3. Run `cargo build` - the build script will automatically discover and transpile it

The build system handles everything automatically!

## Troubleshooting

### Build Errors

If you encounter build errors:

1. **Check crustyc is built**: Ensure the transpiler is compiled in the parent directory
2. **Check syntax**: Verify your `.crst` files have valid Crusty syntax
3. **Check generated Rust**: Look in `target/debug/build/crusty-example-*/out/` to see generated `.rs` files
4. **Enable verbose output**: Use `cargo build -vv` to see detailed build steps

### Transpilation Errors

If crustyc reports errors:

1. Check the error message for line/column information
2. Verify your Crusty syntax matches the language specification
3. Ensure you're using supported features (no unions, goto, or #include)

## Next Steps

- Explore the generated Rust code in the build output directory
- Modify the examples to experiment with Crusty syntax
- Add your own `.crst` files to test different language features
- Check the main Crusty documentation for advanced features

## License

MIT License - See LICENSE.txt in the repository root.
