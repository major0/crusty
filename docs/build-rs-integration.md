# Build.rs Integration Guide

This guide explains how to integrate Crusty transpilation into your Cargo build process using a `build.rs` script.

## Overview

Crusty projects use standard Cargo for building, with a `build.rs` script that handles transpilation from Crusty to Rust. This approach provides:

- **Standard Cargo workflow**: Use familiar `cargo build`, `cargo test`, `cargo run` commands
- **Incremental builds**: Only re-transpile changed files
- **IDE integration**: Works with rust-analyzer and other Rust tooling
- **Dependency management**: Use Cargo.toml for all dependencies

## Basic Setup

### 1. Project Structure

```
my-crusty-project/
├── Cargo.toml
├── build.rs
└── src/
    ├── main.crst
    ├── lib.crst
    └── module/
        └── helper.crst
```

### 2. Cargo.toml Configuration

```toml
[package]
name = "my-crusty-project"
version = "0.1.0"
edition = "2021"

[build-dependencies]
# Add crustyc as a build dependency
crustyc = "0.1"

[dependencies]
# Your runtime dependencies go here
```

### 3. Build Script (build.rs)

See `build.rs.example` in the repository root for a complete, documented example.

Basic structure:

```rust
use std::env;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    
    // Transpile all .crst files in src/ to OUT_DIR
    let status = Command::new("crustyc")
        .arg("src")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--emit")
        .arg("rust")
        .arg("--no-compile")
        .status()
        .expect("Failed to execute crustyc");
    
    if !status.success() {
        panic!("Transpilation failed");
    }
    
    // Set up incremental builds
    println!("cargo:rerun-if-changed=src");
}
```

## Transpilation Modes

### Single File Mode

Transpile files one at a time:

```rust
for crst_file in discover_crst_files("src")? {
    Command::new("crustyc")
        .arg(&crst_file)
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--emit")
        .arg("rust")
        .arg("--no-compile")
        .status()?;
    
    println!("cargo:rerun-if-changed={}", crst_file.display());
}
```

**Pros**: Fine-grained control, clear error messages per file
**Cons**: Slower for many files

### Batch Mode

Transpile entire directory at once:

```rust
Command::new("crustyc")
    .arg("src")
    .arg("--out-dir")
    .arg(&out_dir)
    .arg("--emit")
    .arg("rust")
    .arg("--no-compile")
    .status()?;
```

**Pros**: Faster for many files, simpler code
**Cons**: Less granular error reporting

## Incremental Builds

To enable incremental builds, use `cargo:rerun-if-changed` directives:

```rust
// Re-run build script when any .crst file changes
for crst_file in discover_crst_files("src")? {
    println!("cargo:rerun-if-changed={}", crst_file.display());
}

// Or watch the entire src directory
println!("cargo:rerun-if-changed=src");
```

## Directory Structure Preservation

The `--out-dir` option preserves the source directory structure:

```
src/
├── main.crst
└── module/
    └── helper.crst

OUT_DIR/
├── main.rs
└── module/
    └── helper.rs
```

This ensures that module paths remain consistent between Crusty and Rust.

## Error Handling

### Build Script Errors

If transpilation fails, the build script should panic with a clear error message:

```rust
let status = Command::new("crustyc")
    .arg("src")
    .arg("--out-dir")
    .arg(&out_dir)
    .arg("--emit")
    .arg("rust")
    .arg("--no-compile")
    .status()
    .expect("Failed to execute crustyc - is it installed?");

if !status.success() {
    panic!("Transpilation failed - check crustyc output above");
}
```

### Verbose Output

For debugging, enable verbose output:

```rust
Command::new("crustyc")
    .arg("src")
    .arg("--out-dir")
    .arg(&out_dir)
    .arg("--emit")
    .arg("rust")
    .arg("--no-compile")
    .arg("--verbose")  // Add verbose flag
    .status()?;
```

## Advanced Patterns

### Conditional Compilation

Use Cargo features to conditionally include Crusty files:

```rust
#[cfg(feature = "crusty-modules")]
{
    // Transpile additional .crst files
    Command::new("crustyc")
        .arg("src/optional")
        .arg("--out-dir")
        .arg(&out_dir)
        .arg("--emit")
        .arg("rust")
        .arg("--no-compile")
        .status()?;
}
```

### Custom Output Locations

For more control over output locations:

```rust
// Transpile to a specific subdirectory
let custom_out = Path::new(&out_dir).join("generated");
fs::create_dir_all(&custom_out)?;

Command::new("crustyc")
    .arg("src")
    .arg("--out-dir")
    .arg(&custom_out)
    .arg("--emit")
    .arg("rust")
    .arg("--no-compile")
    .status()?;
```

### Parallel Transpilation

For large projects, transpile files in parallel:

```rust
use std::thread;

let handles: Vec<_> = crst_files
    .chunks(4)  // Process 4 files per thread
    .map(|chunk| {
        let chunk = chunk.to_vec();
        let out_dir = out_dir.clone();
        
        thread::spawn(move || {
            for file in chunk {
                Command::new("crustyc")
                    .arg(&file)
                    .arg("--out-dir")
                    .arg(&out_dir)
                    .arg("--emit")
                    .arg("rust")
                    .arg("--no-compile")
                    .status()
                    .expect("Failed to transpile");
            }
        })
    })
    .collect();

for handle in handles {
    handle.join().unwrap();
}
```

## Troubleshooting

### "crustyc not found"

Ensure crustyc is installed and in PATH:

```bash
cargo install crustyc
```

Or specify the full path in build.rs:

```rust
Command::new("/path/to/crustyc")
```

### "OUT_DIR not set"

This usually means the build script is being run outside of Cargo. Always use `cargo build`.

### Incremental builds not working

Make sure you're using `cargo:rerun-if-changed` directives for all .crst files.

### Directory structure not preserved

Verify you're using `--out-dir` (not `-o`) and passing a directory path.

## Examples

See the `example/` directory in the Crusty repository for complete working examples.

## Best Practices

1. **Always use --no-compile**: Let Cargo handle Rust compilation
2. **Set up rerun-if-changed**: Enable incremental builds
3. **Use batch mode for many files**: More efficient than single-file mode
4. **Handle errors gracefully**: Provide clear error messages
5. **Document your build.rs**: Explain any custom logic

## Integration with CI/CD

In CI/CD pipelines, ensure crustyc is installed before building:

```yaml
# GitHub Actions example
- name: Install crustyc
  run: cargo install crustyc

- name: Build project
  run: cargo build --release
```

## Next Steps

- Read the [Crusty Language Guide](language-guide.md)
- Explore [Module System](module-system.md)
- See [Example Projects](../example/)
