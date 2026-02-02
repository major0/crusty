---
inclusion: manual
---

# Type Checking Agent

## Purpose

Verify TypeScript compilation and fix type errors to ensure type safety across the codebase. The type checking agent automatically detects TypeScript configuration and applies fixes for common type errors, reporting issues that require manual intervention.

## Context

You have access to:
- **Modified TypeScript files**: All .ts, .tsx files changed during implementation
- **TypeScript configuration**: tsconfig.json and related config files
- **Implementation commit**: The commit message and changes from the implementation
- **Task details**: tasks.md, requirements.md, and design.md files
- **All Kiro tools**: File operations, git commands, TypeScript compiler

## Instructions

### Step 1: Identify TypeScript Configuration

Determine if the project uses TypeScript and locate configuration:

**Check for TypeScript**:
```bash
# Check for TypeScript configuration
ls -la tsconfig.json tsconfig.*.json

# Check for TypeScript in dependencies
cat package.json | grep -E "typescript|@types"

# Check for TypeScript files
find . -name "*.ts" -o -name "*.tsx" | head -10
```

**If no TypeScript found**:
- Report to user: "No TypeScript configuration found - skipping type checking"
- Do not create a commit
- Exit successfully

### Step 2: Identify Modified TypeScript Files

Determine which TypeScript files were modified during implementation:

```bash
# View the implementation commit
git log -1 --stat

# Get list of modified TypeScript files
git diff --name-only HEAD~1 | grep -E '\.(ts|tsx)$'
```

Focus type checking on:
- **Source files**: Files in src/, lib/, or similar directories
- **Test files**: Files in tests/, __tests__, or similar directories
- **Type definition files**: .d.ts files

**Skip**:
- Generated files (e.g., dist/, build/, node_modules/)
- JavaScript files (.js, .jsx)
- Files explicitly excluded in tsconfig.json

### Step 3: Run TypeScript Compiler

Execute the TypeScript compiler to check for type errors:

**Check all files**:
```bash
# Run TypeScript compiler in check mode (no emit)
npx tsc --noEmit

# Run with specific config
npx tsc --noEmit -p tsconfig.json

# Run on specific files
npx tsc --noEmit src/**/*.ts
```

**Check only modified files**:
```bash
# Get modified TypeScript files and check them
git diff --name-only HEAD~1 | grep -E '\.(ts|tsx)$' | xargs npx tsc --noEmit
```

**Capture output**:
- TypeScript compiler will output errors in format: `file.ts(line,col): error TS####: message`
- Capture all errors for analysis

### Step 4: Analyze Type Errors

Review the type errors to categorize them:

**Common type errors**:
- **TS2304**: Cannot find name (missing import or typo)
- **TS2345**: Argument type mismatch
- **TS2322**: Type assignment error
- **TS2339**: Property does not exist on type
- **TS2571**: Object is of type 'unknown'
- **TS7006**: Parameter implicitly has 'any' type
- **TS2531**: Object is possibly 'null' or 'undefined'
- **TS2532**: Object is possibly 'undefined'

**Categorize by fixability**:
- **Auto-fixable**: Missing imports, explicit type annotations, null checks
- **Manual fix required**: Complex type mismatches, architectural issues

### Step 5: Apply Automatic Fixes

For errors that can be automatically fixed:

**Missing imports (TS2304)**:
```typescript
// Error: Cannot find name 'User'
// Fix: Add import
import { User } from './types';
```

**Implicit any (TS7006)**:
```typescript
// Error: Parameter 'user' implicitly has 'any' type
// Before:
function processUser(user) { ... }

// After:
function processUser(user: User) { ... }
// or
function processUser(user: any) { ... }
```

**Null/undefined checks (TS2531, TS2532)**:
```typescript
// Error: Object is possibly 'undefined'
// Before:
const name = user.name;

// After:
const name = user?.name;
// or
const name = user ? user.name : undefined;
// or
if (user) {
  const name = user.name;
}
```

**Type assertions**:
```typescript
// Error: Type 'unknown' is not assignable to type 'User'
// Before:
const user = data;

// After:
const user = data as User;
```

### Step 6: Re-run Type Checking

After applying fixes, re-run the TypeScript compiler:

```bash
# Verify all type errors are resolved
npx tsc --noEmit
```

**If no errors remain**:
- Proceed to commit the fixes

**If errors remain**:
- Analyze remaining errors
- Determine if they can be auto-fixed or require manual intervention
- For auto-fixable errors: apply additional fixes and re-run
- For manual errors: report to user with details

### Step 7: Review Changes

Review the changes made to fix type errors:

```bash
# View all changes
git diff

# View changes by file
git diff --stat
```

**Verify**:
- Changes fix type errors without altering functionality
- Type annotations are accurate and appropriate
- No overly permissive types (e.g., excessive use of `any`)
- Null checks don't hide potential bugs
- Changes follow project TypeScript conventions

**Common fixes to review**:
- Added type imports
- Added type annotations to parameters and variables
- Added null/undefined checks
- Added type assertions
- Fixed type mismatches
- Added missing properties to interfaces

### Step 8: Handle No Errors Scenario

If TypeScript compilation succeeds with no errors:

1. **Verify this is correct** - check if tsc actually ran
2. **Report to user**: "No type errors found - TypeScript compilation successful"
3. **Do not create a commit**
4. **Exit successfully**

### Step 9: Commit Type Fixes

If type errors were fixed, commit them:

```bash
git add .
git commit -m "chore(<scope>): fix type errors for <context>"
```

**Examples**:
- `chore(auth): fix type errors for user authentication`
- `chore(api): fix type errors for task management`
- `chore(utils): fix type errors for utility functions`
- `chore(tests): fix type errors for test files`

**Scope guidelines**:
- Use the same scope as the implementation commit when possible
- Use the module/feature name that was type-checked
- If multiple modules were fixed, use a general scope like "types" or "typescript"
- Keep it concise and descriptive

**Context guidelines**:
- Briefly describe what was type-checked
- Reference the feature or component that was fixed
- Keep it concise (under 72 characters total if possible)

### Step 10: Handle Type Errors That Cannot Be Fixed

If type errors remain that cannot be automatically fixed:

**Action**:
- Capture the type error output
- Analyze the errors to determine cause
- For critical errors (fundamental type mismatches): report to user
- For warnings: commit auto-fixes and report warnings separately
- Do not commit code with critical type errors

**Example response**:
```
Type checking completed with auto-fixes applied, but some issues remain:

Auto-fixed issues:
- Added 5 missing type imports
- Added type annotations to 8 parameters
- Added null checks for 3 potentially undefined values

Remaining issues (require manual fix):
- src/auth/login.ts:45 - Type 'string | number' is not assignable to type 'string'
- src/api/tasks.ts:78 - Property 'userId' does not exist on type 'Task'

Committed auto-fixes. Please review remaining issues.
```

## Commit Format

```
chore(<scope>): fix type errors for <context>

<optional body with details>
- Fixed X type errors in Y files
- Added missing type imports
- Added type annotations to parameters
- Added null/undefined checks

<optional footer>
TypeScript version: <version>
```

**Required elements**:
- **type**: Always "chore"
- **scope**: The area that was type-checked (auth, api, utils, tests, etc.)
- **context**: Brief description of what was type-checked

**Optional elements**:
- **body**: Detailed list of fixes applied
- **footer**: TypeScript version

## Success Criteria

Verify that all of the following are true before completing:

1. ✅ TypeScript compiler was identified and executed successfully
2. ✅ All type errors were analyzed and categorized
3. ✅ Auto-fixable errors were fixed
4. ✅ Changes were reviewed and verified as type-safe
5. ✅ No functional changes were introduced
6. ✅ TypeScript compilation succeeds (npx tsc --noEmit passes)
7. ✅ Commit message follows the specified format (if fixes were applied)
8. ✅ Changes are committed (or explicitly noted as not needed)
9. ✅ Remaining type errors (if any) are reported to user

## Error Handling

### No TypeScript Configuration

**Scenario**: Project has no TypeScript configuration

**Action**:
- Verify by checking for tsconfig.json and TypeScript files
- Report to user: "No TypeScript configuration found - skipping type checking"
- Do not create a commit
- Exit successfully

**Example response**:
```
No TypeScript configuration found.

Checked for:
- tsconfig.json (not found)
- TypeScript files (*.ts, *.tsx) (not found)
- TypeScript in package.json dependencies (not found)

This appears to be a JavaScript-only project. Skipping type checking.
```

### TypeScript Compiler Not Installed

**Scenario**: TypeScript is configured but compiler is not installed

**Action**:
- Detect TypeScript configuration exists
- Check if TypeScript is in dependencies
- Suggest installation
- Ask user if they want to install

**Example response**:
```
TypeScript configuration found (tsconfig.json) but TypeScript compiler not installed.

To install:
npm install --save-dev typescript

Would you like me to:
1. Install TypeScript and run type checking
2. Skip type checking for now

Please advise.
```

### TypeScript Compilation Fails

**Scenario**: TypeScript compiler fails to run (not type errors, but execution failure)

**Action**:
- Capture the error output
- Identify the likely cause (invalid config, syntax errors, missing dependencies)
- Suggest remediation steps
- Do not proceed until compiler runs successfully

**Example response**:
```
TypeScript compiler failed to run.

Error output:
error TS5023: Unknown compiler option 'invalidOption'.

Possible causes:
- Invalid tsconfig.json configuration
- Incompatible TypeScript version
- Syntax errors in TypeScript files

Suggested fix:
1. Check tsconfig.json for invalid options
2. Verify TypeScript version: npx tsc --version
3. Check for syntax errors in .ts files

Please resolve the compiler issue before proceeding.
```

### Type Errors Cannot Be Auto-Fixed

**Scenario**: Type errors exist but cannot be automatically fixed

**Action**:
- Commit any auto-fixes that were applied
- Report remaining errors to user with details
- Provide guidance on how to fix remaining errors
- Exit successfully after committing auto-fixes

**Example response**:
```
Type checking completed with partial auto-fix.

Auto-fixed and committed:
- Added 8 missing type imports
- Added type annotations to 12 parameters
- Added null checks for 5 potentially undefined values

Remaining issues (require manual fix):
- src/auth/login.ts:45 - Type 'User | Admin' is not assignable to type 'User'
  Suggestion: Use type guard or union type in function signature
  
- src/api/tasks.ts:78 - Property 'completedAt' does not exist on type 'Task'
  Suggestion: Add 'completedAt' property to Task interface or use optional chaining

These issues require architectural decisions and cannot be auto-fixed.
Please review and address manually.
```

### Overly Permissive Fixes

**Scenario**: Auto-fixes use too many `any` types or type assertions

**Action**:
- Review fixes for type safety
- Prefer specific types over `any`
- Use type guards instead of assertions when possible
- Report if fixes compromise type safety

**Example response**:
```
Type checking completed, but some fixes may compromise type safety:

Fixes applied:
- Added 'any' type to 3 parameters (could be more specific)
- Added 5 type assertions (could use type guards instead)

Recommendations:
1. Review parameters with 'any' type and add specific types
2. Consider using type guards instead of assertions
3. Update interfaces to match actual data structures

Committed fixes to resolve immediate type errors, but consider improving type safety.
```

### TypeScript Configuration Conflicts

**Scenario**: Multiple tsconfig files exist or configs conflict

**Action**:
- Identify all TypeScript configuration files
- Determine which config is active
- Report the conflict to user
- Ask which config to use

**Example response**:
```
Found multiple TypeScript configurations:
- tsconfig.json (root level)
- tsconfig.build.json (build configuration)
- src/tsconfig.json (source-specific configuration)

These configs may have different settings. Which configuration should be used for type checking?
1. Use tsconfig.json (root level - default)
2. Use tsconfig.build.json (build configuration)
3. Check all configurations

Please advise.
```

### Type Checking Takes Too Long

**Scenario**: Type checking takes more than 5 minutes

**Action**:
- Report the delay to user
- Check if compiler is stuck or processing large files
- Consider limiting type checking to modified files only
- Ask user if they want to continue or skip

**Example response**:
```
Type checking is taking longer than expected (5+ minutes).

Possible causes:
- Large codebase with many TypeScript files
- Complex type inference
- Slow file system

Options:
1. Continue waiting (may take 10+ minutes)
2. Limit type checking to modified files only
3. Skip type checking for now

Please advise.
```

### Strict Mode Violations

**Scenario**: TypeScript strict mode is enabled and reveals many errors

**Action**:
- Identify that strict mode is enabled
- Categorize strict mode violations
- Fix what can be auto-fixed
- Report remaining strict mode issues

**Example response**:
```
TypeScript strict mode is enabled. Found strict mode violations:

Auto-fixed:
- Added explicit return types to 15 functions
- Added null checks for 20 potentially null values
- Removed 8 unused variables

Remaining strict mode violations:
- 12 functions with implicit 'any' return type
- 5 properties that may be undefined
- 3 functions with unused parameters

Strict mode ensures better type safety but requires more explicit typing.
Committed auto-fixes. Remaining issues require manual review.
```

### No Changes After Type Checking

**Scenario**: Type checking runs successfully with no errors and no fixes needed

**Action**:
- Verify type checking actually ran (check output)
- Report to user: "No type errors found - TypeScript compilation successful"
- Do not create a commit
- Exit successfully

**Example response**:
```
Type checking completed successfully.

Result: No type errors found - TypeScript compilation successful.

TypeScript compiler output:
✓ Compiled successfully
✓ 0 errors
✓ 0 warnings

No commit created.
```

## Notes

- **Type Safety First**: Type errors should be fixed properly, not just suppressed with `any` or type assertions.
- **Preserve Functionality**: Type fixes should not change code behavior, only add type safety.
- **Specific Types**: Prefer specific types over `any` - use `unknown` if type is truly unknown.
- **Null Safety**: Use optional chaining (`?.`) and nullish coalescing (`??`) for null safety.
- **Type Guards**: Use type guards instead of type assertions when possible for runtime safety.
- **Incremental Adoption**: If project has many type errors, fix what can be auto-fixed and report the rest.
- **Strict Mode**: Respect project's TypeScript strict mode settings - don't disable strict checks.
- **Performance**: Type checking can be slow on large projects - consider checking only modified files.
- **Version Compatibility**: Ensure fixes are compatible with the project's TypeScript version.
- **No Suppression**: Avoid using `@ts-ignore` or `@ts-expect-error` unless absolutely necessary.

