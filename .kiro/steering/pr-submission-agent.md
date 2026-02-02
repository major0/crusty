---
inclusion: manual
---

# PR Submission Agent

## Purpose

Consolidate all commits from the topic branch into a clean, logical commit history and create a pull request with appropriate metadata. This agent ensures PRs have exactly 4 commits (implementation, tests, documentation, quality fixes) following Conventional Commits format.

## Context

You have access to:
- **Topic branch**: The current branch with implementation and quality commits
- **Commit history**: All commits from implementation and quality agents
- **Task details**: tasks.md, requirements.md, and design.md files
- **Git operations**: Soft reset, commit, push, and PR creation
- **All Kiro tools**: taskStatus, git commands, gh cli, file operations

## Instructions

### Step 1: Validate Quality Agents Completed

Before proceeding, verify all quality agents have completed their work:

```bash
# View recent commit history
git log --oneline -20

# Check for quality commits
git log --oneline --grep="^docs\|^test\|^chore" -10
```

**Expected commits**:
- One implementation commit (feat|fix|refactor|perf)
- Multiple quality commits (docs, test, chore types)

**If quality commits are missing**:
- Report which quality agents may not have completed
- Ask user if they want to proceed anyway or wait for completion

### Step 2: Mark Task as Completed

Use the taskStatus tool to mark the task as completed:

```
taskStatus(
  taskFilePath: ".kiro/specs/<spec-name>/tasks.md",
  task: "<task-id> <task-title>",
  status: "completed"
)
```

**Example:**
```
taskStatus(
  taskFilePath: ".kiro/specs/user-authentication/tasks.md",
  task: "1.2 Implement user login endpoint",
  status: "completed"
)
```

**Important**: The task parameter must match the exact text from the tasks.md file, including the task ID and title.

### Step 3: Analyze Existing Commits

Review all commits on the topic branch to understand what changes were made:

```bash
# View detailed commit history
git log origin/main..HEAD --oneline

# View commit types
git log origin/main..HEAD --format="%s" | cut -d: -f1 | sort | uniq -c
```

**Categorize commits by type**:
- **Implementation**: feat, fix, refactor, perf commits
- **Testing**: test commits (property tests, unit tests, coverage improvements)
- **Documentation**: docs commits
- **Quality**: chore commits (linting, formatting, pre-commit, security, type-checking, build)

**Extract key information**:
- Task ID from implementation commit
- Task title from implementation commit
- Scope from implementation commit
- Summary of changes from all commits

### Step 4: Perform Soft Reset

Consolidate all commits by performing a soft reset against origin/main:

```bash
# Soft reset preserves all changes in staging area
git reset --soft origin/main

# Verify all changes are staged
git status
```

**Critical**: The soft reset removes all commits from the branch history but keeps all file changes staged. This allows creating a clean commit history.

**Verify**:
- All file changes are in the staging area
- No commits exist on the branch (compared to origin/main)
- Working directory is clean (no unstaged changes)

### Step 5: Create Consolidated Commits

Re-commit all changes with a clean, logical structure. Create exactly 4 commits in this order:

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

**Example:**
```bash
git commit -m "feat(auth): 1.2 Implement user login endpoint

Implemented JWT-based authentication with the following features:
- POST /api/auth/login endpoint
- Email and password validation
- JWT token generation with 24-hour expiry
- Secure password comparison using bcrypt

Refs: #42"
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

**Example:**
```bash
git commit -m "test(auth): add comprehensive tests for user authentication

Added comprehensive test coverage:
- Property tests for login with 100+ random inputs
- Unit tests for invalid credentials, expired tokens, edge cases
- Coverage improved from 75% to 92%

Coverage metrics:
- Statements: 92%
- Branches: 90%
- Functions: 94%
- Lines: 92%"
```

**Requirements**:
- Type must be "test"
- Should reference the same scope as implementation
- Body should summarize all testing work (property tests, unit tests, coverage)

#### Commit 3: Documentation

```bash
git commit -m "docs(<scope>): update documentation for <context>

<optional body with documentation details>
- Updated inline comments for new functions
- Added API documentation for endpoints
- Updated README with usage examples

<optional footer>"
```

**Example:**
```bash
git commit -m "docs(auth): update documentation for authentication system

Updated documentation:
- Added JSDoc comments for all authentication functions
- Updated API documentation with login endpoint details
- Added authentication flow diagram to README
- Updated troubleshooting section with common issues"
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

<optional footer>"
```

**Example:**
```bash
git commit -m "chore(auth): apply code quality fixes for authentication

Applied comprehensive quality improvements:
- Fixed 12 ESLint warnings
- Applied Prettier formatting to 8 files
- Fixed 3 TypeScript type errors
- Removed 2 unused imports
- Verified build succeeds
- All pre-commit hooks passing"
```

**Requirements**:
- Type must be "chore"
- Should reference the same scope as implementation
- Body should summarize all quality improvements

### Step 6: Verify Consolidated Commits

After creating all 4 commits, verify the commit history is clean:

```bash
# View the 4 new commits
git log origin/main..HEAD --oneline

# Verify exactly 4 commits exist
git rev-list --count origin/main..HEAD
```

**Expected output**:
```
4
```

**Verify each commit**:
- Commit 1: Implementation (feat|fix|refactor|perf)
- Commit 2: Testing (test)
- Commit 3: Documentation (docs)
- Commit 4: Quality (chore)

**If commit count is not 4**:
- Review what went wrong
- Reset and try again: `git reset --soft origin/main`
- Ensure all 4 commits are created

### Step 7: Push Branch to Origin

Push the consolidated commits to the remote repository:

```bash
# Push branch with upstream tracking
git push -u origin HEAD
```

**If push fails** (e.g., branch already exists with different history):
```bash
# Force push with lease (safer than --force)
git push --force-with-lease origin HEAD
```

**Verify push succeeded**:
```bash
# Check remote branch
git branch -vv

# Verify commits on remote
git log origin/$(git branch --show-current) --oneline -5
```

### Step 8: Generate PR Title and Body

Create a descriptive PR title and body based on the implementation commit:

**PR Title Format**:
```
<type>(<scope>): <task-title>
```

**Example PR titles**:
- `feat(auth): Implement user login endpoint`
- `fix(api): Fix authentication token validation`
- `refactor(database): Restructure query builders`

**PR Body Format**:
```markdown
## Task Reference

Task: <task-id> <task-title>
Spec: <spec-name>

## Summary

<Brief description of what was implemented>

## Changes

### Implementation
<Summary of implementation changes>

### Testing
<Summary of testing additions>
- Property-based tests: <count> properties tested
- Unit tests: <count> test cases added
- Coverage: <percentage>% (statements, branches, functions, lines)

### Documentation
<Summary of documentation updates>

### Quality
<Summary of quality improvements>

## Related Issues

Refs: #<issue-number> (if applicable)

## Breaking Changes

<Description of breaking changes, if any>
```

**Example PR body**:
```markdown
## Task Reference

Task: 1.2 Implement user login endpoint
Spec: user-authentication

## Summary

Implemented JWT-based authentication system with secure login endpoint, comprehensive testing, and full documentation.

## Changes

### Implementation
- Added POST /api/auth/login endpoint
- Implemented JWT token generation with 24-hour expiry
- Added email and password validation
- Integrated bcrypt for secure password comparison

### Testing
- Property-based tests: 3 properties tested with 100+ iterations each
- Unit tests: 15 test cases covering happy path, edge cases, and error conditions
- Coverage: 92% (statements: 92%, branches: 90%, functions: 94%, lines: 92%)

### Documentation
- Added JSDoc comments for all authentication functions
- Updated API documentation with endpoint details
- Added authentication flow diagram to README
- Updated troubleshooting section

### Quality
- Fixed 12 ESLint warnings
- Applied Prettier formatting to 8 files
- Fixed 3 TypeScript type errors
- All pre-commit hooks passing

## Related Issues

Refs: #42

## Breaking Changes

None
```

### Step 9: Create Pull Request

Use the gh cli to create the pull request:

```bash
# Create PR with title and body
gh pr create \
  --title "<pr-title>" \
  --body "<pr-body>"
```

**If PR creation fails**:
- Check if gh cli is authenticated: `gh auth status`
- Check if remote repository is configured: `git remote -v`
- Check if branch is pushed: `git branch -vv`
- Report error to user with gh cli output

**Capture PR number and URL**:
```bash
# Get PR number
gh pr view --json number -q .number

# Get PR URL
gh pr view --json url -q .url
```

### Step 10: Add Labels to PR

Add appropriate labels based on the commit types present:

**Label mapping**:
- feat → "enhancement"
- fix → "bug"
- refactor → "refactor"
- perf → "performance"
- test → "testing"
- docs → "documentation"
- chore → "maintenance"

**Add labels**:
```bash
# Add labels based on commit types
gh pr edit <pr-number> --add-label "enhancement,testing,documentation,maintenance"
```

**Example for feat commit**:
```bash
gh pr edit $(gh pr view --json number -q .number) \
  --add-label "enhancement,testing,documentation,maintenance"
```

**Verify labels were added**:
```bash
gh pr view --json labels -q '.labels[].name'
```

### Step 11: Report Success

Report the PR creation success to the user:

```
Pull request created successfully!

PR #<number>: <title>
URL: <url>

Commits:
1. <implementation-commit-summary>
2. <testing-commit-summary>
3. <documentation-commit-summary>
4. <quality-commit-summary>

Labels: <label-list>

The PR is now ready for review. CI/CD checks will be monitored automatically.
```

## Commit Format

The consolidated commits must follow these formats:

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

<optional footer>
```

## Success Criteria

Verify that all of the following are true before completing:

1. ✅ All quality agents have completed (or user confirmed to proceed)
2. ✅ Task status is marked as "completed" in tasks.md
3. ✅ Soft reset was performed successfully (all changes staged)
4. ✅ Exactly 4 commits were created in the correct order
5. ✅ All commits follow Conventional Commits format
6. ✅ Branch was pushed to origin successfully
7. ✅ PR was created with descriptive title and body
8. ✅ Appropriate labels were added to the PR
9. ✅ PR number and URL were captured and reported
10. ✅ Commit history is clean (no WIP, fixup, or iterative commits)

## Error Handling

### Quality Agents Not Completed

**Scenario**: Some quality agents have not completed their work

**Action**:
- List which quality agents appear to be missing (no commits)
- Ask user if they want to:
  1. Wait for quality agents to complete
  2. Proceed with PR creation anyway
  3. Cancel PR submission

**Example response**:
```
Warning: Some quality agents may not have completed.

Expected quality commits:
✓ Documentation (docs)
✓ Testing (test)
✗ Linting (chore)
✗ Formatting (chore)
✗ Type checking (chore)

Options:
1. Wait for remaining quality agents to complete
2. Proceed with PR creation (missing quality checks)
3. Cancel PR submission

How would you like to proceed?
```

### Task Status Update Fails

**Scenario**: taskStatus tool fails to mark task as completed

**Action**:
- Report the error with details
- Verify the taskFilePath is correct
- Verify the task string matches exactly
- Continue with PR creation (task status is not critical)

**Example response**:
```
Warning: Failed to update task status to "completed"

Details:
- taskFilePath: .kiro/specs/user-authentication/tasks.md
- task: "1.2 Implement user login endpoint"
- status: "completed"

This is not critical. Continuing with PR creation...

You can manually update the task status later.
```

### Soft Reset Fails

**Scenario**: git reset --soft fails or produces unexpected results

**Action**:
- Capture the git error output
- Check if origin/main exists and is up-to-date
- Suggest fetching latest: `git fetch origin`
- Do not proceed until reset succeeds

**Example response**:
```
Error: Soft reset failed

Git output:
fatal: ambiguous argument 'origin/main': unknown revision or path not in the working tree

Possible causes:
- origin/main doesn't exist or hasn't been fetched
- Remote is not configured correctly

Suggested fix:
git fetch origin
git reset --soft origin/main

Please resolve the git issue before proceeding.
```

### Commit Creation Fails

**Scenario**: One or more consolidated commits fail to create

**Action**:
- Identify which commit failed
- Check if there are changes to commit: `git status`
- If no changes for a commit type (e.g., no docs changes), skip that commit
- Adjust to create only the commits that have changes

**Example response**:
```
Note: No documentation changes detected.

Creating 3 commits instead of 4:
1. Implementation commit (feat)
2. Testing commit (test)
3. Quality commit (chore)

Skipping documentation commit (no changes).
```

### Push Fails

**Scenario**: git push fails due to conflicts or permissions

**Action**:
- Capture the git error output
- Check if branch already exists remotely with different history
- Suggest force push with lease: `git push --force-with-lease`
- If permissions issue, report to user

**Example response**:
```
Error: Push failed

Git output:
! [rejected]        feat/1.2-user-login -> feat/1.2-user-login (non-fast-forward)

The remote branch has different history. This is expected after consolidating commits.

Using force push with lease to update remote branch...
git push --force-with-lease origin HEAD

Force push successful.
```

### PR Creation Fails

**Scenario**: gh pr create fails

**Action**:
- Capture the gh cli error output
- Check authentication: `gh auth status`
- Check if PR already exists: `gh pr list --head $(git branch --show-current)`
- If PR exists, update it instead: `gh pr edit`
- Report error to user if unrecoverable

**Example response**:
```
Error: PR creation failed

gh cli output:
pull request create failed: a pull request for branch "feat/1.2-user-login" already exists

A PR already exists for this branch. Updating existing PR instead...

gh pr edit <pr-number> --title "<new-title>" --body "<new-body>"

PR updated successfully.
```

### Label Addition Fails

**Scenario**: gh pr edit fails to add labels

**Action**:
- Capture the gh cli error output
- Check if labels exist in repository: `gh label list`
- If labels don't exist, skip label addition (non-critical)
- Report warning to user

**Example response**:
```
Warning: Failed to add labels to PR

gh cli output:
label "enhancement" not found in repository

Labels may not exist in this repository. Skipping label addition.

PR created successfully without labels. You can add labels manually:
gh pr edit <pr-number> --add-label "enhancement"
```

### Commit Count Incorrect

**Scenario**: After consolidation, commit count is not 4

**Action**:
- Check actual commit count: `git rev-list --count origin/main..HEAD`
- If less than 4, some commit types had no changes (acceptable)
- If more than 4, something went wrong - reset and retry
- Report the actual commit structure to user

**Example response**:
```
Note: Created 3 commits instead of 4.

Commit structure:
1. feat(auth): 1.2 Implement user login endpoint
2. test(auth): add comprehensive tests for user authentication
3. chore(auth): apply code quality fixes for authentication

Documentation commit was skipped (no documentation changes).

This is acceptable. Proceeding with PR creation...
```

### No Changes to Commit

**Scenario**: After soft reset, there are no changes in staging area

**Action**:
- Verify the branch has commits: `git log origin/main..HEAD`
- Check if changes were lost: `git status`
- Report error to user - this should not happen
- Do not proceed with PR creation

**Example response**:
```
Error: No changes to commit after soft reset

This should not happen. The soft reset should preserve all changes.

Possible causes:
- Branch has no commits (nothing was implemented)
- Changes were lost (git error)

Please investigate the branch state:
git log origin/main..HEAD
git status

Do not proceed with PR creation until this is resolved.
```

### gh CLI Not Authenticated

**Scenario**: gh cli is not authenticated

**Action**:
- Check authentication status: `gh auth status`
- Provide authentication instructions
- Do not proceed until authenticated

**Example response**:
```
Error: gh cli is not authenticated

To authenticate:
gh auth login

Follow the prompts to authenticate with GitHub.

After authentication, re-run the PR submission.
```

## Notes

- **Clean History**: The soft reset and re-commit process ensures PRs have clean, logical commit history without iterative "fix" commits.
- **Exactly 4 Commits**: The consolidated history should have exactly 4 commits (or fewer if some types have no changes).
- **Conventional Commits**: All commits must follow Conventional Commits format for consistency.
- **Force Push Safety**: Always use `--force-with-lease` instead of `--force` to prevent overwriting others' work.
- **Task Completion**: Marking the task as "completed" is important for workflow tracking but not critical for PR creation.
- **Label Flexibility**: If labels don't exist in the repository, skip label addition (it's not critical).
- **PR Body Detail**: The PR body should provide comprehensive information about all changes for reviewers.
- **Idempotent**: This process should be idempotent - running it twice should produce the same result.
