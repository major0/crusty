# Crusty Example Programs

This directory contains example Crusty programs that demonstrate the language features and syntax.

## Overview

The examples showcase:
- **main.crst**: Basic hello world program with arithmetic and control flow
- **functions.crst**: Various function declaration styles and patterns
- **structs.crst**: Struct definitions and usage

## Building the Examples

The example project uses Cargo with a `build.rs` script that automatically transpiles `.crst` files to Rust before compilation.

### Prerequisites

- Rust toolchain (rustc, cargo)
- crustyc transpiler (built from parent directory)

### Build Steps

1. Build the crustyc transpiler first (from repository root):
   ```bash
   cargo build --release
   ```

2. Build the example project:
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
- C-style function declarations (`int main()`, `int add(int a, int b)`)
- Variable declarations with type annotations
- Macro invocations with double-underscore naming (`__println__`)
- Control flow (if/else, while loops)
- Function calls and return values

### functions.crst

Demonstrates:
- Public functions (default)
- Static (private) functions
- Void functions
- Multiple parameters
- Early returns
- Nested control flow
- For loops
- Boolean return types
- Break and continue statements

### structs.crst

Demonstrates:
- Struct definitions
- Field access with dot notation
- Struct initialization
- Nested structures
- Functions operating on structs

## Crusty Syntax Highlights

### Function Declarations

Crusty uses C-style function syntax:

```crusty
// Return type comes before function name
int add(int a, int b) {
    return a + b;
}

// void for no return value
void print_message(char* msg) {
    __println__("{}", msg);
}

// static for private functions
static int helper(int x) {
    return x * 2;
}
```

### Macros

Crusty macros use double-underscore naming (no `!` suffix):

```crusty
__println__("Hello, world!");
__vec__[1, 2, 3];
__format__("Value: {}", x);
```

These transpile to Rust macros with `!`:

```rust
println!("Hello, world!");
vec![1, 2, 3];
format!("Value: {}", x);
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
