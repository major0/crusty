# Crusty Example Programs

This directory contains example Crusty programs that demonstrate the language features and syntax.

## Overview

The examples showcase:
- **main.crst**: Basic hello world program with arithmetic and control flow
- **functions.crst**: Various function declaration styles and patterns
- **structs.crst**: Struct definitions and usage
- **methods.crst**: Struct methods and impl blocks with type-scoped calls
- **generics.crst**: Generic types with explicit type parameters
- **attributes.crst**: Attribute usage (#[derive], #[test], etc.)
- **macros.crst**: Macro invocations with double-underscore naming
- **ranges.crst**: Range syntax and operations
- **slices.crst**: Slice types and operations
- **closures.crst**: Nested functions (closures) with capture semantics
- **rust_escape.crst**: Using the __rust__ escape hatch for raw Rust code

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

### methods.crst

Demonstrates:
- Struct methods with self parameters
- Static methods (associated functions)
- Type-scoped static method calls using @Type.method() syntax
- Instance method calls with -> operator
- Multiple impl blocks for the same type

### generics.crst

Demonstrates:
- Generic type parameters with explicit syntax using parentheses
- Type-scoped calls with generic parameters (@Type(T).method())
- Nested generic parameters with alternating parentheses/brackets
- Multiple type parameters
- Type inference when generic parameters are omitted

### attributes.crst

Demonstrates:
- #[derive(...)] for automatic trait implementations
- #[test] for unit tests
- #[cfg(...)] for conditional compilation
- Attributes on structs, functions, and fields
- Multiple attributes on the same item

### macros.crst

Demonstrates:
- Macro invocations with double-underscore naming (no ! suffix)
- __println__, __vec__, __assert__, __format__ macros
- Different macro delimiters (parentheses, brackets, braces)
- Macros in expression and statement contexts
- Note: Crusty macros use __name__ syntax, transpiled to Rust name! syntax

### ranges.crst

Demonstrates:
- Range syntax (start..end, start..=end)
- Open-ended ranges (.., start.., ..end)
- Range usage in for loops
- Range expressions
- Inclusive vs exclusive ranges

### slices.crst

Demonstrates:
- Slice types (&[Type], &var [Type])
- Slice indexing with ranges
- Slice operations (len, is_empty)
- Mutable slices
- Array to slice conversion

### closures.crst

Demonstrates:
- Nested function definitions within functions
- Immutable capture (Fn trait)
- Mutable capture (FnMut trait)
- Multiple captures in a single nested function
- Passing nested functions as parameters
- Scoping rules (only variables defined before are accessible)
- Multiple nested functions sharing captures

### rust_escape.crst

Demonstrates:
- Using __rust__ { ... } to embed raw Rust code
- Pattern matching with match expressions
- Complex closures with trait bounds
- Advanced iterator chains
- Rust-specific features not yet in Crusty
- Mixing Crusty and Rust code in the same function
- Use cases for the escape hatch

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

```c
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

### Type-Scoped Calls

Crusty uses @ prefix and dot notation for type-scoped static method calls:

```c
// Type-scoped static method call
int value = @Option.Some(42);

// With explicit generic parameters (parentheses syntax)
@Vec(int).new();

// Nested type paths
@Foo.Bar.boo();  // Translates to Rust's Foo::Bar.boo()
```

These transpile to Rust's :: syntax:

```rust
let value = Option::Some(42);
Vec::<i32>::new();
Foo::Bar.boo();
```

### Nested Functions (Closures)

Crusty supports defining functions within functions:

```c
void outer() {
    int captured = 42;
    
    int inner(int x) {
        return x + captured;  // Captures 'captured' immutably
    }
    
    __println__("{}", inner(10));
}
```

These transpile to Rust closures:

```rust
fn outer() {
    let captured = 42;
    
    let inner = |x: i32| -> i32 {
        x + captured
    };
    
    println!("{}", inner(10));
}
```

Scoping rule: Nested functions can only capture variables defined BEFORE the function declaration.

### __rust__ Escape Hatch

For Rust-specific features not yet in Crusty, use the __rust__ macro:

```c
void example() {
    __rust__ {
        // Raw Rust code here
        match some_value {
            Some(x) => println!("Got {}", x),
            None => println!("Nothing"),
        }
    }
}
```

The __rust__ block contents are emitted directly as Rust code without translation.

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
