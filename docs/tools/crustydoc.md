# crustydoc - Documentation Generator

## Introduction

crustydoc is a documentation generation tool for Crusty projects. Rather than implementing a custom documentation system, crustydoc acts as a thin wrapper around rustdoc, leveraging the full power of Rust's documentation ecosystem.

## Rationale

The decision to use rustdoc directly instead of a custom documentation generator provides several benefits:

- **Zero maintenance**: rustdoc is maintained by the Rust team
- **Full feature parity**: All rustdoc features work automatically (search, cross-references, examples, etc.)
- **Consistent output**: Documentation looks identical to Rust documentation
- **Cargo integration**: Works seamlessly with Cargo's doc generation

## Architecture

The crustydoc tool follows a simple three-step workflow:

1. Transpiles Crusty source files (.crst) to Rust
2. Invokes rustdoc on the generated Rust code
3. Maps any errors back to Crusty source locations

## CLI Interface

```bash
# Generate documentation for a single file
crustydoc src/lib.crst --output target/doc

# Generate documentation for entire project (via Cargo)
crustydoc --manifest-path Cargo.toml

# Open documentation in browser after generation
crustydoc src/lib.crst --open

# Treat missing documentation as errors
crustydoc src/lib.crst -D missing-docs

# Document private items
crustydoc src/lib.crst --document-private-items

# Pass additional rustdoc options
crustydoc src/lib.crst -- --html-in-header header.html
```

## Cargo Integration

For Crusty projects using Cargo, documentation generation works automatically:

```toml
# Cargo.toml
[package]
name = "my-crusty-lib"
version = "0.1.0"

[build-dependencies]
crustyc = "0.1"
```

```rust
// build.rs
fn main() {
    // Transpile all .crst files to OUT_DIR
    crustyc::transpile_all("src", env::var("OUT_DIR").unwrap());
}
```

Then run:
```bash
cargo doc --open
```

Cargo will:
1. Run build.rs to transpile .crst files
2. Run rustdoc on the generated Rust code
3. Open the documentation in a browser

## Error Mapping

When rustdoc reports errors, crustydoc maps them back to Crusty source locations:

```
error: missing documentation for a function
  --> src/utils.crst:42:1
   |
42 | int calculate(int x, int y) {
   | ^^^^^^^^^^^^^^^^^^^^^^^^^^^
   |
   = note: requested on the command line with `-D missing-docs`
```

## Documentation Validation

crustydoc supports rustdoc's built-in documentation validation:

- `-D missing-docs`: Treat missing documentation as errors
- `--document-private-items`: Include private items in documentation

Documentation coverage statistics are reported from rustdoc output.

## Doc Comment Preservation

The Crusty transpiler preserves all documentation comments when generating Rust code:

- `///` and `/** ... */` outer doc comments
- `//!` and `/*! ... */` inner doc comments

Doc comments maintain their position relative to code elements, ensuring rustdoc generates accurate documentation.
