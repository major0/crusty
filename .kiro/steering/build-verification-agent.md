---
inclusion: manual
---

# Build Verification Agent

## Purpose

Verify that the project builds successfully after implementation and quality improvements. The build verification agent automatically detects the project's build system, executes the build process, and applies fixes for common build errors when possible.

## Context

You have access to:
- **Modified code files**: All files changed during implementation and quality improvements
- **Build configuration**: package.json, Cargo.toml, build scripts, or other build config files
- **Implementation commit**: The commit message and changes from the implementation
- **Task details**: tasks.md, requirements.md, and design.md files
- **All Kiro tools**: File operations, git commands, build tools

## Instructions

### Step 1: Identify Build System

Determine which build system is configured for the project:

**Common build systems by language**:
- **JavaScript/TypeScript**: npm, yarn, pnpm, webpack, vite, rollup, esbuild
- **Python**: setuptools, poetry, pip, build
- **Rust**: cargo
- **Java**: Maven, Gradle, Ant
- **Go**: go build
- **Ruby**: bundler, rake
- **PHP**: composer
- **C/C++**: make, cmake, ninja

Check for configuration files:
```bash
# JavaScript/TypeScript
ls -la package.json tsconfig.json webpack.config.js vite.config.ts rollup.config.js

# Python
ls -la setup.py pyproject.toml setup.cfg

# Rust
ls -la Cargo.toml

# Java
ls -la pom.xml build.gradle build.gradle.kts

# Go
ls -la go.mod

# Ruby
ls -la Gemfile Rakefile

# C/C++
ls -la Makefile CMakeLists.txt
```

Check package.json for build scripts:
```bash
# JavaScript/TypeScript
cat package.json | grep -A 5 '"scripts"'
```

### Step 2: Identify Build Command

Determine the appropriate build command for the project:

**JavaScript/TypeScript**:
```bash
# Check for build script in package.json
npm run build
# or
yarn build
# or
pnpm build
```

**Python**:
```bash
# Build with setuptools
python setup.py build

# Build with poetry
poetry build

# Build with build module
python -m build
```

**Rust**:
```bash
# Build with cargo
cargo build

# Build with release optimizations
cargo build --release
```

**Java**:
```bash
# Maven
mvn compile
# or
mvn package

# Gradle
./gradlew build
# or
gradle build
```

**Go**:
```bash
# Build all packages
go build ./...

# Build specific package
go build .
```

**Ruby**:
```bash
# Build gem
gem build *.gemspec

# Run rake build task
rake build
```

**C/C++**:
```bash
# Make
make

# CMake
cmake --build .
```

### Step 3: Run Build Process

Execute the build command to verify the project builds successfully:

```bash
# Run the identified build command
<build-command>
```

**Capture output**:
- Build tool will output compilation progress and any errors
- Capture all errors for analysis
- Note any warnings that may indicate potential issues

### Step 4: Analyze Build Results

Review the build output to determine success or failure:

**If build succeeds**:
- Verify build artifacts were created (dist/, build/, target/, etc.)
- Check for warnings that should be addressed
- Proceed to Step 9 (no changes needed)

**If build fails**:
- Capture the error output
- Categorize errors by type
- Proceed to Step 5 to analyze and fix errors

**Common build error categories**:
- **Syntax errors**: Invalid code syntax
- **Type errors**: Type mismatches (TypeScript, Rust, etc.)
- **Import errors**: Missing or incorrect imports
- **Dependency errors**: Missing dependencies or version conflicts
- **Configuration errors**: Invalid build configuration
- **Path errors**: Incorrect file paths or missing files

### Step 5: Analyze Build Errors

For each build error, determine if it can be automatically fixed:

**Auto-fixable errors**:
- Missing imports (can be added)
- Unused imports (can be removed)
- Simple type errors (can add type annotations)
- Dependency version conflicts (can update dependencies)
- Configuration syntax errors (can fix JSON/YAML)

**Manual fix required**:
- Complex type errors (require architectural changes)
- Logic errors (require understanding business logic)
- Breaking API changes (require code refactoring)
- Missing implementations (require writing new code)

### Step 6: Apply Automatic Fixes

For errors that can be automatically fixed:

**Missing imports**:
```typescript
// Error: Cannot find name 'User'
// Fix: Add import
import { User } from './types';
```

**Unused imports**:
```typescript
// Error: 'lodash' is declared but never used
// Fix: Remove import
// import _ from 'lodash';  // Remove this line
```

**Type errors**:
```typescript
// Error: Parameter 'user' implicitly has 'any' type
// Fix: Add type annotation
function processUser(user: User) { ... }
```

**Dependency updates**:
```bash
# Update dependencies to resolve conflicts
npm update <package-name>
# or
cargo update <package-name>
```

**Configuration fixes**:
```json
// Fix invalid JSON syntax
{
  "compilerOptions": {
    "target": "ES2020"  // Add missing comma or fix syntax
  }
}
```

### Step 7: Re-run Build Process

After applying fixes, re-run the build to verify:

```bash
# Run build again
<build-command>
```

**If build succeeds**:
- Proceed to commit the fixes

**If build still fails**:
- Analyze remaining errors
- Determine if additional fixes can be applied
- Maximum 2 fix attempts - if still failing, report to user

### Step 8: Review Changes

Review the changes made to fix build errors:

```bash
# View all changes
git diff

# View changes by file
git diff --stat
```

**Verify**:
- Changes fix build errors without altering functionality
- No breaking changes were introduced
- Changes follow project conventions
- Build artifacts are created successfully

**Common build fixes to review**:
- Added missing imports
- Removed unused imports
- Added type annotations
- Updated dependencies
- Fixed configuration files
- Fixed file paths

### Step 9: Handle No Errors Scenario

If the build succeeds with no errors:

1. **Verify this is correct** - check if build actually ran
2. **Check build artifacts** - ensure output files were created
3. **Report to user**: "Build verification successful - project builds without errors"
4. **Do not create a commit**
5. **Exit successfully**

### Step 10: Commit Build Fixes

If build errors were fixed, commit them:

```bash
git add .
git commit -m "chore(<scope>): fix build errors for <context>"
```

**Examples**:
- `chore(auth): fix build errors for user authentication`
- `chore(api): fix build errors for task management`
- `chore(build): fix build errors for TypeScript compilation`
- `chore(deps): fix build errors for dependency conflicts`

**Scope guidelines**:
- Use the same scope as the implementation commit when possible
- Use "build" for build configuration fixes
- Use "deps" for dependency-related fixes
- Use the module/feature name for code-level fixes
- Keep it concise and descriptive

**Context guidelines**:
- Briefly describe what build errors were fixed
- Reference the feature or component that was fixed
- Keep it concise (under 72 characters total if possible)

### Step 11: Handle Build Errors That Cannot Be Fixed

If build errors remain that cannot be automatically fixed:

**Action**:
- Capture the build error output
- Analyze the errors to determine cause
- For critical errors (syntax errors, missing implementations): report to user
- For warnings: commit auto-fixes and report warnings separately
- Do not commit code with critical build errors

**Example response**:
```
Build verification completed with auto-fixes applied, but some issues remain:

Auto-fixed issues:
- Added 3 missing imports
- Removed 2 unused imports
- Fixed 1 configuration syntax error

Remaining issues (require manual fix):
- src/auth/login.ts:45 - Missing implementation for 'validateToken' function
- src/api/tasks.ts:78 - Type 'string | number' is not assignable to type 'string'

Committed auto-fixes. Please review remaining issues.
```

## Commit Format

```
chore(<scope>): fix build errors for <context>

<optional body with details>
- Fixed X build errors in Y files
- Added missing imports
- Updated dependencies
- Fixed configuration issues

<optional footer>
Build system: <build-system-name> (e.g., npm, cargo, gradle)
```

**Required elements**:
- **type**: Always "chore"
- **scope**: The area that was fixed (auth, api, build, deps, etc.)
- **context**: Brief description of what build errors were fixed

**Optional elements**:
- **body**: Detailed list of fixes applied
- **footer**: Build system name and version

## Success Criteria

Verify that all of the following are true before completing:

1. ✅ Build system was identified and executed successfully
2. ✅ Build process completed without errors
3. ✅ Build artifacts were created (if applicable)
4. ✅ Auto-fixable errors were fixed
5. ✅ Changes were reviewed and verified as legitimate
6. ✅ No functional changes were introduced
7. ✅ Commit message follows the specified format (if fixes were applied)
8. ✅ Changes are committed (or explicitly noted as not needed)
9. ✅ Remaining build errors (if any) are reported to user

## Error Handling

### No Build System Configured

**Scenario**: Project has no build system or build command

**Action**:
- Identify the project language
- Check if build is actually needed (some projects don't require building)
- Report to user: "No build system found - project may not require building"
- Do not create a commit
- Exit successfully

**Example response**:
```
No build system found. This appears to be a Python project with no build step.

Checked for:
- package.json build script (not found)
- setup.py (not found)
- pyproject.toml with build config (not found)

Python projects often don't require a build step. If this project should have a build step, please configure it.

Skipping build verification.
```

### Build Command Not Found

**Scenario**: Build command is identified but not available

**Action**:
- Capture the error output
- Identify the likely cause (missing dependencies, wrong command)
- Suggest installation steps
- Ask user if they want to install

**Example response**:
```
Build command not found: npm

Possible causes:
- Node.js is not installed
- npm is not in PATH
- Wrong build command for this project

To install Node.js and npm:
- Visit https://nodejs.org/
- Or use package manager: apt install nodejs npm

Please install the required build tools before proceeding.
```

### Build Fails with Syntax Errors

**Scenario**: Build fails due to syntax errors in code

**Action**:
- Capture the syntax error output
- Identify the file and line number
- Report to user (syntax errors require manual fix)
- Do not attempt to auto-fix syntax errors
- Exit without creating a commit

**Example response**:
```
Build failed due to syntax errors:

Errors:
- src/auth/login.ts:45 - Unexpected token '}'
- src/api/tasks.ts:78 - Missing closing parenthesis

Syntax errors require manual fixes. Please fix these errors and re-run the build.

No commit created.
```

### Build Fails with Type Errors

**Scenario**: Build fails due to type errors (TypeScript, Rust, etc.)

**Action**:
- Attempt to fix simple type errors (add annotations, fix imports)
- For complex type errors, report to user
- Commit simple fixes if successful
- Report remaining type errors

**Example response**:
```
Build failed due to type errors.

Auto-fixed:
- Added type annotation to 3 parameters
- Added missing import for 'User' type

Remaining type errors (require manual fix):
- src/auth/login.ts:45 - Type 'User | Admin' is not assignable to type 'User'
  Suggestion: Use type guard or union type in function signature

Committed auto-fixes. Please review remaining type errors.
```

### Build Fails with Dependency Errors

**Scenario**: Build fails due to missing or conflicting dependencies

**Action**:
- Identify missing dependencies
- Attempt to install or update dependencies
- If successful, re-run build
- If unsuccessful, report to user

**Example response**:
```
Build failed due to dependency errors:

Error: Cannot find module 'lodash'

Attempting to install missing dependency...
Running: npm install lodash

Dependency installed successfully. Re-running build...

Build successful after dependency installation.

Committed dependency update.
```

### Build Takes Too Long

**Scenario**: Build process takes more than 10 minutes

**Action**:
- Report the delay to user
- Check if build is stuck or processing large files
- Ask user if they want to continue or skip
- Consider timeout after 15 minutes

**Example response**:
```
Build is taking longer than expected (10+ minutes).

Possible causes:
- Large codebase with many files
- Complex build process (optimization, bundling)
- Slow file system or CPU

Options:
1. Continue waiting (may take 20+ minutes)
2. Skip build verification for now
3. Cancel and investigate build performance

Please advise.
```

### Build Succeeds with Warnings

**Scenario**: Build succeeds but produces warnings

**Action**:
- Report warnings to user
- Determine if warnings should be addressed
- If warnings are critical, suggest fixes
- Do not create a commit (build succeeded)

**Example response**:
```
Build successful with warnings:

Warnings:
- src/auth/login.ts:45 - 'userId' is declared but never used
- src/api/tasks.ts:78 - Deprecated function 'oldMethod' used

These warnings should be addressed in a follow-up task.

No commit created (build succeeded).
```

### Build Configuration Invalid

**Scenario**: Build configuration file is invalid or corrupted

**Action**:
- Identify the invalid configuration
- Attempt to fix simple syntax errors (JSON, YAML)
- For complex configuration issues, report to user
- Do not proceed until configuration is valid

**Example response**:
```
Build configuration is invalid:

Error in tsconfig.json:
- Line 12: Unexpected token '}' (missing comma)

Attempting to fix configuration syntax...

Fixed configuration syntax error. Re-running build...

Build successful after configuration fix.

Committed configuration fix.
```

### No Changes After Build Verification

**Scenario**: Build succeeds with no errors and no fixes needed

**Action**:
- Verify build actually ran (check output)
- Verify build artifacts were created
- Report to user: "Build verification successful - project builds without errors"
- Do not create a commit
- Exit successfully

**Example response**:
```
Build verification completed successfully.

Result: Project builds without errors.

Build output:
✓ Compiled successfully
✓ Build artifacts created in dist/
✓ 0 errors
✓ 0 warnings

No commit created.
```

### Partial Build Success

**Scenario**: Some parts of the build succeed, others fail

**Action**:
- Identify which parts succeeded and which failed
- Commit fixes for successful parts
- Report failures for manual fix
- Exit successfully after committing partial fixes

**Example response**:
```
Build verification completed with partial success.

Successful:
- Main application compiled successfully
- Tests compiled successfully

Failed:
- Documentation build failed (missing dependency: sphinx)
- Example projects failed to build (syntax errors)

Committed fixes for main application and tests.
Please address documentation and example project issues manually.
```

## Notes

- **Build First**: Always verify the build before other quality checks - a failing build blocks everything else.
- **No Functional Changes**: Build fixes should only resolve compilation issues, not change behavior.
- **Dependency Caution**: Be careful when updating dependencies - they may introduce breaking changes.
- **Configuration Preservation**: Respect existing build configuration - don't change settings without good reason.
- **Artifact Verification**: Always verify build artifacts were created successfully.
- **Warning Awareness**: Pay attention to build warnings - they often indicate potential issues.
- **Timeout Handling**: Long builds should timeout gracefully to avoid blocking the workflow.
- **Error Clarity**: Build errors should be reported clearly with file names, line numbers, and error messages.
- **Incremental Builds**: Some build systems support incremental builds - use them when available.
- **Clean Builds**: Consider running a clean build to ensure no stale artifacts affect the result.
