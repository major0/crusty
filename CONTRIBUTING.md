# Contributing to Crusty

Thank you for your interest in contributing to Crusty! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors.

## Getting Started

### Prerequisites

- Rust toolchain (stable) - [Install Rust](https://rustup.rs/)
- Git
- Python 3 (for pre-commit hooks)
- Familiarity with Rust and C programming languages

### Setting Up Your Development Environment

1. **Fork and clone the repository:**
   ```bash
   git clone https://github.com/YOUR_USERNAME/crusty.git
   cd crusty
   ```

2. **Build the project:**
   ```bash
   cargo build
   ```

3. **Run tests to verify setup:**
   ```bash
   cargo test
   ```

4. **Install pre-commit hooks:**
   ```bash
   pip install pre-commit
   pre-commit install
   ```

## Development Workflow

### 1. Create a Feature Branch

```bash
git checkout -b feature/your-feature-name
```

Branch naming conventions:
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation updates
- `test/` - Test additions or updates
- `refactor/` - Code refactoring

### 2. Make Your Changes

- Write clear, readable code
- Follow Rust naming conventions and style guidelines
- Add tests for new functionality
- Update documentation as needed
- Keep commits focused and atomic

### 3. Test Your Changes

Run the full test suite:
```bash
cargo test
```

Run specific tests:
```bash
cargo test test_name
```

Run property-based tests with more iterations:
```bash
cargo test --release -- --ignored
```

### 4. Format and Lint

Format your code:
```bash
cargo fmt
```

Check for common mistakes:
```bash
cargo clippy -- -D warnings
```

### 5. Commit Your Changes

We use [Conventional Commits](https://www.conventionalcommits.org/) for commit messages.

**Format:**
```
type(scope): subject

body

footer
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring
- `chore`: Maintenance tasks
- `style`: Code style changes

**Scope:**
- Task number (e.g., `task-2.1`) when implementing from tasks.md
- Component name (e.g., `parser`, `lexer`, `codegen`)

**Examples:**
```
feat(parser): add support for labeled loops

Implemented parsing for labeled loops with .label: syntax.
Translates to Rust's 'label: syntax.

Validates: Requirements 6.13, 6.14, 6.15
```

```
fix(codegen): correct dot notation in type-scoped calls

Changed @Type.method() syntax to properly translate to Rust Type::method().

Fixes: #42
```

```
docs(readme): add syntax examples section

Added comprehensive examples showing Crusty syntax including
functions, structs, methods, and macros.
```

### 6. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a pull request on GitHub with:
- Clear title describing the change
- Description of what was changed and why
- Reference to related issues (if any)
- Screenshots or examples (if applicable)

## Contribution Guidelines

### Code Style

- Follow Rust standard style guidelines
- Use `cargo fmt` to format code
- Address all `cargo clippy` warnings
- Write clear, self-documenting code
- Add comments for complex logic

### Testing

- Write unit tests for new functions and modules
- Write property-based tests for universal properties
- Write integration tests for end-to-end functionality
- Ensure all tests pass before submitting PR
- Aim for high code coverage (90%+ for core modules)

### Documentation

- Update README.md if adding user-facing features
- Update specification documents (requirements.md, design.md, tasks.md) for significant changes
- Add doc comments to public APIs
- Include examples in documentation
- Update CHANGELOG.md (when it exists)

### Commit Guidelines

- Make atomic commits (one logical change per commit)
- Write clear commit messages following Conventional Commits
- Reference requirements when implementing from tasks.md
- Reference issues when fixing bugs
- Keep commits focused and reviewable

## Types of Contributions

### Bug Reports

When reporting bugs, please include:
- Clear description of the issue
- Steps to reproduce
- Expected behavior
- Actual behavior
- Crusty version
- Operating system
- Minimal code example demonstrating the bug

### Feature Requests

When requesting features, please include:
- Clear description of the feature
- Use case and motivation
- Examples of how it would be used
- Consideration of alternatives
- Willingness to implement (if applicable)

### Code Contributions

We welcome contributions in these areas:

**High Priority:**
- Implementing tasks from tasks.md
- Bug fixes
- Test coverage improvements
- Documentation improvements

**Medium Priority:**
- Performance optimizations
- Error message improvements
- New language features (discuss first)
- Build system enhancements

**Low Priority:**
- Code refactoring
- Style improvements
- Minor optimizations

### Documentation Contributions

Documentation improvements are always welcome:
- Fixing typos and grammar
- Clarifying confusing sections
- Adding examples
- Improving organization
- Translating documentation

## Review Process

### Pull Request Review

All pull requests go through review:
1. Automated CI checks must pass
2. Code review by maintainers
3. Discussion and iteration
4. Approval and merge

### Review Criteria

Reviewers will check for:
- Correctness and functionality
- Test coverage
- Code quality and style
- Documentation completeness
- Adherence to project conventions
- Performance considerations
- Security implications

### Addressing Review Comments

- Respond to all review comments
- Make requested changes in new commits
- Push updates to the same branch
- Request re-review when ready

## Development Tips

### Working with the Codebase

**Key modules:**
- `src/lexer.rs` - Tokenization
- `src/parser.rs` - Parsing Crusty syntax
- `src/ast.rs` - Abstract syntax tree definitions
- `src/semantic.rs` - Semantic analysis and type checking
- `src/codegen.rs` - Code generation (Crusty → Rust)
- `src/cli.rs` - Command-line interface

**Testing:**
- Unit tests are in the same file as the code (Rust convention)
- Property-based tests use the `proptest` crate
- Integration tests are in `tests/` directory

**Debugging:**
- Use `--verbose` flag for detailed output
- Use `--emit=ast` to inspect the AST
- Use `RUST_LOG=debug` for detailed logging

### Common Tasks

**Adding a new language feature:**
1. Update requirements.md with user story and acceptance criteria
2. Update design.md with design details
3. Add task to tasks.md
4. Implement lexer changes (if needed)
5. Implement parser changes
6. Update AST (if needed)
7. Implement semantic analysis (if needed)
8. Implement code generation
9. Add tests (unit and property-based)
10. Update documentation

**Fixing a bug:**
1. Write a failing test that reproduces the bug
2. Fix the bug
3. Verify the test now passes
4. Add regression test if needed
5. Update documentation if behavior changed

## Getting Help

- **Questions**: Open a [GitHub Discussion](https://github.com/major0/crusty/discussions)
- **Bugs**: Open a [GitHub Issue](https://github.com/major0/crusty/issues)
- **Chat**: (Coming soon)

## Recognition

Contributors will be recognized in:
- Git commit history
- Release notes
- Project documentation (future)

Thank you for contributing to Crusty!
