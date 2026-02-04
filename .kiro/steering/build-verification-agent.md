---
inclusion: manual
---

# Build Verification Agent

## Purpose

Verify that the Crusty compiler project builds successfully using Cargo after implementation and quality improvements. The build verification agent executes the Rust build process and applies fixes for common build errors when possible.

## Context

You have access to:
- **Modified code files**: All Rust (.rs) and Crusty (.crst) files changed during implementation
- **Build configuration**: Cargo.toml, Cargo.lock, build.rs
- **Implementation commit**: The commit message and changes from the implementation
- **Task details**: tasks.md, requirements.md, and design.md files
- **All Kiro tools**: File operations, git commands, Cargo build tools

## Instructions

### Step 1: Verify Cargo Configuration

Check that Cargo is properly configured:

```bash
# Verify Cargo.toml exists
test -f Cargo.toml || error "Cargo.toml not found"

# View project configuration
cat Cargo.toml
```

### Step 2: Run Cargo Build

Execute the Cargo build command:

```bash
# Build in debug mode (default)
cargo build

# Or build with release optimizations if needed
cargo build --release
```

**Build targets**:
- Library: `cargo build --lib`
- Binaries: `cargo build --bins`
- Specific binary: `cargo build --bin crustyc`
- All targets: `cargo build --all-targets`

### Step 3: Analyze Build Results

Review the build output:

**If build succeeds**:
- Verify artifacts in `target/debug/` or `target/release/`
- Check for warnings
- Proceed to Step 9 (no changes needed)

**If build fails**:
- Capture Cargo error output
- Categorize errors
- Proceed to Step 4

**Common Rust build errors**:
- **Syntax errors**: Invalid Rust syntax
- **Type errors**: Type mismatches, trait bounds
- **Borrow checker**: Lifetime/ownership violations
- **Missing dependencies**: Crates not in Cargo.toml
- **Feature errors**: Required features not enabled
- **Macro errors**: Macro expansion failures

### Step 4: Apply Automatic Fixes

For auto-fixable errors:

**Missing imports**:
```rust
// Error: cannot find type `HashMap` in this scope
// Fix: Add use statement
use std::collections::HashMap;
```

**Unused imports**:
```rust
// Warning: unused import
// Fix: Remove or prefix with underscore
use std::collections::HashMap; // Remove if unused
```

**Type annotations**:
```rust
// Error: type annotations needed
// Fix: Add explicit type
let value: i32 = parse_number();
```

**Dependency updates**:
```bash
# Update dependencies
cargo update <crate-name>
```

### Step 5: Re-run Build

After applying fixes:

```bash
cargo build
```

**If build succeeds**: Proceed to commit fixes
**If build fails**: Analyze remaining errors (max 2 attempts)

### Step 6: Handle No Errors Scenario

If build succeeds with no errors:

1. Verify build actually ran
2. Check artifacts were created
3. Report: "Build verification successful"
4. Do not create a commit
5. Exit successfully

### Step 7: Commit Build Fixes

If fixes were applied:

```bash
git add .
git commit -m "chore(<scope>): fix build errors for <context>"
```

**Examples**:
- `chore(parser): fix build errors for parser module`
- `chore(codegen): fix build errors for code generation`
- `chore(deps): fix build errors for dependency updates`

## Commit Format

```
chore(<scope>): fix build errors for <context>

<optional body>
- Fixed X build errors in Y files
- Added missing imports
- Updated dependencies
- Fixed type annotations

<optional footer>
Build system: Cargo <version>
```

## Success Criteria

1. ✅ Cargo build executed successfully
2. ✅ Build artifacts created in target/
3. ✅ Auto-fixable errors fixed
4. ✅ Changes reviewed and verified
5. ✅ No functional changes introduced
6. ✅ Commit message follows format (if fixes applied)
7. ✅ Remaining errors reported to user

## Error Handling

### Build Command Not Found

**Scenario**: Cargo is not installed

**Action**:
- Report error
- Suggest installation: https://rustup.rs/
- Exit without proceeding

### Build Fails with Syntax Errors

**Scenario**: Rust syntax errors prevent compilation

**Action**:
- Capture error output with file/line numbers
- Report to user (syntax errors require manual fix)
- Do not attempt auto-fix
- Exit without commit

### Build Fails with Type Errors

**Scenario**: Type system errors

**Action**:
- Attempt simple fixes (add type annotations)
- For complex errors, report to user
- Commit simple fixes if successful

### Dependency Errors

**Scenario**: Missing or conflicting dependencies

**Action**:
- Check Cargo.toml for missing crates
- Attempt `cargo update` for version conflicts
- Report unresolvable conflicts to user

## Notes

- **Build First**: Always verify build before other quality checks
- **No Functional Changes**: Build fixes should only resolve compilation issues
- **Cargo Warnings**: Pay attention to warnings - they often indicate issues
- **Clean Builds**: Consider `cargo clean` before building to ensure no stale artifacts
- **Feature Flags**: Be aware of feature-gated code that may not build without specific features
