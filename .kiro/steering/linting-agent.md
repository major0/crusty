---
inclusion: manual
---

# Linting Agent

## Purpose

Run ESLint, TSLint, or other language-specific linters with auto-fix enabled to ensure code quality standards are met. The linting agent automatically detects the project's linting configuration and applies fixes for common code quality issues.

## Context

You have access to:
- **Modified code files**: All files changed during implementation
- **Linting configuration**: .eslintrc, tslint.json, or other linter config files
- **Implementation commit**: The commit message and changes from the implementation
- **Task details**: tasks.md, requirements.md, and design.md files
- **All Kiro tools**: File operations, git commands, linting tools

## Instructions

### Step 1: Identify Linting Tools

Determine which linting tools are configured for the project:

**Common linters by language**:
- **JavaScript/TypeScript**: ESLint, TSLint (deprecated), Standard
- **Python**: pylint, flake8, ruff
- **Rust**: clippy
- **Java**: Checkstyle, PMD, SpotBugs
- **Go**: golint, staticcheck
- **Ruby**: RuboCop
- **PHP**: PHP_CodeSniffer, PHPStan

Check for configuration files:
```bash
# JavaScript/TypeScript
ls -la .eslintrc* eslint.config.* .eslintrc.json .eslintrc.js .eslintrc.yml

# Python
ls -la .pylintrc setup.cfg pyproject.toml .flake8

# Rust
ls -la clippy.toml .clippy.toml

# Java
ls -la checkstyle.xml pmd.xml

# Go
ls -la .golangci.yml

# Ruby
ls -la .rubocop.yml
```

Check package.json or other dependency files for linter packages:
```bash
# JavaScript/TypeScript
cat package.json | grep -E "eslint|tslint"

# Python
cat requirements.txt | grep -E "pylint|flake8|ruff"

# Rust
cat Cargo.toml | grep clippy
```

### Step 2: Identify Modified Files

Determine which files were modified during implementation:

```bash
# View the implementation commit
git log -1 --stat

# Get list of modified files
git diff --name-only HEAD~1
```

Focus linting on:
- **Source files**: Files in src/, lib/, or similar directories
- **Test files**: Files in tests/, __tests__, or similar directories
- **Configuration files**: Only if they have linting rules

**Skip**:
- Generated files (e.g., dist/, build/, node_modules/)
- Binary files
- Files explicitly excluded in linter config

### Step 3: Run Linter with Auto-Fix

Execute the linter with auto-fix enabled on modified files:

**JavaScript/TypeScript (ESLint)**:
```bash
# Fix all files
npx eslint . --fix

# Fix specific files
npx eslint src/**/*.ts --fix

# Fix only modified files
git diff --name-only HEAD~1 | grep -E '\.(js|ts|jsx|tsx)$' | xargs npx eslint --fix
```

**Python (pylint/flake8/ruff)**:
```bash
# Ruff (supports auto-fix)
ruff check --fix .

# Flake8 (no auto-fix, but can report issues)
flake8 .

# Pylint (no auto-fix, but can report issues)
pylint src/
```

**Rust (clippy)**:
```bash
# Run clippy with auto-fix
cargo clippy --fix --allow-dirty --allow-staged

# Run clippy without fix (to see remaining issues)
cargo clippy -- -D warnings
```

**Go (golangci-lint)**:
```bash
# Run with auto-fix
golangci-lint run --fix

# Run on specific files
golangci-lint run --fix ./src/...
```

**Ruby (RuboCop)**:
```bash
# Run with auto-fix
rubocop -a

# Run with aggressive auto-fix
rubocop -A
```

### Step 4: Verify Linting Results

After running the linter with auto-fix, verify the results:

```bash
# Re-run linter without auto-fix to check for remaining issues
npx eslint .
# or
ruff check .
# or
cargo clippy
```

**If no issues remain**:
- Proceed to commit the fixes

**If issues remain**:
- Analyze the remaining issues
- Determine if they can be auto-fixed or require manual intervention
- For auto-fixable issues: run linter again with more aggressive options
- For manual issues: report to user with details

### Step 5: Review Changes

Review the changes made by the linter:

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

**Common linting fixes**:
- Removing unused imports
- Fixing indentation and spacing
- Adding missing semicolons
- Removing trailing whitespace
- Fixing quote style (single vs double)
- Reordering imports
- Removing unused variables
- Fixing naming conventions

### Step 6: Handle No Changes Scenario

If the linter made no changes:

1. **Verify this is correct** - check if linter actually ran
2. **Check if linter is configured** - ensure linter config exists
3. **Report to user**: "No linting fixes needed - code already meets linting standards"
4. **Do not create an empty commit**
5. **Exit successfully**

### Step 7: Commit Linting Fixes

If the linter made changes, commit them:

```bash
git add .
git commit -m "chore(<scope>): apply linting fixes for <context>"
```

**Examples**:
- `chore(auth): apply linting fixes for user authentication`
- `chore(api): apply linting fixes for task management`
- `chore(utils): apply linting fixes for utility functions`
- `chore(tests): apply linting fixes for test files`

**Scope guidelines**:
- Use the same scope as the implementation commit when possible
- Use the module/feature name that was linted
- If multiple modules were linted, use a general scope like "code" or "lint"
- Keep it concise and descriptive

**Context guidelines**:
- Briefly describe what was linted
- Reference the feature or component that was linted
- Keep it concise (under 72 characters total if possible)

### Step 8: Handle Linting Errors

If the linter reports errors that cannot be auto-fixed:

**Action**:
- Capture the linting error output
- Analyze the errors to determine severity
- For critical errors (syntax errors, type errors): report to user
- For warnings: commit auto-fixes and report warnings separately
- Do not commit code with critical linting errors

**Example response**:
```
Linting completed with auto-fixes applied, but some issues remain:

Auto-fixed issues:
- Removed 5 unused imports
- Fixed 12 indentation issues
- Added 3 missing semicolons

Remaining issues (require manual fix):
- src/auth/login.ts:45 - 'userId' is assigned but never used
- src/api/tasks.ts:78 - Unexpected console.log statement

Committed auto-fixes. Please review remaining issues.
```

## Commit Format

```
chore(<scope>): apply linting fixes for <context>

<optional body with details>
- Fixed X linting issues in Y files
- Removed unused imports
- Fixed indentation and spacing
- Applied code style conventions

<optional footer>
Linter: <linter-name> (e.g., ESLint, Clippy, RuboCop)
```

**Required elements**:
- **type**: Always "chore"
- **scope**: The area that was linted (auth, api, utils, tests, etc.)
- **context**: Brief description of what was linted

**Optional elements**:
- **body**: Detailed list of fixes applied
- **footer**: Linter name and version

## Success Criteria

Verify that all of the following are true before completing:

1. ✅ Linting tool was identified and executed successfully
2. ✅ Auto-fix was enabled and applied to modified files
3. ✅ Changes were reviewed and verified as legitimate
4. ✅ No functional changes were introduced
5. ✅ No breaking changes were made
6. ✅ Commit message follows the specified format
7. ✅ Changes are committed (or explicitly noted as not needed)
8. ✅ Remaining linting issues (if any) are reported to user

## Error Handling

### No Linter Configured

**Scenario**: Project has no linting tool configured

**Action**:
- Identify the project language
- Recommend appropriate linter
- Ask user if they want to configure linting
- If yes, configure and proceed; if no, report and exit

**Example response**:
```
No linting tool found. This is a TypeScript project.

Recommended linter: ESLint

Would you like me to:
1. Install ESLint: npm install --save-dev eslint @typescript-eslint/parser @typescript-eslint/eslint-plugin
2. Create basic .eslintrc.json configuration
3. Skip linting for now

Please advise.
```

### Linter Execution Fails

**Scenario**: Linter command fails to execute

**Action**:
- Capture the error output
- Identify the likely cause (missing dependencies, invalid config, syntax errors)
- Suggest remediation steps
- Do not proceed until linter runs successfully

**Example response**:
```
Linter failed to run.

Error output:
Error: Cannot find module 'eslint'

Possible causes:
- ESLint is not installed
- node_modules directory is missing
- Package dependencies are not installed

Suggested fix:
npm install

Please resolve the linter issue before proceeding.
```

### Linter Introduces Breaking Changes

**Scenario**: Auto-fix changes code behavior or introduces errors

**Action**:
- Revert the problematic changes
- Run linter with less aggressive options
- Report the issue to user
- Ask for guidance on how to proceed

**Example response**:
```
Linter auto-fix introduced potential breaking changes:

Changed:
- src/auth/login.ts:45 - Removed variable that may be used in production

This change could break functionality. Options:
1. Revert this specific change and keep other fixes
2. Revert all linting changes
3. Manually review and fix

Please advise how to proceed.
```

### Linter Configuration Conflicts

**Scenario**: Multiple linter configs exist or configs conflict

**Action**:
- Identify all linter configuration files
- Determine which config is active
- Report the conflict to user
- Ask which config to use

**Example response**:
```
Found multiple ESLint configurations:
- .eslintrc.json (root level)
- .eslintrc.js (in src/ directory)

These configs may conflict. Which configuration should be used?
1. Use .eslintrc.json (root level)
2. Use .eslintrc.js (src/ directory)
3. Merge configurations

Please advise.
```

### Linter Takes Too Long

**Scenario**: Linting takes more than 5 minutes

**Action**:
- Report the delay to user
- Check if linter is stuck or processing large files
- Consider limiting linting to modified files only
- Ask user if they want to continue or skip

**Example response**:
```
Linting is taking longer than expected (5+ minutes).

Possible causes:
- Large number of files to lint
- Complex linting rules
- Slow file system

Options:
1. Continue waiting (may take 10+ minutes)
2. Limit linting to modified files only
3. Skip linting for now

Please advise.
```

### No Changes After Linting

**Scenario**: Linter runs successfully but makes no changes

**Action**:
- Verify linter actually ran (check output)
- Verify linter config is not empty
- Report to user: "No linting fixes needed - code already meets linting standards"
- Do not create a commit
- Exit successfully

**Example response**:
```
Linting completed successfully.

Result: No changes needed - code already meets linting standards.

Linter output:
✓ 45 files checked
✓ 0 errors
✓ 0 warnings
✓ 0 fixable issues

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
Linting completed with partial auto-fix.

Auto-fixed and committed:
- Removed 8 unused imports
- Fixed 15 indentation issues
- Applied consistent quote style

Remaining issues (require manual fix):
- src/auth/login.ts:45 - Complexity too high (complexity: 15, max: 10)
- src/api/tasks.ts:78 - Function too long (lines: 85, max: 50)

These issues require refactoring and cannot be auto-fixed.
Please review and address manually.
```

## Notes

- **Auto-Fix Only**: This agent focuses on auto-fixable linting issues. Manual issues should be reported but not block the workflow.
- **No Functional Changes**: Linting should only change code style, not behavior. Always verify changes don't affect functionality.
- **Scope Appropriately**: Lint only modified files when possible to minimize changes and reduce execution time.
- **Respect Configuration**: Use the project's existing linter configuration. Don't create new configs without user approval.
- **Report Remaining Issues**: If linting issues remain after auto-fix, report them clearly but don't fail the workflow.
- **Commit Separately**: Linting fixes should be in a separate commit from implementation to maintain clean history.
- **Be Conservative**: When in doubt, skip aggressive auto-fixes that might change behavior.
- **Performance**: Linting should complete in under 5 minutes. If it takes longer, consider limiting scope.
