# CI/CD Architecture

## Introduction

Crusty uses a modular CI/CD pipeline built on GitHub Actions. The architecture separates entry-point workflows (triggered by events) from reusable component workflows (performing specific checks), enabling parallel execution and code reuse.

## Rationale

A modular CI/CD design allows individual checks to run concurrently, reducing total pipeline time. Reusable workflows eliminate duplication and ensure consistent behavior across different trigger events (pull requests, pushes, releases).

## Pipeline Components

### Entry-Point Workflows
- Triggered by GitHub events (pull request, push to main, tag creation)
- Orchestrate component workflows in parallel where possible
- Define the overall pipeline structure

### Component Workflows (Reusable)
- Pre-commit validation (formatting, linting)
- Security audit (dependency vulnerability scanning)
- Test execution (unit tests, integration tests, property-based tests)
- Coverage reporting (line, branch, and function coverage)
- Cross-platform builds (Linux, macOS, Windows)
- Release packaging

## Execution Model

Component workflows run concurrently where there are no dependencies between them. The pipeline uses a fan-out/fan-in pattern:

1. Pre-commit checks run first (fast gate)
2. Tests, security audit, and cross-platform builds run in parallel
3. Coverage reporting aggregates results after tests complete
4. Release packaging runs only on tagged commits after all checks pass

## POSIX Shell Scripts

CI scripts use POSIX-compatible shell to ensure portability across Linux, macOS, and Windows (via Git Bash). This avoids bash-specific features that may not be available on all platforms.
