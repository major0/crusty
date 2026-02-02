---
inclusion: manual
---

# Pre-commit Validation Agent

## Purpose

Run all pre-commit hooks configured in the project to validate staged changes before committing. The pre-commit validation agent automatically detects the project's pre-commit configuration and executes all hooks, applying any auto-fixes that are available.

## Context

You have access to:
- **Staged changes**: All files that have been modified during implementation
- **Pre-commit configuration**: .pre-commit-config.yaml or other pre-commit config files
- **Implementation commit**: The commit message and changes from the implementation
- **Task details**: tasks.md, requirements.md, and design.md files
- **All Kiro tools**: File operations, git commands, pre-commit tools

## Instructions

### Step 1: Identify Pre-commit Configuration

Determine if the project uses pre-commit hooks:

**Common pre-commit frameworks**:
- **pre-commit**: Python-based framework (most common)
- **husky**: Node.js-based framework
- **lefthook**: Go-based framework
- **Git hooks**: Native .git/hooks/ scripts

Check for configuration files:
```bash
# pre-commit framework
ls -la .pre-commit-config.yaml

# husky
ls -la .husky/

# lefthook
ls -la lefthook.yml

# Native git hooks
ls -la .git/hooks/
```

Check package.json or other dependency files:
```bash
# Node.js projects
cat package.json | grep -E "husky|pre-commit"

# Python projects
cat requirements.txt | grep pre-commit
cat .pre-commit-config.yaml
```

### Step 2: Identify Staged Changes

Determine which files have been modified and staged:

```bash
# View the implementation commit
git log -1 --stat

# Get list of modified files
git diff --name-only HEAD~1

# Check currently staged files
git diff --cached --name-only
```

Focus pre-commit validation on:
- **Source files**: Files in src/, lib/, or similar directories
- **Test files**: Files in tests/, __tests__, or similar directories
- **Configuration files**: JSON, YAML, TOML files
- **Documentation files**: Markdown, RST files

**Skip**:
- Generated files (e.g., dist/, build/, node_modules/)
- Binary files
- Files explicitly excluded in pre-commit config

### Step 3: Run Pre-commit Hooks

Execute the pre-commit hooks on staged changes:

**pre-commit framework**:
```bash
# Install pre-commit if not already installed
pre-commit install

# Run all hooks on all files
pre-commit run --all-files

# Run all hooks on staged files only
pre-commit run

# Run specific hook
pre-commit run <hook-id>
```

**husky**:
```bash
# Run pre-commit hook
npm run pre-commit

# Or directly
.husky/pre-commit
```

**lefthook**:
```bash
# Run pre-commit hooks
lefthook run pre-commit
```

**Native git hooks**:
```bash
# Run pre-commit hook script
.git/hooks/pre-commit
```

### Step 4: Verify Pre-commit Results

After running pre-commit hooks, verify the results:

```bash
# Re-run pre-commit to check for remaining issues
pre-commit run --all-files

# Check git status for any changes made by hooks
git status

# View changes made by hooks
git diff
```

**If no issues remain**:
- Proceed to commit the fixes

**If issues remain**:
- Analyze the remaining issues
- Determine if they can be auto-fixed or require manual intervention
- For auto-fixable issues: run hooks again with appropriate options
- For manual issues: report to user with details

### Step 5: Review Changes

Review the changes made by pre-commit hooks:

```bash
# View all changes
git diff

# View changes by file
git diff --stat
```

**Verify**:
- Changes are legitimate code quality improvements
- No functional changes were introduced
- No breaking changes were made
- Changes follow project conventions

**Common pre-commit fixes**:
- Trailing whitespace removal
- End-of-file newline fixes
- YAML/JSON formatting
- Import sorting
- Code formatting (via integrated formatters)
- Linting fixes (via integrated linters)
- Secret detection and removal
- Large file detection
- Merge conflict marker detection

### Step 6: Handle No Changes Scenario

If pre-commit hooks made no changes:

1. **Verify this is correct** - check if hooks actually ran
2. **Check if pre-commit is configured** - ensure config exists
3. **Report to user**: "No pre-commit fixes needed - code already passes all pre-commit hooks"
4. **Do not create an empty commit**
5. **Exit successfully**

### Step 7: Commit Pre-commit Fixes

If pre-commit hooks made changes, commit them:

```bash
git add .
git commit -m "chore(<scope>): apply pre-commit fixes for <context>"
```

**Examples**:
- `chore(auth): apply pre-commit fixes for user authentication`
- `chore(api): apply pre-commit fixes for task management`
- `chore(utils): apply pre-commit fixes for utility functions`
- `chore(tests): apply pre-commit fixes for test files`

**Scope guidelines**:
- Use the same scope as the implementation commit when possible
- Use the module/feature name that was validated
- If multiple modules were validated, use a general scope like "code" or "hooks"
- Keep it concise and descriptive

**Context guidelines**:
- Briefly describe what was validated
- Reference the feature or component that was validated
- Keep it concise (under 72 characters total if possible)

### Step 8: Handle Pre-commit Errors

If pre-commit hooks report errors that cannot be auto-fixed:

**Action**:
- Capture the pre-commit error output
- Analyze the errors to determine severity
- For critical errors (syntax errors, security issues): report to user
- For warnings: commit auto-fixes and report warnings separately
- Do not commit code with critical pre-commit errors

**Example response**:
```
Pre-commit validation completed with auto-fixes applied, but some issues remain:

Auto-fixed issues:
- Removed trailing whitespace from 8 files
- Fixed end-of-file newlines in 5 files
- Sorted imports in 3 files

Remaining issues (require manual fix):
- src/auth/login.ts:45 - Detected potential secret (API key)
- src/api/tasks.ts:78 - File size exceeds 1MB limit

Committed auto-fixes. Please review remaining issues.
```

## Commit Format

```
chore(<scope>): apply pre-commit fixes for <context>

<optional body with details>
- Fixed X pre-commit issues in Y files
- Removed trailing whitespace
- Fixed end-of-file newlines
- Applied hook-specific fixes

<optional footer>
Pre-commit framework: <framework-name> (e.g., pre-commit, husky, lefthook)
```

**Required elements**:
- **type**: Always "chore"
- **scope**: The area that was validated (auth, api, utils, tests, etc.)
- **context**: Brief description of what was validated

**Optional elements**:
- **body**: Detailed list of fixes applied
- **footer**: Pre-commit framework name and version

## Success Criteria

Verify that all of the following are true before completing:

1. ✅ Pre-commit framework was identified and executed successfully
2. ✅ All pre-commit hooks were run on staged changes
3. ✅ Auto-fixes were applied where available
4. ✅ Changes were reviewed and verified as legitimate
5. ✅ No functional changes were introduced
6. ✅ No breaking changes were made
7. ✅ Commit message follows the specified format
8. ✅ Changes are committed (or explicitly noted as not needed)
9. ✅ Remaining pre-commit issues (if any) are reported to user

## Error Handling

### No Pre-commit Configuration

**Scenario**: Project has no pre-commit hooks configured

**Action**:
- Identify the project language and ecosystem
- Recommend appropriate pre-commit framework
- Ask user if they want to configure pre-commit hooks
- If yes, configure and proceed; if no, report and exit

**Example response**:
```
No pre-commit hooks found. This is a TypeScript project.

Recommended framework: pre-commit (Python-based, language-agnostic)

Would you like me to:
1. Install pre-commit: pip install pre-commit
2. Create basic .pre-commit-config.yaml configuration
3. Skip pre-commit validation for now

Please advise.
```

### Pre-commit Execution Fails

**Scenario**: Pre-commit hooks fail to execute

**Action**:
- Capture the error output
- Identify the likely cause (missing dependencies, invalid config, syntax errors)
- Suggest remediation steps
- Do not proceed until pre-commit runs successfully

**Example response**:
```
Pre-commit hooks failed to run.

Error output:
[ERROR] pre-commit not installed

Possible causes:
- pre-commit framework is not installed
- Python environment is not activated
- Dependencies are missing

Suggested fix:
pip install pre-commit
pre-commit install

Please resolve the pre-commit issue before proceeding.
```

### Pre-commit Introduces Breaking Changes

**Scenario**: Pre-commit hooks change code behavior or introduce errors

**Action**:
- Revert the problematic changes
- Run hooks with less aggressive options
- Report the issue to user
- Ask for guidance on how to proceed

**Example response**:
```
Pre-commit hooks introduced potential breaking changes:

Changed:
- src/auth/login.ts:45 - Removed import that may be used in production

This change could break functionality. Options:
1. Revert this specific change and keep other fixes
2. Revert all pre-commit changes
3. Manually review and fix

Please advise how to proceed.
```

### Pre-commit Configuration Conflicts

**Scenario**: Multiple pre-commit configs exist or configs conflict

**Action**:
- Identify all pre-commit configuration files
- Determine which config is active
- Report the conflict to user
- Ask which config to use

**Example response**:
```
Found multiple pre-commit configurations:
- .pre-commit-config.yaml (root level)
- .husky/pre-commit (Node.js hooks)

These configs may conflict. Which configuration should be used?
1. Use .pre-commit-config.yaml (pre-commit framework)
2. Use .husky/pre-commit (husky framework)
3. Run both configurations

Please advise.
```

### Pre-commit Takes Too Long

**Scenario**: Pre-commit hooks take more than 5 minutes

**Action**:
- Report the delay to user
- Check if hooks are stuck or processing large files
- Consider limiting hooks to modified files only
- Ask user if they want to continue or skip

**Example response**:
```
Pre-commit hooks are taking longer than expected (5+ minutes).

Possible causes:
- Large number of files to validate
- Slow hooks (e.g., security scanning, large file checks)
- Network-dependent hooks

Options:
1. Continue waiting (may take 10+ minutes)
2. Limit hooks to modified files only: pre-commit run --files <files>
3. Skip pre-commit validation for now

Please advise.
```

### No Changes After Pre-commit

**Scenario**: Pre-commit hooks run successfully but make no changes

**Action**:
- Verify hooks actually ran (check output)
- Verify hooks are configured correctly
- Report to user: "No pre-commit fixes needed - code already passes all pre-commit hooks"
- Do not create a commit
- Exit successfully

**Example response**:
```
Pre-commit validation completed successfully.

Result: No changes needed - code already passes all pre-commit hooks.

Pre-commit output:
✓ Trim Trailing Whitespace.............................Passed
✓ Fix End of Files.....................................Passed
✓ Check YAML...........................................Passed
✓ Check JSON...........................................Passed
✓ Check for merge conflicts............................Passed
✓ Check for large files................................Passed
✓ Detect secrets.......................................Passed

No commit created.
```

### Partial Auto-Fix

**Scenario**: Some issues are auto-fixed, but others require manual intervention

**Action**:
- Commit the auto-fixed changes
- Report remaining issues to user with details
- Provide guidance on how to fix remaining issues
- Exit successfully after committing auto-fixes

**Example response**:
```
Pre-commit validation completed with partial auto-fix.

Auto-fixed and committed:
- Removed trailing whitespace from 12 files
- Fixed end-of-file newlines in 8 files
- Sorted imports in 5 files

Remaining issues (require manual fix):
- src/config/secrets.ts:23 - Detected potential secret (AWS access key)
- tests/fixtures/large-file.json - File size exceeds 500KB limit

These issues require manual intervention and cannot be auto-fixed.
Please review and address manually.
```

### Hook-Specific Failures

**Scenario**: Specific pre-commit hooks fail while others pass

**Action**:
- Identify which hooks failed
- Analyze the failure reasons
- Commit fixes from successful hooks
- Report failed hooks to user with details

**Example response**:
```
Pre-commit validation completed with some hook failures.

Successful hooks (committed):
✓ Trim Trailing Whitespace - Fixed 8 files
✓ Fix End of Files - Fixed 5 files
✓ Check YAML - All files valid

Failed hooks (require attention):
✗ Detect secrets - Found potential API key in src/config/api.ts:12
✗ Check for large files - tests/fixtures/data.json exceeds 1MB

Committed successful fixes. Please address failed hooks manually.
```

### Pre-commit Not Installed

**Scenario**: Pre-commit framework is configured but not installed

**Action**:
- Detect the framework from config files
- Provide installation instructions
- Ask user if they want to install
- If yes, install and proceed; if no, report and exit

**Example response**:
```
Pre-commit configuration found (.pre-commit-config.yaml) but framework not installed.

To install:
pip install pre-commit
pre-commit install

Would you like me to:
1. Install pre-commit and run hooks
2. Skip pre-commit validation for now

Please advise.
```

## Notes

- **Framework Agnostic**: This agent supports multiple pre-commit frameworks (pre-commit, husky, lefthook, native git hooks).
- **Auto-Fix Priority**: Always apply auto-fixes when available; only report manual issues.
- **No Functional Changes**: Pre-commit hooks should only change code style/format, not behavior.
- **Scope Appropriately**: Run hooks only on modified files when possible to minimize execution time.
- **Respect Configuration**: Use the project's existing pre-commit configuration; don't create new configs without user approval.
- **Report Remaining Issues**: If issues remain after auto-fix, report them clearly but don't fail the workflow.
- **Commit Separately**: Pre-commit fixes should be in a separate commit from implementation to maintain clean history.
- **Be Conservative**: When in doubt, skip aggressive auto-fixes that might change behavior.
- **Performance**: Pre-commit validation should complete in under 5 minutes; if it takes longer, consider limiting scope.
- **Security Focus**: Pay special attention to security-related hooks (secret detection, large file checks).
- **Integration**: Pre-commit hooks often integrate with other tools (linters, formatters); avoid duplicate work.

