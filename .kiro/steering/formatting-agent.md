---
inclusion: manual
---

# Formatting Agent

## Purpose

Run `cargo fmt` (rustfmt) to ensure consistent Rust code style across the codebase. The formatting agent automatically applies Rust's standard formatting rules to all modified Rust source files.

## Context

You have access to:
- **Modified code files**: All Rust (.rs) files changed during implementation
- **Formatting configuration**: rustfmt.toml or .rustfmt.toml (if present)
- **Implementation commit**: The commit message and changes from the implementation
- **Task details**: tasks.md, requirements.md, and design.md files
- **All Kiro tools**: File operations, git commands, rustfmt

## Instructions

### Step 1: Verify Rustfmt is Available

Check that rustfmt is installed:

```bash
# Check rustfmt version
rustfmt --version

# If not installed, install via rustup
rustup component add rustfmt
```

### Step 2: Identify Modified Rust Files

Determine which Rust files were modified:

```bash
# View implementation commit
git log -1 --stat

# Get list of modified Rust files
git diff --name-only HEAD~1 | grep '\.rs$'
```

Focus formatting on:
- **Source files**: Files in src/
- **Test files**: Files in tests/
- **Build scripts**: build.rs
- **Examples**: Files in examples/

**Skip**:
- Generated files in target/
- External dependencies in .cargo/

### Step 3: Run Cargo Format

Execute rustfmt on all Rust files:

```bash
# Format all Rust files in the project
cargo fmt

# Check formatting without applying changes
cargo fmt -- --check
```

**Rustfmt behavior**:
- Formats according to Rust style guidelines
- Respects rustfmt.toml configuration if present
- Applies consistent indentation, spacing, and line breaks
- Organizes imports and removes trailing whitespace

### Step 4: Verify Formatting Results

After running cargo fmt, verify no issues remain:

```bash
# Check if formatting is complete
cargo fmt -- --check
```

**If no issues**: Proceed to commit fixes
**If issues remain**: Re-run cargo fmt

### Step 5: Review Changes

Review formatting changes:

```bash
# View all changes
git diff

# View changes by file
git diff --stat
```

**Verify**:
- Changes are style-only (no functional changes)
- Consistent indentation and spacing
- Proper line breaks and alignment
- Import organization

**Common rustfmt fixes**:
- Consistent indentation (4 spaces)
- Line length adjustments (default 100 chars)
- Trailing whitespace removal
- Consistent brace placement
- Import sorting and grouping
- Consistent spacing around operators

### Step 6: Handle No Changes Scenario

If cargo fmt made no changes:

1. Verify rustfmt actually ran
2. Report: "No formatting fixes needed - code already meets Rust style guidelines"
3. Do not create a commit
4. Exit successfully

### Step 7: Commit Formatting Fixes

If rustfmt made changes:

```bash
git add .
git commit -m "chore(<scope>): apply formatting fixes for <context>"
```

**Examples**:
- `chore(parser): apply formatting fixes for parser module`
- `chore(codegen): apply formatting fixes for code generation`
- `chore(tests): apply formatting fixes for test files`

**Scope guidelines**:
- Use the same scope as implementation commit
- Use module name (parser, codegen, semantic, etc.)
- Use "format" for project-wide formatting

## Commit Format

```
chore(<scope>): apply formatting fixes for <context>

<optional body>
- Formatted X files with cargo fmt
- Applied consistent indentation
- Fixed line length issues
- Organized imports

<optional footer>
Formatter: rustfmt <version>
```

## Success Criteria

1. ✅ Rustfmt executed successfully
2. ✅ All Rust files formatted according to style guide
3. ✅ Changes reviewed and verified as style-only
4. ✅ No functional changes introduced
5. ✅ Commit message follows format
6. ✅ Changes committed (or noted as not needed)

## Error Handling

### Rustfmt Not Installed

**Scenario**: rustfmt is not available

**Action**:
- Report error
- Suggest installation: `rustup component add rustfmt`
- Exit without proceeding

### Rustfmt Execution Fails

**Scenario**: cargo fmt fails to run

**Action**:
- Capture error output
- Check for syntax errors (rustfmt requires valid Rust)
- Report to user
- Do not proceed until rustfmt runs successfully

### Syntax Errors Prevent Formatting

**Scenario**: Rust files have syntax errors

**Action**:
- Report syntax errors with file/line numbers
- Explain that rustfmt requires valid Rust syntax
- Suggest fixing syntax errors first
- Exit without commit

### No Changes After Formatting

**Scenario**: cargo fmt runs but makes no changes

**Action**:
- Verify rustfmt ran (check output)
- Report: "No formatting fixes needed"
- Do not create a commit
- Exit successfully

### Rustfmt Configuration Issues

**Scenario**: rustfmt.toml has invalid configuration

**Action**:
- Report configuration error
- Suggest checking rustfmt.toml syntax
- Use default formatting if config is invalid
- Report to user

## Notes

- **Style Only**: Formatting changes code style, never behavior
- **Idempotent**: Running cargo fmt multiple times produces same result
- **Standard Style**: Rustfmt follows Rust's official style guidelines
- **Configuration**: Respect rustfmt.toml if present in project
- **Commit Separately**: Formatting fixes in separate commit for clean history
- **Pre-commit Integration**: Formatting is also checked by pre-commit hooks
- **CI/CD Integration**: Formatting is verified in CI/CD pipeline
