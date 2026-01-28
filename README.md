# Crusty Programming Language

[![CI](https://github.com/major0/crusty/workflows/CI/badge.svg)](https://github.com/major0/crusty/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Crusty is a C-like programming language that provides familiar C syntax while guaranteeing Rust compatibility. Phase 1 focuses on building a bidirectional transpiler between Crusty and Rust source code.

## Features

- **C-like Syntax**: Write code with familiar C syntax
- **Rust Compatibility**: Transpiles to safe, idiomatic Rust code
- **Bidirectional**: Convert between Crusty and Rust
- **Type Safety**: Leverages Rust's type system and ownership model
- **No Standard Library**: Uses Rust's std library directly

## Installation

### Prerequisites

- Rust toolchain (stable)
- Cargo package manager

### Building from Source

```bash
git clone https://github.com/major0/crusty.git
cd crusty
cargo build --release
```

## Usage

### Compile Crusty to Rust

```bash
crustyc input.crst -o output.rs
```

### Compile Crusty to Binary

```bash
crustyc input.crst --emit=binary -o program
```

### Transpile Rust to Crusty

```bash
crustyc input.rs --from-lang=rust -o output.crst
```

## Development

### Pre-commit Hooks

Install pre-commit hooks for code quality checks:

```bash
pip install pre-commit
pre-commit install
```

### Running Tests

```bash
cargo test
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Documentation

For detailed documentation, see:
- [Requirements](/.kiro/specs/crusty-compiler-phase1/requirements.md)
- [Design](/.kiro/specs/crusty-compiler-phase1/design.md)
- [Implementation Tasks](/.kiro/specs/crusty-compiler-phase1/tasks.md)

## License

This project is licensed under the MIT License - see the [LICENSE.txt](LICENSE.txt) file for details.

## Contributing

Contributions are welcome! Please ensure:
- All tests pass (`cargo test`)
- Code is formatted (`cargo fmt`)
- No clippy warnings (`cargo clippy`)
- Commits follow [Conventional Commits](https://www.conventionalcommits.org/)

## Project Status

Phase 1 is currently in active development. See [tasks.md](/.kiro/specs/crusty-compiler-phase1/tasks.md) for implementation progress.
