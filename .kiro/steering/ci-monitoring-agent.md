---
inclusion: manual
---

# CI/CD Monitoring Agent

## Purpose

Monitor CI/CD pipeline checks after PR creation and automatically fix failures to ensure PRs pass all required checks. This agent watches the CI/CD status, analyzes failures, applies fixes, and maintains clean commit history through soft reset and re-commit cycles.

## Context

You have access to:
- **Pull request**: The PR created by the PR submission agent
- **CI/CD checks**: Status and logs from GitHub Actions, CircleCI, or other CI/CD systems
- **Commit history**: All commits on the topic branch
- **Task details**: tasks.md, requirements.md, and design.md files
- **Git operations**: Soft reset, commit, force push
- **All Kiro tools**: git commands, gh cli, file operations

## Instructions

### Step 1: Identify PR Number

Determine the PR number for the current branch:

```bash
# Get PR number for current branch
gh pr view --json number -q .number

# Or list PRs for current branch
gh pr list --head $(git branch --show-current) --json number,title,url
```

**If no PR exists**:
- Report error: "No PR found for current branch"
- Verify branch was pushed: `git branch -vv`
- Exit without monitoring

**Capture PR information**:
- PR number
- PR URL
- PR title

### Step 2: Start CI/CD Monitoring

Begin monitoring the CI/CD checks for the PR:

```bash
# Watch CI/CD checks (blocks until complete or timeout)
gh pr checks <pr-number> --watch

# Alternative: Poll checks periodically
gh pr checks <pr-number>
```

**Monitoring behavior**:
- The `--watch` flag will continuously monitor until checks complete
- Checks are considered complete when all have a conclusion (success, failure, cancelled, skipped)
- Timeout after 30 minutes if checks don't complete

**Check status values**:
- **success**: Check passed
- **failure**: Check failed
- **cancelled**: Check was cancelled
- **skipped**: Check was skipped
- **pending**: Check is still running
- **queued**: Check is queued but not started

### Step 3: Analyze Check Results

After checks complete (or timeout), analyze the results:

```bash
# Get detailed check information
gh pr checks <pr-number> --json name,conclusion,detailsUrl,summary

# Example output:
# [
#   {
#     "name": "build",
#     "conclusion": "success",
#     "detailsUrl": "https://github.com/...",
#     "summary": "Build completed successfully"
#   },
#   {
#     "name": "test",
#     "conclusion": "failure",
#     "detailsUrl": "https://github.com/...",
#     "summary": "3 tests failed"
#   }
# ]
```

**Categorize results**:
- **All success**: All checks passed → Report success and exit
- **Any failure**: One or more checks failed → Proceed to Step 4
- **Any cancelled**: Checks were cancelled → Report and ask user
- **Timeout**: Checks didn't complete in 30 minutes → Report and exit

### Step 4: Retrieve Failure Logs

For each failed check, retrieve detailed logs:

```bash
# Get check details
gh pr checks <pr-number> --json name,conclusion,detailsUrl,summary

# View logs for specific check (if available via gh cli)
gh run view <run-id> --log

# Or direct user to details URL for manual inspection
```

**Extract failure information**:
- Check name (e.g., "test", "lint", "build")
- Failure reason (e.g., "3 tests failed", "ESLint errors")
- Specific errors (e.g., test names, line numbers, error messages)
- Details URL for full logs

**Common failure types**:
- **Test failures**: Unit tests, integration tests, property tests failing
- **Linting failures**: ESLint, TSLint errors not caught locally
- **Build failures**: Compilation errors, dependency issues
- **Coverage failures**: Coverage below threshold
- **Security failures**: Security vulnerabilities detected
- **Type checking failures**: TypeScript errors not caught locally
- **Formatting failures**: Code style violations

### Step 5: Determine Fixability

Analyze each failure to determine if it can be automatically fixed:

**Auto-fixable failures**:
- Linting errors (run linter with auto-fix)
- Formatting errors (run formatter)
- Simple test failures (fix obvious bugs)
- Coverage gaps (add missing tests)
- Type errors (add type annotations)
- Build errors (fix imports, dependencies)

**Manual fix required**:
- Complex test failures (require understanding business logic)
- Security vulnerabilities (require security expertise)
- Breaking API changes (require architectural decisions)
- Infrastructure failures (CI/CD configuration issues)
- External service failures (third-party API down)

**If all failures require manual fix**:
- Report failures to user with detailed logs
- Provide guidance on how to fix
- Exit without attempting fixes

### Step 6: Pull Latest Changes

Before applying fixes, ensure the branch is up-to-date:

```bash
# Fetch latest from origin
git fetch origin

# Check if origin/main has new commits
git log HEAD..origin/main --oneline

# If origin/main has updates, merge them
git merge origin/main

# Or rebase if preferred
git rebase origin/main
```

**If merge conflicts occur**:
- Report conflicts to user
- Provide conflict resolution guidance
- Exit without attempting fixes

**If no updates needed**:
- Proceed to Step 7

### Step 7: Apply Fixes

For each auto-fixable failure, apply the appropriate fix:

**Test failures**:
```bash
# Run tests to reproduce failure
npm test

# Analyze failure and fix the code
# (This requires understanding the test failure)

# Re-run tests to verify fix
npm test
```

**Linting failures**:
```bash
# Run linter with auto-fix
npx eslint . --fix

# Verify linting passes
npx eslint .
```

**Formatting failures**:
```bash
# Run formatter
npx prettier --write .

# Verify formatting passes
npx prettier --check .
```

**Coverage failures**:
```bash
# Run coverage analysis
npm test -- --coverage

# Identify uncovered code
# Add tests for uncovered code

# Re-run coverage
npm test -- --coverage
```

**Type checking failures**:
```bash
# Run type checker
npx tsc --noEmit

# Fix type errors (add annotations, fix imports)

# Re-run type checker
npx tsc --noEmit
```

**Build failures**:
```bash
# Run build
npm run build

# Fix build errors (imports, dependencies, syntax)

# Re-run build
npm run build
```

**Verify all fixes**:
- Run all checks locally before pushing
- Ensure no new errors were introduced
- Ensure fixes don't break existing functionality

### Step 8: Soft Reset and Re-commit

After applying fixes, consolidate the commit history:

```bash
# Soft reset preserves all changes in staging area
git reset --soft origin/main

# Verify all changes are staged
git status
```

**Critical**: The soft reset removes all commits from the branch history but keeps all file changes staged. This maintains clean commit history.

**Verify**:
- All file changes (original + fixes) are in the staging area
- No commits exist on the branch (compared to origin/main)
- Working directory is clean (no unstaged changes)

### Step 9: Re-create Consolidated Commits

Re-commit all changes with the same clean structure as PR submission:

#### Commit 1: Implementation

```bash
git commit -m "<type>(<scope>): <task-id> <task-title>

<optional body with implementation details>
- Key changes made
- Design decisions
- Important notes

<optional footer>
Refs: #<issue-number>
Breaking Change: <description if applicable>"
```

**Requirements**:
- Type must be one of: feat, fix, refactor, perf
- Must include task ID and task title
- Scope should match the implementation area
- Body should summarize key implementation details

#### Commit 2: Testing

```bash
git commit -m "test(<scope>): add comprehensive tests for <context>

<optional body with testing details>
- Property-based tests with 100+ iterations
- Unit tests for edge cases and error conditions
- Coverage improved to 90%+ across all metrics

<optional footer>
Coverage metrics:
- Statements: X%
- Branches: X%
- Functions: X%
- Lines: X%"
```

**Requirements**:
- Type must be "test"
- Should reference the same scope as implementation
- Body should summarize all testing work (including new tests added to fix failures)

#### Commit 3: Documentation

```bash
git commit -m "docs(<scope>): update documentation for <context>

<optional body with documentation details>
- Updated inline comments for new functions
- Added API documentation for endpoints
- Updated README with usage examples

<optional footer>"
```

**Requirements**:
- Type must be "docs"
- Should reference the same scope as implementation
- Body should summarize all documentation updates

#### Commit 4: Quality Fixes

```bash
git commit -m "chore(<scope>): apply code quality fixes for <context>

<optional body with quality details>
- Applied linting fixes (ESLint)
- Applied formatting fixes (Prettier)
- Fixed type errors (TypeScript)
- Applied security fixes
- Fixed build errors
- Passed all pre-commit hooks
- Fixed CI/CD failures

<optional footer>"
```

**Requirements**:
- Type must be "chore"
- Should reference the same scope as implementation
- Body should summarize all quality improvements (including CI/CD fixes)

### Step 10: Verify Branch Tracking and Force Push

Before pushing, verify the branch is tracking origin/main:

```bash
# Check branch tracking
git branch -vv | grep "$(git branch --show-current)"
```

**Expected output should show tracking origin/main**:
```
* feat/1.2-user-login abc1234 [origin/main>] feat(auth): 1.2 Implement user login endpoint
```

**If not tracking origin/main**:
```bash
# Set upstream tracking to origin/main
git branch --set-upstream-to=origin/main
```

Push the updated commits to the PR branch:

```bash
# Force push with lease (no need to specify origin/branch since we track origin/main)
git push --force-with-lease
```

**Force push with lease**:
- Safer than `--force` because it checks if remote has unexpected changes
- Prevents overwriting others' work
- Fails if remote branch has commits you don't have locally

**Important**: Do NOT use `git push --force-with-lease origin <branch-name>` as this pattern requires command approval. Since the branch tracks origin/main, simply use `git push --force-with-lease`.

**If push fails**:
- Fetch latest: `git fetch origin`
- Check for conflicts: `git log HEAD..origin/$(git branch --show-current)`
- Report to user if remote has unexpected changes

**Verify push succeeded**:
```bash
# Check remote branch
git branch -vv

# Verify commits on remote
git log origin/$(git branch --show-current) --oneline -5
```

### Step 11: Restart Monitoring

After pushing fixes, restart CI/CD monitoring:

```bash
# Wait a moment for CI/CD to detect the push
sleep 10

# Restart monitoring
gh pr checks <pr-number> --watch
```

**Retry limit**:
- Maximum 3 fix attempts
- Track attempt count: 1st attempt, 2nd attempt, 3rd attempt
- If 3rd attempt fails, report to user and exit

**Between attempts**:
- Wait for CI/CD checks to complete
- Analyze new failures (may be different from previous)
- Apply fixes for new failures
- Repeat soft reset and re-commit process

### Step 12: Report Final Status

After monitoring completes (success or max retries reached):

**If all checks pass**:
```
CI/CD monitoring completed successfully!

PR #<number>: <title>
URL: <url>

All checks passed:
✓ <check-1-name>
✓ <check-2-name>
✓ <check-3-name>

The PR is ready for review and merge.
```

**If fixes were applied**:
```
CI/CD monitoring completed with automatic fixes!

PR #<number>: <title>
URL: <url>

Fixes applied (attempt <N>/3):
- Fixed <failure-type-1>
- Fixed <failure-type-2>
- Fixed <failure-type-3>

All checks now passing:
✓ <check-1-name>
✓ <check-2-name>
✓ <check-3-name>

The PR is ready for review and merge.
```

**If max retries reached**:
```
CI/CD monitoring failed after 3 attempts.

PR #<number>: <title>
URL: <url>

Remaining failures:
✗ <check-1-name>: <failure-reason>
  Details: <details-url>
  
✗ <check-2-name>: <failure-reason>
  Details: <details-url>

Attempted fixes:
- Attempt 1: <what-was-fixed>
- Attempt 2: <what-was-fixed>
- Attempt 3: <what-was-fixed>

These failures require manual intervention. Please review the failure logs and apply fixes manually.
```

**If timeout occurred**:
```
CI/CD monitoring timed out after 30 minutes.

PR #<number>: <title>
URL: <url>

Check status:
- <check-1-name>: <status>
- <check-2-name>: <status>
- <check-3-name>: <status>

Some checks are still running. You can continue monitoring manually:
gh pr checks <pr-number> --watch

Or view the PR on GitHub:
<pr-url>
```

## Commit Format

The re-committed changes must follow the same format as PR submission:

### Implementation Commit
```
<type>(<scope>): <task-id> <task-title>

<optional body>
- Implementation details
- Design decisions
- Important notes

<optional footer>
Refs: #<issue-number>
Breaking Change: <description>
```

### Testing Commit
```
test(<scope>): add comprehensive tests for <context>

<optional body>
- Property tests summary
- Unit tests summary
- Coverage metrics
- CI/CD test fixes

<optional footer>
Coverage: X% statements, X% branches, X% functions, X% lines
```

### Documentation Commit
```
docs(<scope>): update documentation for <context>

<optional body>
- Inline comments updates
- API documentation updates
- README updates

<optional footer>
```

### Quality Commit
```
chore(<scope>): apply code quality fixes for <context>

<optional body>
- Linting fixes
- Formatting fixes
- Type checking fixes
- Security fixes
- Build fixes
- Pre-commit fixes
- CI/CD failure fixes

<optional footer>
```

## Success Criteria

Verify that all of the following are true before completing:

1. ✅ PR number was identified successfully
2. ✅ CI/CD monitoring was started
3. ✅ Check results were analyzed
4. ✅ All checks passed OR max retries (3) reached
5. ✅ Fixes were applied for auto-fixable failures
6. ✅ Soft reset was performed after each fix attempt
7. ✅ Clean commit history was maintained (4 commits)
8. ✅ Force push with lease succeeded
9. ✅ Final status was reported to user
10. ✅ Timeout handling (30 minutes) was respected

## Error Handling

### No PR Found

**Scenario**: No PR exists for the current branch

**Action**:
- Check if branch was pushed: `git branch -vv`
- Check if PR was created: `gh pr list --head $(git branch --show-current)`
- Report error to user
- Exit without monitoring

**Example response**:
```
Error: No PR found for current branch

Branch: feat/1.2-user-login

Possible causes:
- PR was not created (check previous step)
- Branch name doesn't match PR head branch
- gh cli is not authenticated

Please verify:
1. Branch is pushed: git branch -vv
2. PR exists: gh pr list --head $(git branch --show-current)
3. gh cli is authenticated: gh auth status

Cannot proceed with CI/CD monitoring.
```

### CI/CD Checks Timeout

**Scenario**: Checks don't complete within 30 minutes

**Action**:
- Report timeout to user
- Show current check status
- Provide instructions for manual monitoring
- Exit gracefully

**Example response**:
```
CI/CD monitoring timed out after 30 minutes.

PR #42: feat(auth): Implement user login endpoint
URL: https://github.com/user/repo/pull/42

Check status at timeout:
- build: success ✓
- test: pending (running for 25 minutes)
- lint: success ✓
- coverage: queued

The 'test' check is taking longer than expected.

You can continue monitoring manually:
gh pr checks 42 --watch

Or view the PR on GitHub:
https://github.com/user/repo/pull/42
```

### All Checks Cancelled

**Scenario**: All or most checks were cancelled

**Action**:
- Report cancellation to user
- Ask if they want to re-trigger checks
- Provide instructions for re-triggering
- Exit without attempting fixes

**Example response**:
```
CI/CD checks were cancelled.

PR #42: feat(auth): Implement user login endpoint
URL: https://github.com/user/repo/pull/42

Cancelled checks:
- build: cancelled
- test: cancelled
- lint: cancelled

Possible causes:
- User manually cancelled checks
- CI/CD system issue
- Workflow configuration error

To re-trigger checks:
1. Make a trivial change: git commit --allow-empty -m "chore: trigger CI"
2. Push: git push
3. Or use GitHub UI: "Re-run all jobs"

Would you like me to trigger checks with an empty commit? (yes/no)
```

### Unfixable Failures

**Scenario**: All failures require manual intervention

**Action**:
- Report failures to user with details
- Provide guidance on how to fix each failure
- Do not attempt automatic fixes
- Exit without retrying

**Example response**:
```
CI/CD checks failed with issues that require manual intervention.

PR #42: feat(auth): Implement user login endpoint
URL: https://github.com/user/repo/pull/42

Failed checks:
✗ security-scan: High severity vulnerability detected
  Details: https://github.com/user/repo/actions/runs/123
  Issue: SQL injection vulnerability in login endpoint
  Fix: Use parameterized queries instead of string concatenation

✗ integration-tests: External API unavailable
  Details: https://github.com/user/repo/actions/runs/124
  Issue: Auth service API returned 503
  Fix: Wait for service to recover or use mock in tests

These issues cannot be automatically fixed. Please:
1. Review the failure details at the URLs above
2. Apply the suggested fixes manually
3. Push the fixes to the PR branch

The CI/CD monitor will automatically restart after you push.
```

### Merge Conflicts During Pull

**Scenario**: Pulling latest from origin/main causes merge conflicts

**Action**:
- Report conflicts to user
- Show conflicting files
- Provide conflict resolution guidance
- Exit without attempting fixes

**Example response**:
```
Error: Merge conflicts detected when pulling latest changes.

Conflicting files:
- src/auth/login.ts
- src/utils/validation.ts

To resolve conflicts:
1. Open each conflicting file
2. Resolve the conflicts (look for <<<<<<, ======, >>>>>>)
3. Stage the resolved files: git add <file>
4. Complete the merge: git commit
5. Re-run the CI/CD monitor

Or abort the merge and fix manually:
git merge --abort

Cannot proceed with automatic fixes until conflicts are resolved.
```

### Force Push Fails

**Scenario**: Force push with lease fails

**Action**:
- Capture the git error output
- Check if remote has unexpected changes
- Report to user with details
- Do not retry automatically

**Example response**:
```
Error: Force push failed

Git output:
! [rejected]        feat/1.2-user-login -> feat/1.2-user-login (stale info)

The remote branch has changes that you don't have locally.

Possible causes:
- Someone else pushed to the branch
- CI/CD system made commits
- Previous push didn't complete

To investigate:
git fetch origin
git log HEAD..origin/feat/1.2-user-login

To force push anyway (DANGEROUS):
git push --force origin HEAD

Please investigate before proceeding.
```

### Max Retries Reached

**Scenario**: 3 fix attempts made but checks still failing

**Action**:
- Report all attempted fixes
- Show remaining failures
- Provide detailed guidance for manual fixes
- Exit gracefully

**Example response**:
```
CI/CD monitoring failed after 3 fix attempts.

PR #42: feat(auth): Implement user login endpoint
URL: https://github.com/user/repo/pull/42

Attempted fixes:
- Attempt 1: Fixed linting errors (12 files), fixed formatting (8 files)
  Result: Linting passed, but tests still failing
  
- Attempt 2: Fixed 2 failing tests, added missing test cases
  Result: Tests passed, but coverage below threshold (87% < 90%)
  
- Attempt 3: Added tests for uncovered branches, improved coverage to 91%
  Result: Coverage passed, but build failing with type errors

Remaining failures:
✗ build: TypeScript compilation errors
  Details: https://github.com/user/repo/actions/runs/125
  Errors:
    - src/auth/login.ts:45 - Type 'User | Admin' is not assignable to type 'User'
    - src/api/tasks.ts:78 - Property 'userId' does not exist on type 'Task'

These type errors require architectural decisions and cannot be automatically fixed.

Please review the errors and apply fixes manually. The type errors suggest:
1. Use type guards or union types in function signatures
2. Update interfaces to match actual data structures
```

### Check Logs Unavailable

**Scenario**: Cannot retrieve detailed logs for failed checks

**Action**:
- Report that logs are unavailable
- Provide details URL for manual inspection
- Attempt fixes based on check name and summary
- If fixes fail, report to user

**Example response**:
```
Warning: Detailed logs unavailable for failed checks.

Failed checks:
✗ test: 3 tests failed
  Details: https://github.com/user/repo/actions/runs/123
  Summary: "3 tests failed"
  
Cannot retrieve detailed logs via gh cli. Please:
1. Visit the details URL above
2. Review the test failure logs
3. Identify which tests failed and why

Attempting generic test fixes based on common issues...
```

### gh CLI Not Authenticated

**Scenario**: gh cli is not authenticated

**Action**:
- Check authentication status: `gh auth status`
- Provide authentication instructions
- Exit without monitoring

**Example response**:
```
Error: gh cli is not authenticated

To authenticate:
gh auth login

Follow the prompts to authenticate with GitHub.

After authentication, re-run the CI/CD monitor.
```

### No Checks Configured

**Scenario**: PR has no CI/CD checks configured

**Action**:
- Report that no checks are configured
- Verify this is expected
- Exit successfully (nothing to monitor)

**Example response**:
```
No CI/CD checks configured for this PR.

PR #42: feat(auth): Implement user login endpoint
URL: https://github.com/user/repo/pull/42

This repository may not have CI/CD workflows configured, or checks may not be required for this branch.

To verify:
1. Check .github/workflows/ for workflow files
2. Check branch protection rules
3. Check repository settings for required checks

No monitoring needed. The PR is ready for review.
```

## Notes

- **Timeout Handling**: Always respect the 30-minute timeout to avoid blocking indefinitely.
- **Retry Limit**: Maximum 3 fix attempts to avoid infinite loops.
- **Clean History**: Always maintain clean commit history through soft reset and re-commit.
- **Force Push Safety**: Always use `--force-with-lease` instead of `--force`.
- **Fixability Assessment**: Carefully assess whether failures can be automatically fixed.
- **Manual Intervention**: Some failures (security, complex logic, infrastructure) require manual fixes.
- **Detailed Reporting**: Provide detailed failure information to help users fix issues manually.
- **Idempotent Fixes**: Fixes should be idempotent - applying them multiple times should be safe.
- **Local Verification**: Always verify fixes locally before pushing.
- **Graceful Degradation**: If automatic fixes fail, provide clear guidance for manual fixes.
