# Build System Integration

## Introduction

Crusty integrates with Rust's build system through build.rs scripts and multi-file project handling. This enables seamless use of Crusty in Cargo-based projects.

## Rationale

By leveraging Cargo's build script mechanism, Crusty projects can:
- Automatically transpile `.crst` files during the build process
- Support incremental builds through `cargo:rerun-if-changed`
- Integrate with existing Rust tooling and workflows

## CLI Options

### Output Directory

The `--out-dir` option specifies where generated Rust files should be placed:

```bash
crustyc --out-dir target/generated src/main.crst
```

This creates the output directory if it doesn't exist and preserves the source directory structure.

### Batch Transpilation

Transpile multiple files in a single invocation:

```bash
crustyc --out-dir target/generated src/**/*.crst
```

This discovers all `.crst` files and transpiles them to the output directory.

## build.rs Integration

### Basic Example

```rust
// build.rs
use std::env;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    // Transpile Crusty files to OUT_DIR
    let status = Command::new("crustyc")
        .arg("src/main.crst")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--no-compile")
        .status()
        .expect("Failed to run crustyc");
    
    if !status.success() {
        panic!("Failed to transpile Crusty files");
    }
    
    // Tell Cargo to rerun if source changes
    println!("cargo:rerun-if-changed=src/main.crst");
}
```

### Multi-File Projects

For projects with multiple Crusty source files:

```rust
// build.rs
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let src_dir = Path::new("src");
    
    // Discover all .crst files
    for entry in fs::read_dir(src_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        
        if path.extension().and_then(|s| s.to_str()) == Some("crst") {
            let status = Command::new("crustyc")
                .arg(&path)
                .arg("--out-dir")
                .arg(&out_dir)
                .arg("--no-compile")
                .status()
                .expect("Failed to run crustyc");
            
            if !status.success() {
                panic!("Failed to transpile {:?}", path);
            }
            
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
```

## Module Resolution

Module resolution in Crusty is handled by Rust's module system (rustc), not by crustyc. This design keeps the transpiler simple and leverages Rust's mature module resolution capabilities.

### How It Works

When encountering a `#use` directive:

```crusty
#use crate.utils.helpers;
```

Crusty's role is simply to:
1. Translate `#use` directives to Rust `use` statements
2. Transpile `.crst` files to `.rs` files
3. Preserve directory structure in the output

Rust's module system (rustc) handles:
- Resolving module imports
- Building dependency graphs
- Symbol resolution across module boundaries
- Locating source files

### Example

```crusty
// src/main.crst
#use crate.utils.helpers;

int main() {
    helpers.do_something();
    return 0;
}
```

Transpiles to:

```rust
// target/generated/main.rs
use crate::utils::helpers;

fn main() -> i32 {
    helpers::do_something();
    0
}
```

The generated Rust code is then compiled by rustc, which resolves the `use crate::utils::helpers` statement using standard Rust module resolution rules.

## Example Project Structure

```
my-crusty-project/
├── Cargo.toml
├── build.rs
└── src/
    ├── main.crst
    ├── lib.crst
    └── utils/
        └── helpers.crst
```

### Cargo.toml

```toml
[package]
name = "my-crusty-project"
version = "0.1.0"
edition = "2021"

[build-dependencies]
crustyc = "0.1"
```

## CI/CD Integration

Example GitHub Actions workflow:

```yaml
- name: Build Crusty project
  run: |
    cargo build --verbose
    cargo test
    cargo run
```

This ensures:
1. All Crusty files transpile successfully
2. Generated Rust code compiles
3. Tests pass
4. The binary runs without errors
