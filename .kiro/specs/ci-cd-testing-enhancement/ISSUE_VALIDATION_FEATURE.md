# Issue Reference Validation for Release Branch PRs

**Date**: 2026-02-02  
**Feature**: Issue tracking validation for release workflows  
**Status**: Specified (not yet implemented)

## Overview

This feature adds mandatory issue reference validation for all pull requests targeting release branches. It ensures that every release-related change is properly tracked and linked to an open GitHub issue, improving release management and traceability.

## Motivation

Release branches require strict change management to ensure:
- All changes are documented and justified
- Changes can be traced back to specific issues or feature requests
- Release notes can be automatically generated from linked issues
- Release managers can verify all changes have proper approval

## Requirements Added

### Requirement 14.3-14.6: Issue Reference Validation

**New Acceptance Criteria**:

3. WHEN a pull request targets a branch matching the pattern release/vX.Y, THE CI_System SHALL validate that at least one commit in the PR contains an issue reference

4. WHEN validating issue references, THE CI_System SHALL accept commits containing keywords "close", "closes", "closed", "fix", "fixes", "fixed", "resolve", "resolves", "resolved", or "related-to" followed by a hash symbol and issue number

5. WHEN an issue reference is found, THE CI_System SHALL verify the referenced issue exists and is in an open state

6. WHEN a pull request to a release branch does not contain a valid issue reference, THE CI_System SHALL fail the validation check and prevent merging

### Glossary Addition

**Issue_Reference**: A commit message pattern that links a commit to a GitHub issue using keywords like "close", "fix", or "related-to" followed by an issue number

## Design Implementation

### Workflow: release-candidate.yml

Added a new `validate-issue-reference` job that runs before all other checks:

```yaml
validate-issue-reference:
  runs-on: ubuntu-latest
  steps:
    - Checkout with full history
    - Get PR commits between base and head
    - Search for issue reference pattern: (close[sd]?|fix(e[sd])?|resolve[sd]?|related-to)\s+#([0-9]+)
    - Extract issue numbers from matches
    - Use gh CLI to verify at least one issue exists and is OPEN
    - Fail if no valid open issue reference found
```

**Job Dependencies**:
- `ci-checks` depends on `validate-issue-reference`
- `create-rc-tag` depends on `ci-checks`

This ensures validation happens first and blocks all subsequent steps if it fails.

## Supported Keywords

The validation accepts the following keywords (case-insensitive):

### Closing Keywords
- `close #123`
- `closes #123`
- `closed #123`
- `fix #123`
- `fixes #123`
- `fixed #123`
- `resolve #123`
- `resolves #123`
- `resolved #123`

### Reference Keywords
- `related-to #123`

## Validation Logic

1. **Pattern Matching**: Search all commit messages in the PR for issue references
2. **Issue Extraction**: Extract all issue numbers from matched patterns
3. **State Verification**: For each issue number:
   - Check if issue exists using `gh issue view`
   - Verify issue state is "OPEN"
   - Continue checking until at least one valid open issue is found
4. **Failure Conditions**:
   - No issue references found in any commit
   - All referenced issues are closed or don't exist
   - GitHub API errors (treated as validation failure)

## Example Commit Messages

### Valid Examples ✅

```
feat(release): add new feature for v1.2

This implements the new authentication flow.

Closes #456
```

```
fix(release): resolve critical bug

Related-to #789
```

```
chore(release): update dependencies

Fixes #123, closes #124
```

### Invalid Examples ❌

```
feat(release): add new feature

No issue reference at all
```

```
fix(release): resolve bug

Closes #999
(where issue #999 is closed or doesn't exist)
```

## Testing Strategy

### Property Test (Property 11)

**Property**: Issue Reference Validation

*For any* pull request targeting a release branch, at least one commit message should contain a valid issue reference using keywords followed by a hash symbol and issue number, and the referenced issue should exist and be in an open state.

**Validates**: Requirements 14.3, 14.4, 14.5, 14.6

**Test Approach**:
- Generate commit message strings with proptest
- Verify pattern matching works correctly
- Verify validation logic checks for open issue state
- Run 100 iterations

### Unit Tests

Added to task 9.5:
- Verify release-candidate.yml includes validate-issue-reference job
- Verify issue validation checks for all supported keywords
- Verify issue validation uses gh CLI to check issue state
- Verify issue validation fails if no open issue reference found
- Verify ci-checks job depends on validate-issue-reference

## Implementation Tasks

### Task 6.1 Updates

Added implementation details for the validate-issue-reference job:
- Fetch all history and get PR commits between base and head
- Search commit messages for issue references using regex pattern
- Extract issue numbers from matched references
- Use gh CLI to verify at least one referenced issue exists and is OPEN
- Fail validation if no valid open issue reference is found

### Task 8.12 (New)

Write property test for issue reference validation:
- Use proptest to generate commit message strings
- Parse release-candidate.yml workflow
- Verify issue reference pattern matches correctly
- Verify validation logic checks for open issue state
- Configure test to run 100 iterations

## Benefits

1. **Traceability**: Every release change is linked to a documented issue
2. **Accountability**: Changes must be justified and approved via issues
3. **Automation**: Validation happens automatically in CI, no manual review needed
4. **Release Notes**: Linked issues can be used to generate release notes
5. **Quality Control**: Prevents ad-hoc changes from entering release branches

## Migration Considerations

### For Existing Release Branches

When implementing this feature:
1. Ensure all open release PRs have issue references
2. Create tracking issues for any PRs without references
3. Update PR descriptions to include issue links
4. Communicate the new requirement to all contributors

### For New Release Branches

All new PRs to release branches will automatically be validated. Contributors should:
1. Create an issue before starting work on release changes
2. Reference the issue in commit messages using supported keywords
3. Ensure the issue remains open until the PR is merged

## Error Messages

The validation provides clear error messages:

```
::error::No issue reference found in PR commits
::error::Release branch PRs must reference an issue using: close #N, fix #N, or related-to #N
```

```
::error::No valid open issue reference found
::error::At least one referenced issue must exist and be open
```

## Future Enhancements

Potential improvements for future iterations:

1. **Issue Label Validation**: Require specific labels on referenced issues (e.g., "release")
2. **Milestone Validation**: Verify issue is assigned to the correct milestone
3. **Multiple Issue Support**: Require multiple issues for complex changes
4. **Custom Keywords**: Allow repository-specific keywords via configuration
5. **Issue Template Enforcement**: Require issues to follow specific templates

## Related Documentation

- Requirements: `.kiro/specs/ci-cd-testing-enhancement/requirements.md` (Requirement 14)
- Design: `.kiro/specs/ci-cd-testing-enhancement/design.md` (Entry-Point Workflow 2, Property 11)
- Tasks: `.kiro/specs/ci-cd-testing-enhancement/tasks.md` (Tasks 6.1, 8.12, 9.5)

---

**Status**: Ready for implementation as part of Task 6.1
