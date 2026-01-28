# Rust Integration

## Introduction

Crusty is designed to integrate seamlessly with the Rust ecosystem.

## Standard Library

Crusty programs use Rust's standard library directly:

```crusty
// Use Vec from std with type-scoped call syntax
Vec<int> numbers = __vec__[1, 2, 3];
numbers.push(4);

// Use String with type-scoped call syntax
String s = @String->from("hello");
s.push_str(" world");

// Use println macro with double-underscore naming (no ! in Crusty)
__println__("Result: {}", result);
```

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

Crusty files are transpiled during the build process:

```rust
// build.rs
fn main() {
    crustyc::compile_crusty_files("src/**/*.crst");
}
```

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
