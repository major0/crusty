---
inclusion: manual
---

# Linting Agent

## Purpose

Run `cargo clippy` to ensure Rust code quality standards are met. The linting agent automatically detects and applies fixes for common code quality issues using Clippy, Rust's official linter.

## Context

You have access to:
- **Modified code files**: All Rust (.rs) files changed during implementation
- **Linting configuration**: clippy.toml or .clippy.toml (if present)
- **Implementation commit**: The commit message and changes from the implementation
- **Task details**: tasks.md, requirements.md, and design.md files
- **All Kiro tools**: File operations, git commands, Clippy

## Instructions

### Step 1: Verify Clippy is Available

Check that Clippy is installed:

```bash
# Check clippy version
cargo clippy --version

# If not installed, install via rustup
rustup component add clippy
```

### Step 2: Identify Modified Rust Files

Determine which Rust files were modified:

```bash
# View implementation commit
git log -1 --stat

# Get list of modified Rust files
git diff --name-only HEAD~1 | grep '\.rs$'
```

Focus linting on:
- **Source files**: Files in src/
- **Test files**: Files in tests/
- **Build scripts**: build.rs
- **Examples**: Files in examples/

**Skip**:
- Generated files in target/
- External dependencies

### Step 3: Run Clippy with Auto-Fix

Execute Clippy with automatic fixes:

```bash
# Run clippy with auto-fix
cargo clippy --fix --allow-dirty --allow-staged

# Run clippy without fix to see remaining issues
cargo clippy -- -D warnings
```

**Clippy lint categories**:
- **Correctness**: Potential bugs and logic errors
- **Suspicious**: Code that looks wrong but may be intentional
- **Complexity**: Unnecessarily complex code
- **Perf**: Performance issues
- **Style**: Style violations
- **Pedantic**: Extra-strict lints (opt-in)
- **Restriction**: Lints for specific coding standards (opt-in)

### Step 4: Verify Linting Results

After running clippy --fix, check for remaining issues:

```bash
# Check for remaining warnings/errors
cargo clippy -- -D warnings
```

**If no issues remain**: Proceed to commit fixes
**If issues remain**: Analyze and determine if auto-fixable

### Step 5: Review Changes

Review changes made by Clippy:

```bash
# View all changes
git diff

# View changes by file
git diff --stat
```

**Verify**:
- Changes are legitimate code quality improvements
- No functional changes introduced
- No breaking changes made
- Changes follow Rust idioms

**Common Clippy fixes**:
- Removing unused imports and variables
- Simplifying boolean expressions
- Using iterator methods instead of loops
- Removing redundant clones
- Using `if let` instead of `match`
- Removing unnecessary `return` statements
- Using `&str` instead of `&String`
- Deriving traits instead of manual implementation

### Step 6: Handle No Changes Scenario

If Clippy made no changes:

1. Verify Clippy actually ran
2. Report: "No linting fixes needed - code meets Clippy standards"
3. Do not create a commit
4. Exit successfully

### Step 7: Commit Linting Fixes

If Clippy made changes:

```bash
git add .
git commit -m "chore(<scope>): apply linting fixes for <context>"
```

**Examples**:
- `chore(parser): apply linting fixes for parser module`
- `chore(codegen): apply linting fixes for code generation`
- `chore(tests): apply linting fixes for test files`

**Scope guidelines**:
- Use the same scope as implementation commit
- Use module name (parser, codegen, semantic, etc.)
- Use "lint" for project-wide linting

## Commit Format

```
chore(<scope>): apply linting fixes for <context>

<optional body>
- Fixed X Clippy warnings in Y files
- Removed unused imports
- Simplified boolean expressions
- Applied Rust idioms

<optional footer>
Linter: Clippy <version>
```

## Success Criteria

1. ✅ Clippy executed successfully
2. ✅ Auto-fixable issues fixed
3. ✅ Changes reviewed and verified
4. ✅ No functional changes introduced
5. ✅ Commit message follows format
6. ✅ Changes committed (or noted as not needed)
7. ✅ Remaining issues reported to user

## Error Handling

### Clippy Not Installed

**Scenario**: Clippy is not available

**Action**:
- Report error
- Suggest installation: `rustup component add clippy`
- Exit without proceeding

### Clippy Execution Fails

**Scenario**: cargo clippy fails to run

**Action**:
- Capture error output
- Check for compilation errors (Clippy requires code to compile)
- Report to user
- Do not proceed until Clippy runs successfully

### Compilation Errors Prevent Linting

**Scenario**: Code doesn't compile

**Action**:
- Report compilation errors
- Explain that Clippy requires code to compile first
- Suggest fixing compilation errors
- Exit without commit

### Clippy Warnings Cannot Be Auto-Fixed

**Scenario**: Some warnings require manual intervention

**Action**:
- Commit auto-fixed changes
- Report remaining warnings to user
- Provide guidance on manual fixes
- Exit successfully after committing auto-fixes

**Example response**:
```
Linting completed with auto-fixes applied, but some issues remain:

Auto-fixed issues:
- Removed 5 unused imports
- Simplified 3 boolean expressions
- Applied 2 iterator improvements

Remaining issues (require manual fix):
- src/parser.rs:45 - Complexity too high (consider refactoring)
- src/codegen.rs:78 - Potential panic in unwrap() (use proper error handling)

Committed auto-fixes. Please review remaining issues.
```

### No Changes After Linting

**Scenario**: Clippy runs but makes no changes

**Action**:
- Verify Clippy ran (check output)
- Report: "No linting fixes needed"
- Do not create a commit
- Exit successfully

## Notes

- **Auto-Fix Only**: Focus on auto-fixable issues; report manual issues
- **No Functional Changes**: Linting should only improve code quality, not behavior
- **Rust Idioms**: Clippy enforces Rust best practices and idioms
- **Commit Separately**: Linting fixes in separate commit for clean history
- **Pre-commit Integration**: Clippy is also checked by pre-commit hooks
- **CI/CD Integration**: Clippy is verified in CI/CD pipeline with `-D warnings`
- **Pedantic Lints**: Project may enable pedantic lints for stricter checking
- **Allow Directives**: Use `#[allow(clippy::lint_name)]` sparingly and with justification
