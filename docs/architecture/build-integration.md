# Build System Integration

## Introduction

Crusty integrates with Rust's build system through `build.rs` scripts and Cargo. The transpiler provides CLI options for batch transpilation and output directory control, enabling seamless use within Cargo's build pipeline.

## Rationale

Rather than requiring a custom build tool, Crusty leverages Cargo's existing `build.rs` mechanism. This means Crusty projects are standard Cargo projects that happen to have a build script transpiling `.crst` files to Rust before compilation. Existing Rust tooling (cargo test, cargo doc, etc.) works without modification.

## Examples

### build.rs Script
```rust
use std::env;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    // Transpile all .crst files to OUT_DIR
    let status = Command::new("crustyc")
        .arg(&format!("{}/src", manifest_dir))
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--no-compile")
        .status()
        .expect("Failed to run crustyc");

    if !status.success() {
        panic!("Transpilation failed");
    }

    println!("cargo:rerun-if-changed=src/");
}
```

### Multi-File Project Structure
```
my-project/
├── Cargo.toml
├── build.rs
└── src/
    ├── main.crst
    ├── utils/
    │   └── helpers.crst
    └── models/
        └── data.crst
```

### Module Resolution
```c
#use crate.utils.helpers;
```
The transpiler resolves this to `src/utils/helpers.crst`, parses it, and resolves symbols across module boundaries.

## CLI Options

| Option | Description |
|--------|-------------|
| `--out-dir <path>` | Output directory for generated Rust files |
| `--no-compile` | Transpile only, don't invoke rustc |
| Directory as input | Discover and transpile all `.crst` files |
