# Rust Integration

## Introduction

Crusty is designed to integrate seamlessly with the Rust ecosystem. Crusty programs can use external Rust crates, and Crusty libraries can be published as Rust crates.

## Standard Library

Crusty programs use Rust's standard library directly:

```crusty
// Use Vec from std with type-scoped call syntax
Vec<int> numbers = __vec__[1, 2, 3];
numbers.push(4);

// Use String with type-scoped call syntax
String s = @String.from("hello");
s.push_str(" world");

// Use println macro with double-underscore naming (no ! in Crusty)
__println__("Result: {}", result);
```

## Using External Crates

Crusty code can import and use types from external Rust crates:

```crusty
// Import external crate types
#use serde.Serialize;
#use serde.Deserialize;
#use tokio.runtime.Runtime;

// Use external types in Crusty code
#[derive(Serialize, Deserialize)]
struct User {
    name: char*,
    age: i32,
}

void process_user(User* user) {
    // Use external crate functions
    let json = @serde_json.to_string(user)?;
    __println__("{}", json);
}
```

### Type Compatibility

The transpiler ensures type compatibility between Crusty and external Rust types:
- Crusty structs can implement external traits
- External types can be used in Crusty function signatures
- Generic types from external crates work correctly

## Cargo Integration

Crusty projects use Cargo for dependency management:

```toml
[package]
name = "my-crusty-project"
version = "0.1.0"

[dependencies]
# Regular Rust dependencies work
serde = "1.0"

[build-dependencies]
crustyc = "0.1"
```

## Build Integration

Crusty files are transpiled during the build process. See [Build Integration](build-integration.md) for detailed documentation.

```rust
// build.rs
fn main() {
    crustyc::compile_crusty_files("src/**/*.crst");
}
```

## Publishing Crusty Libraries

Crusty libraries can be published as Rust crates:

### Library Structure

```
my-crusty-lib/
├── Cargo.toml
├── build.rs
└── src/
    ├── lib.crst
    └── utils.crst
```

### Cargo.toml

```toml
[package]
name = "my-crusty-lib"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["rlib"]

[build-dependencies]
crustyc = "0.1"
```

### Publishing

```bash
cargo build --release
cargo publish
```

## Consuming Crusty Libraries from Rust

Rust projects can depend on Crusty libraries:

```toml
[dependencies]
my-crusty-lib = "0.1"
```

```rust
// Rust code using Crusty library
use my_crusty_lib::User;

fn main() {
    let user = User::new("Alice", 30);
    user.process();
}
```

### API Compatibility

The transpiler ensures that Crusty libraries expose Rust-compatible APIs:
- Public functions become `pub fn`
- Public structs become `pub struct`
- Type signatures are Rust-compatible
- Documentation comments are preserved

## Escape Hatch

When Crusty syntax is insufficient, embed raw Rust:

```crusty
void example() {
    // Crusty code
    int x = 42;
    
    // Raw Rust block
    __rust__ {
        let closure = |y| y * 2;
        let result = closure(x);
    }
}
```

## Interoperability

Crusty code can call Rust code and vice versa:

```crusty
// Call Rust function
extern fn rust_function(int x) -> int;

void main() {
    int result = rust_function(42);
}
```

```rust
// Call Crusty function from Rust
extern "C" fn crusty_function(x: i32) -> i32;
```

## Performance Parity

Crusty code compiles to the same Rust code that a human would write, ensuring:
- No runtime overhead
- Same optimization opportunities
- Identical performance characteristics
- Zero-cost abstractions
