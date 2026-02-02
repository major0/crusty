---
inclusion: manual
---

# Formatting Agent

## Purpose

Run Prettier or other language-specific code formatters with auto-fix enabled to ensure consistent code style across the codebase. The formatting agent automatically detects the project's formatting configuration and applies fixes for code style inconsistencies.

## Context

You have access to:
- **Modified code files**: All files changed during implementation
- **Formatting configuration**: .prettierrc, .editorconfig, or other formatter config files
- **Implementation commit**: The commit message and changes from the implementation
- **Task details**: tasks.md, requirements.md, and design.md files
- **All Kiro tools**: File operations, git commands, formatting tools

## Instructions

### Step 1: Identify Formatting Tools

Determine which formatting tools are configured for the project:

**Common formatters by language**:
- **JavaScript/TypeScript**: Prettier, dprint
- **Python**: black, autopep8, yapf
- **Rust**: rustfmt
- **Java**: google-java-format, prettier-java
- **Go**: gofmt, goimports
- **Ruby**: RuboCop (with --auto-correct-all)
- **PHP**: PHP-CS-Fixer
- **C/C++**: clang-format

Check for configuration files:
```bash
# JavaScript/TypeScript
ls -la .prettierrc* prettier.config.* .prettierrc.json .prettierrc.js .prettierrc.yml

# Python
ls -la pyproject.toml setup.cfg .black .yapfrc

# Rust
ls -la rustfmt.toml .rustfmt.toml

# Java
ls -la .google-java-format.xml

# Go
# gofmt uses no config file

# Ruby
ls -la .rubocop.yml

# C/C++
ls -la .clang-format
```

Check package.json or other dependency files for formatter packages:
```bash
# JavaScript/TypeScript
cat package.json | grep -E "prettier|dprint"

# Python
cat requirements.txt | grep -E "black|autopep8|yapf"

# Rust
cat Cargo.toml | grep rustfmt
```

### Step 2: Identify Modified Files

Determine which files were modified during implementation:

```bash
# View the implementation commit
git log -1 --stat

# Get list of modified files
git diff --name-only HEAD~1
```

Focus formatting on:
- **Source files**: Files in src/, lib/, or similar directories
- **Test files**: Files in tests/, __tests__, or similar directories
- **Configuration files**: JSON, YAML, TOML files

**Skip**:
- Generated files (e.g., dist/, build/, node_modules/)
- Binary files
- Files explicitly excluded in formatter config (e.g., .prettierignore)

### Step 3: Run Formatter with Auto-Fix

Execute the formatter with auto-fix enabled on modified files:

**JavaScript/TypeScript (Prettier)**:
```bash
# Format all files
npx prettier --write .

# Format specific files
npx prettier --write "src/**/*.{ts,tsx,js,jsx}"

# Format only modified files
git diff --name-only HEAD~1 | grep -E '\.(js|ts|jsx|tsx|json|md|yml|yaml)$' | xargs npx prettier --write
```

**Python (black)**:
```bash
# Format all files
black .

# Format specific directory
black src/

# Format only modified files
git diff --name-only HEAD~1 | grep -E '\.py$' | xargs black
```

**Rust (rustfmt)**:
```bash
# Format all files in project
cargo fmt

# Format specific files
rustfmt src/main.rs

# Check formatting without applying
cargo fmt -- --check
```

**Go (gofmt/goimports)**:
```bash
# Format all files
gofmt -w .

# Format with import organization
goimports -w .

# Format specific files
gofmt -w src/
```

**Java (google-java-format)**:
```bash
# Format all Java files
java -jar google-java-format.jar --replace $(find . -name "*.java")

# Format specific directory
java -jar google-java-format.jar --replace src/**/*.java
```

**Ruby (RuboCop)**:
```bash
# Format with auto-correct
rubocop --auto-correct-all

# Format specific files
rubocop --auto-correct-all app/
```

**C/C++ (clang-format)**:
```bash
# Format all files
find . -name "*.cpp" -o -name "*.h" | xargs clang-format -i

# Format specific files
clang-format -i src/*.cpp
```

### Step 4: Verify Formatting Results

After running the formatter, verify the results:

```bash
# Re-run formatter in check mode to verify no issues remain
npx prettier --check .
# or
black --check .
# or
cargo fmt -- --check
# or
gofmt -l .
```

**If no issues remain**:
- Proceed to commit the fixes

**If issues remain**:
- Analyze the remaining issues
- Re-run formatter with appropriate options
- Report any files that cannot be formatted

### Step 5: Review Changes

Review the changes made by the formatter:

```bash
# View all changes
git diff

# View changes by file
git diff --stat
```

**Verify**:
- Changes are only style/formatting (no functional changes)
- No breaking changes were made
- Changes follow project conventions
- All files are properly formatted

**Common formatting fixes**:
- Consistent indentation (spaces vs tabs)
- Line length adjustments
- Trailing whitespace removal
- Consistent quote style
- Bracket placement
- Import ordering
- Blank line consistency
- Semicolon consistency

### Step 6: Handle No Changes Scenario

If the formatter made no changes:

1. **Verify this is correct** - check if formatter actually ran
2. **Check if formatter is configured** - ensure formatter config exists
3. **Report to user**: "No formatting fixes needed - code already meets formatting standards"
4. **Do not create an empty commit**
5. **Exit successfully**

### Step 7: Commit Formatting Fixes

If the formatter made changes, commit them:

```bash
git add .
git commit -m "chore(<scope>): apply formatting fixes for <context>"
```

**Examples**:
- `chore(auth): apply formatting fixes for user authentication`
- `chore(api): apply formatting fixes for task management`
- `chore(utils): apply formatting fixes for utility functions`
- `chore(tests): apply formatting fixes for test files`

**Scope guidelines**:
- Use the same scope as the implementation commit when possible
- Use the module/feature name that was formatted
- If multiple modules were formatted, use a general scope like "code" or "format"
- Keep it concise and descriptive

**Context guidelines**:
- Briefly describe what was formatted
- Reference the feature or component that was formatted
- Keep it concise (under 72 characters total if possible)

### Step 8: Handle Formatting Errors

If the formatter reports errors that cannot be auto-fixed:

**Action**:
- Capture the formatting error output
- Analyze the errors to determine cause
- For syntax errors: report to user (formatter cannot fix invalid code)
- For configuration errors: report to user with suggested fixes
- Do not commit code with formatting errors

**Example response**:
```
Formatting completed with auto-fixes applied, but some issues remain:

Auto-fixed issues:
- Reformatted 15 files
- Fixed indentation in 8 files
- Adjusted line lengths in 5 files

Remaining issues (require manual fix):
- src/auth/login.ts:45 - Syntax error prevents formatting
- src/api/tasks.ts - File exceeds maximum line length even after formatting

Committed auto-fixes. Please review remaining issues.
```

## Commit Format

```
chore(<scope>): apply formatting fixes for <context>

<optional body with details>
- Formatted X files
- Applied consistent indentation
- Fixed line length issues
- Removed trailing whitespace

<optional footer>
Formatter: <formatter-name> (e.g., Prettier, black, rustfmt)
```

**Required elements**:
- **type**: Always "chore"
- **scope**: The area that was formatted (auth, api, utils, tests, etc.)
- **context**: Brief description of what was formatted

**Optional elements**:
- **body**: Detailed list of fixes applied
- **footer**: Formatter name and version

## Success Criteria

Verify that all of the following are true before completing:

1. ✅ Formatting tool was identified and executed successfully
2. ✅ Auto-fix was enabled and applied to modified files
3. ✅ Changes were reviewed and verified as style-only
4. ✅ No functional changes were introduced
5. ✅ No breaking changes were made
6. ✅ Commit message follows the specified format
7. ✅ Changes are committed (or explicitly noted as not needed)
8. ✅ Remaining formatting issues (if any) are reported to user

## Error Handling

### No Formatter Configured

**Scenario**: Project has no formatting tool configured

**Action**:
- Identify the project language
- Recommend appropriate formatter
- Ask user if they want to configure formatting
- If yes, configure and proceed; if no, report and exit

**Example response**:
```
No formatting tool found. This is a TypeScript project.

Recommended formatter: Prettier

Would you like me to:
1. Install Prettier: npm install --save-dev prettier
2. Create basic .prettierrc.json configuration
3. Skip formatting for now

Please advise.
```

### Formatter Execution Fails

**Scenario**: Formatter command fails to execute

**Action**:
- Capture the error output
- Identify the likely cause (missing dependencies, invalid config, syntax errors)
- Suggest remediation steps
- Do not proceed until formatter runs successfully

**Example response**:
```
Formatter failed to run.

Error output:
Error: Cannot find module 'prettier'

Possible causes:
- Prettier is not installed
- node_modules directory is missing
- Package dependencies are not installed

Suggested fix:
npm install

Please resolve the formatter issue before proceeding.
```

### Formatter Introduces Breaking Changes

**Scenario**: Formatter changes code behavior (rare but possible with aggressive formatters)

**Action**:
- Revert the problematic changes
- Run formatter with less aggressive options
- Report the issue to user
- Ask for guidance on how to proceed

**Example response**:
```
Formatter introduced potential breaking changes:

Changed:
- src/auth/login.ts:45 - String concatenation split across lines may change behavior

This change could break functionality. Options:
1. Revert this specific change and keep other fixes
2. Revert all formatting changes
3. Manually review and fix

Please advise how to proceed.
```

### Formatter Configuration Conflicts

**Scenario**: Multiple formatter configs exist or configs conflict

**Action**:
- Identify all formatter configuration files
- Determine which config is active
- Report the conflict to user
- Ask which config to use

**Example response**:
```
Found multiple Prettier configurations:
- .prettierrc.json (root level)
- .prettierrc.js (in src/ directory)

These configs may conflict. Which configuration should be used?
1. Use .prettierrc.json (root level)
2. Use .prettierrc.js (src/ directory)
3. Merge configurations

Please advise.
```

### Formatter Takes Too Long

**Scenario**: Formatting takes more than 5 minutes

**Action**:
- Report the delay to user
- Check if formatter is stuck or processing large files
- Consider limiting formatting to modified files only
- Ask user if they want to continue or skip

**Example response**:
```
Formatting is taking longer than expected (5+ minutes).

Possible causes:
- Large number of files to format
- Large file sizes
- Slow file system

Options:
1. Continue waiting (may take 10+ minutes)
2. Limit formatting to modified files only
3. Skip formatting for now

Please advise.
```

### No Changes After Formatting

**Scenario**: Formatter runs successfully but makes no changes

**Action**:
- Verify formatter actually ran (check output)
- Verify formatter config is not empty
- Report to user: "No formatting fixes needed - code already meets formatting standards"
- Do not create a commit
- Exit successfully

**Example response**:
```
Formatting completed successfully.

Result: No changes needed - code already meets formatting standards.

Formatter output:
✓ 45 files checked
✓ 0 files reformatted
✓ All files properly formatted

No commit created.
```

### Partial Formatting

**Scenario**: Some files are formatted, but others cannot be formatted

**Action**:
- Commit the formatted changes
- Report files that could not be formatted with details
- Provide guidance on how to fix remaining issues
- Exit successfully after committing formatted files

**Example response**:
```
Formatting completed with partial success.

Formatted and committed:
- Reformatted 42 files
- Applied consistent indentation
- Fixed line length issues

Files that could not be formatted:
- src/auth/login.ts - Syntax error on line 45
- src/api/tasks.ts - Invalid JSON in embedded string

These files require manual fixes before they can be formatted.
Please review and address manually.
```

### Syntax Errors Prevent Formatting

**Scenario**: Files have syntax errors that prevent formatting

**Action**:
- Report the syntax errors clearly
- Do not attempt to format files with syntax errors
- Suggest fixing syntax errors first
- Exit without creating a commit

**Example response**:
```
Cannot format files due to syntax errors:

Errors found:
- src/auth/login.ts:45 - Unexpected token '}'
- src/api/tasks.ts:78 - Missing closing parenthesis

Formatters require valid syntax to operate. Please fix these syntax errors first, then run formatting again.

No commit created.
```

## Notes

- **Style Only**: Formatting should only change code style, never behavior. Always verify changes are cosmetic.
- **No Functional Changes**: Formatters should not alter program logic. If they do, the formatter config needs adjustment.
- **Scope Appropriately**: Format only modified files when possible to minimize changes and reduce execution time.
- **Respect Configuration**: Use the project's existing formatter configuration. Don't create new configs without user approval.
- **Idempotent**: Running the formatter multiple times should produce the same result.
- **Commit Separately**: Formatting fixes should be in a separate commit from implementation to maintain clean history.
- **Performance**: Formatting should complete in under 5 minutes. If it takes longer, consider limiting scope.
- **Compatibility**: Ensure formatter is compatible with the project's language version and syntax features.
- **Editor Integration**: Formatting via CLI should match editor formatting to avoid conflicts.
- **Ignore Files**: Respect .prettierignore, .rustfmt.toml skip patterns, and similar exclusion mechanisms.
