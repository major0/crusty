# Contributing to Crusty

Thank you for your interest in contributing to Crusty! This document provides guidelines and instructions for contributing to the project.

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors.

## Understanding Crusty's Philosophy

Before contributing, please read [SYNTAX_PHILOSOPHY.md](.kiro/specs/crusty-compiler-phase1/SYNTAX_PHILOSOPHY.md) to understand Crusty's core principle: **syntactic transpilation, not semantic transformation**. This philosophy guides all implementation decisions.

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

### 7. CI/CD Checks

All pull requests automatically run through CI/CD workflows:

**Automated Checks:**
- Pre-commit hook validation
- Code formatting (rustfmt)
- Security audit (cargo-audit)
- Linting (clippy)
- Multi-platform builds (Linux, macOS, Windows)
- Unit tests
- Property-based tests
- Code coverage (90% threshold)

**What to do if CI fails:**
- Review the failed check in the GitHub Actions tab
- Fix the issue locally
- Run the same check locally to verify the fix
- Push the fix to your branch (CI will re-run automatically)

**Common CI failures:**
- **Format check fails**: Run `cargo fmt` locally
- **Clippy fails**: Run `cargo clippy -- -D warnings` and fix warnings
- **Tests fail**: Run `cargo test` locally and fix failing tests
- **Coverage below threshold**: Add tests to increase coverage
- **Security audit fails**: Update dependencies with security vulnerabilities

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

## Release Process

The project uses a structured release process with automated CI/CD workflows.

### Release Branches

Releases are managed through release branches:
- Branch naming: `release/vX.Y` (e.g., `release/v1.2`)
- Each release branch represents a minor version series
- Patch releases are tagged on the release branch

### Release Candidate Process

1. **Create release branch** from main:
   ```bash
   git checkout -b release/v1.2 main
   git push origin release/v1.2
   ```

2. **Automatic validation** runs on push:
   - Validates issue references in commits
   - Runs full CI checks
   - Creates release candidate tag (e.g., `v1.2.0-rc1`)

3. **Issue reference requirement**:
   - All commits in release branch PRs must reference an issue
   - Use: `close #N`, `fix #N`, `resolve #N`, or `related-to #N`
   - At least one referenced issue must be open

4. **Testing release candidates**:
   - Test the RC thoroughly
   - If issues found, fix on release branch and push
   - New RC tag is created automatically (e.g., `v1.2.0-rc2`)

### Final Release Process

1. **Create release tag** on the release branch:
   ```bash
   git tag v1.2.0
   git push origin v1.2.0
   ```

2. **Automatic release workflow** runs:
   - Verifies tag placement on correct branch
   - Verifies version ancestry (v1.2.0 must descend from v1.2.0-rc tags)
   - Runs full CI checks
   - Builds release artifacts for all platforms
   - Generates changelog
   - Creates GitHub release
   - Updates version alias tags (v1.2 and v1)

3. **Version alias tags**:
   - `v1.2` always points to latest patch (v1.2.3)
   - `v1` always points to latest minor (v1.5.2)
   - Allows users to track latest releases

### Release Requirements

- Release tags must be on the correct release branch
- Version history must be linear (each version builds on previous)
- All CI checks must pass
- Code coverage must meet 90% threshold
- No security vulnerabilities in dependencies

### Shell Script Standards

All CI/CD shell scripts follow strict POSIX compliance standards:
- Pure POSIX sh (no bashisms)
- Use `test` command instead of `[` or `[[`
- Quote all parameter expansions
- Pass shellcheck validation

See [.kiro/steering/shell-scripts.md](.kiro/steering/shell-scripts.md) for complete standards.

## Recognition

Contributors will be recognized in:
- Git commit history
- Release notes
- Project documentation (future)

Thank you for contributing to Crusty!


---

## See Also

- [SYNTAX_PHILOSOPHY.md](.kiro/specs/crusty-compiler-phase1/SYNTAX_PHILOSOPHY.md) - Core principle: syntax-only transpilation
- [requirements.md](.kiro/specs/crusty-compiler-phase1/requirements.md) - Detailed feature requirements
- [design.md](.kiro/specs/crusty-compiler-phase1/design.md) - Architecture and component design
- [tasks.md](.kiro/specs/crusty-compiler-phase1/tasks.md) - Implementation plan and progress
- [README.md](README.md) - Project overview and quick start guide
