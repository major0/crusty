---
inclusion: manual
---

# Pre-commit Validation Agent

## Purpose

Run all pre-commit hooks configured in `.pre-commit-config.yaml` to validate staged changes before committing. The pre-commit validation agent executes all hooks including Crusty syntax checks, Rust formatting, Clippy linting, and ShellCheck.

## Context

You have access to:
- **Staged changes**: All files modified during implementation
- **Pre-commit configuration**: .pre-commit-config.yaml
- **Implementation commit**: The commit message and changes from the implementation
- **Task details**: tasks.md, requirements.md, and design.md files
- **All Kiro tools**: File operations, git commands, pre-commit framework

## Instructions

### Step 1: Verify Pre-commit is Installed

Check that pre-commit is installed:

```bash
# Check pre-commit version
pre-commit --version

# If not installed, install via pip
pip install pre-commit

# Install hooks
pre-commit install
```

### Step 2: Identify Pre-commit Hooks

This project uses the following pre-commit hooks:

**Crusty Syntax Check**:
- Hook ID: `crustyc-syntax`
- Validates Crusty (.crst) files for syntax errors
- Script: `.github/scripts/check-crusty-syntax.sh`

**Cargo Format Check**:
- Hook ID: `cargo-fmt`
- Checks Rust code formatting with rustfmt
- Command: `cargo fmt -- --check`

**Cargo Clippy**:
- Hook ID: `cargo-clippy`
- Runs Clippy linter on Rust code
- Command: `cargo clippy -- -D warnings`

**ShellCheck**:
- Hook ID: `shellcheck`
- Validates shell scripts for errors
- Command: `shellcheck`

### Step 3: Run Pre-commit Hooks

Execute all pre-commit hooks:

```bash
# Run all hooks on all files
pre-commit run --all-files

# Run all hooks on staged files only
pre-commit run

# Run specific hook
pre-commit run crustyc-syntax
pre-commit run cargo-fmt
pre-commit run cargo-clippy
pre-commit run shellcheck
```

### Step 4: Verify Pre-commit Results

After running hooks, verify results:

```bash
# Re-run to check for remaining issues
pre-commit run --all-files

# Check git status for any changes made by hooks
git status

# View changes made by hooks
git diff
```

**If no issues remain**: Proceed to commit fixes
**If issues remain**: Analyze and determine if auto-fixable

### Step 5: Review Changes

Review changes made by pre-commit hooks:

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
- Changes follow project conventions

**Common pre-commit fixes**:
- Trailing whitespace removal
- End-of-file newline fixes
- Rust code formatting
- Clippy auto-fixes
- Shell script corrections

### Step 6: Handle No Changes Scenario

If pre-commit hooks made no changes:

1. Verify hooks actually ran
2. Report: "No pre-commit fixes needed - code passes all pre-commit hooks"
3. Do not create a commit
4. Exit successfully

### Step 7: Commit Pre-commit Fixes

If pre-commit hooks made changes:

```bash
git add .
git commit -m "chore(<scope>): apply pre-commit fixes for <context>"
```

**Examples**:
- `chore(parser): apply pre-commit fixes for parser module`
- `chore(codegen): apply pre-commit fixes for code generation`
- `chore(scripts): apply pre-commit fixes for shell scripts`

**Scope guidelines**:
- Use the same scope as implementation commit
- Use module name for Rust code fixes
- Use "scripts" for shell script fixes
- Use "crusty" for Crusty syntax fixes

## Commit Format

```
chore(<scope>): apply pre-commit fixes for <context>

<optional body>
- Fixed X pre-commit issues in Y files
- Applied Rust formatting
- Fixed Clippy warnings
- Fixed shell script issues
- Fixed Crusty syntax issues

<optional footer>
Pre-commit framework: <version>
```

## Success Criteria

1. ✅ Pre-commit framework executed successfully
2. ✅ All pre-commit hooks run on staged changes
3. ✅ Auto-fixes applied where available
4. ✅ Changes reviewed and verified
5. ✅ No functional changes introduced
6. ✅ Commit message follows format
7. ✅ Changes committed (or noted as not needed)
8. ✅ Remaining issues reported to user

## Error Handling

### Pre-commit Not Installed

**Scenario**: Pre-commit framework is not installed

**Action**:
- Report error
- Suggest installation: `pip install pre-commit`
- Exit without proceeding

### Pre-commit Execution Fails

**Scenario**: Pre-commit hooks fail to execute

**Action**:
- Capture error output
- Identify likely cause
- Suggest remediation steps
- Do not proceed until hooks run successfully

### Crusty Syntax Errors

**Scenario**: Crusty syntax check fails

**Action**:
- Capture syntax error output with file/line numbers
- Report to user (syntax errors require manual fix)
- Do not attempt auto-fix
- Exit without commit

### Cargo Format Failures

**Scenario**: Rust code formatting check fails

**Action**:
- Run `cargo fmt` to apply formatting
- Re-run pre-commit hooks
- Commit formatting fixes

### Cargo Clippy Failures

**Scenario**: Clippy linting fails

**Action**:
- Run `cargo clippy --fix` to apply auto-fixes
- Re-run pre-commit hooks
- Report remaining issues to user
- Commit auto-fixes

### ShellCheck Failures

**Scenario**: Shell script validation fails

**Action**:
- Capture ShellCheck errors
- Apply fixes for common issues
- Report complex issues to user
- Commit auto-fixes

### No Changes After Pre-commit

**Scenario**: Pre-commit hooks run but make no changes

**Action**:
- Verify hooks ran (check output)
- Report: "No pre-commit fixes needed"
- Do not create a commit
- Exit successfully

**Example response**:
```
Pre-commit validation completed successfully.

Result: No changes needed - code passes all pre-commit hooks.

Pre-commit output:
✓ Crusty Syntax Check.............................Passed
✓ Cargo Format Check..............................Passed
✓ Cargo Clippy....................................Passed
✓ ShellCheck......................................Passed

No commit created.
```

### Partial Auto-Fix

**Scenario**: Some issues auto-fixed, others require manual intervention

**Action**:
- Commit auto-fixed changes
- Report remaining issues to user
- Provide guidance on manual fixes
- Exit successfully after committing auto-fixes

**Example response**:
```
Pre-commit validation completed with partial auto-fix.

Auto-fixed and committed:
- Applied Rust formatting to 8 files
- Fixed 5 Clippy warnings
- Fixed 2 shell script issues

Remaining issues (require manual fix):
- src/parser.crst:45 - Crusty syntax error: unexpected token
- .github/scripts/deploy.sh:23 - ShellCheck SC2086: Quote to prevent word splitting

These issues require manual intervention.
Please review and address manually.
```

## Notes

- **Framework Agnostic**: This agent uses the pre-commit framework configured in .pre-commit-config.yaml
- **Auto-Fix Priority**: Always apply auto-fixes when available
- **No Functional Changes**: Pre-commit hooks should only change style/format, not behavior
- **Scope Appropriately**: Run hooks only on modified files when possible
- **Respect Configuration**: Use project's existing pre-commit configuration
- **Report Remaining Issues**: If issues remain after auto-fix, report clearly
- **Commit Separately**: Pre-commit fixes in separate commit for clean history
- **Be Conservative**: Skip aggressive auto-fixes that might change behavior
- **Performance**: Pre-commit should complete in under 5 minutes
- **Security Focus**: Pay attention to security-related hooks
- **Integration**: Pre-commit hooks integrate with other tools (rustfmt, clippy, shellcheck)
- **CI/CD Integration**: Pre-commit hooks are also run in CI/CD pipeline
