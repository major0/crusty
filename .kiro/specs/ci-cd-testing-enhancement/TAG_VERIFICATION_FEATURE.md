# Release Tag Verification Feature

**Date**: 2026-02-02  
**Feature**: Tag placement and version ancestry verification  
**Status**: Specified (not yet implemented)

## Overview

This feature adds automated verification of release tag placement and version history continuity. It ensures that release tags are correctly placed on their corresponding release branches and that version history follows a linear progression, preventing out-of-order or misplaced releases.

## Motivation

Release tags must maintain strict version history to ensure:
- Tags are placed on the correct release branch
- Version history is linear and traceable
- Each release builds upon the previous release
- No releases are created from incorrect branches or commits
- Release artifacts correspond to the correct codebase state

Without verification, it's possible to:
- Tag a commit that isn't on the release branch
- Create v1.2.3 without v1.2.2 existing
- Create v1.2.3 from a commit that doesn't include v1.2.2's changes
- Break the linear version history

## Requirements Added

### Requirement 14.15-14.19: Release Tag Verification

**New Acceptance Criteria**:

15. WHEN a release publication workflow is triggered by a tag vX.Y.Z, THE CI_System SHALL verify the tagged commit exists within the corresponding release/vX.Y branch

16. WHEN verifying tag placement, THE CI_System SHALL check that the tagged commit is reachable from the release/vX.Y branch head

17. WHEN a release tag vX.Y.Z is created where Z is greater than zero, THE CI_System SHALL verify that the previous release tag vX.Y.(Z-1) is a direct ancestor of the newly tagged commit

18. WHEN a release tag vX.Y.0 is created for a new release branch, THE CI_System SHALL automatically create the tag without requiring ancestor verification

19. WHEN tag verification fails, THE CI_System SHALL fail the release publication workflow and report which validation check failed

## Design Implementation

### Workflow: release-publish.yml

Added a new `verify-tag` job that runs before all other checks:

```yaml
verify-tag:
  runs-on: ubuntu-latest
  steps:
    - Checkout with full history (fetch-depth: 0)
    - Extract version components (major, minor, patch) from tag
    - Verify tag is on release/vX.Y branch:
      * Check release branch exists
      * Get commit that tag points to
      * Use git merge-base --is-ancestor to verify commit is on branch
    - Verify version ancestry (if patch > 0):
      * Find previous tag vX.Y.(Z-1)
      * Verify previous tag exists
      * Use git merge-base --is-ancestor to verify previous tag is ancestor
    - Skip ancestry check for vX.Y.0 tags
```

**Job Dependencies**:
- `ci-checks` depends on `verify-tag`
- All subsequent jobs depend on `ci-checks`

This ensures verification happens first and blocks all subsequent steps if it fails.

## Verification Logic

### 1. Branch Placement Verification

**Purpose**: Ensure the tagged commit is on the correct release branch

**Steps**:
1. Extract major.minor from tag (e.g., v1.2.3 → 1.2)
2. Construct release branch name: `release/v{major}.{minor}`
3. Verify release branch exists
4. Get commit SHA that the tag points to
5. Check if commit is reachable from release branch head:
   ```bash
   git merge-base --is-ancestor $TAG_COMMIT origin/$RELEASE_BRANCH
   ```

**Failure Conditions**:
- Release branch doesn't exist
- Tagged commit is not reachable from release branch
- Tagged commit is on a different branch

### 2. Version Ancestry Verification

**Purpose**: Ensure version history is linear and continuous

**Steps**:
1. Extract patch version from tag (e.g., v1.2.3 → 3)
2. If patch == 0, skip ancestry check (initial release)
3. If patch > 0:
   - Calculate previous version: vX.Y.(Z-1)
   - Verify previous tag exists
   - Get commit SHA for previous tag
   - Check if previous commit is ancestor of current commit:
     ```bash
     git merge-base --is-ancestor $PREV_TAG_COMMIT $TAG_COMMIT
     ```

**Failure Conditions**:
- Previous tag doesn't exist (e.g., creating v1.2.3 without v1.2.2)
- Previous tag is not an ancestor (version history is not linear)
- Commits were cherry-picked or rebased incorrectly

### 3. Special Case: Initial Release (vX.Y.0)

**Purpose**: Allow first release in a series without ancestry check

**Behavior**:
- Only branch placement is verified
- No previous tag is required
- Ancestry check is skipped
- Automatically passes if on correct branch

**Rationale**: The first release in a series (v1.2.0) has no predecessor, so ancestry verification doesn't apply.

## Example Scenarios

### Valid Scenario 1: Sequential Releases ✅

```
release/v1.2 branch:
  A --- B --- C --- D --- E
        ↑           ↑       ↑
      v1.2.0     v1.2.1  v1.2.2

Creating v1.2.2:
- Tag points to commit E
- Commit E is on release/v1.2 ✓
- Previous tag v1.2.1 exists ✓
- v1.2.1 (commit D) is ancestor of E ✓
- Verification passes ✓
```

### Valid Scenario 2: Initial Release ✅

```
release/v1.3 branch (new):
  A --- B --- C
              ↑
            v1.3.0

Creating v1.3.0:
- Tag points to commit C
- Commit C is on release/v1.3 ✓
- Patch version is 0, skip ancestry check ✓
- Verification passes ✓
```

### Invalid Scenario 1: Wrong Branch ❌

```
main branch:
  A --- B --- C --- D
              ↑
            v1.2.1 (wrong!)

release/v1.2 branch:
  A --- B --- E --- F

Creating v1.2.1:
- Tag points to commit C
- Commit C is NOT on release/v1.2 ✗
- Verification fails ✗
```

### Invalid Scenario 2: Missing Previous Version ❌

```
release/v1.2 branch:
  A --- B --- C --- D
        ↑             ↑
      v1.2.0       v1.2.2 (wrong!)

Creating v1.2.2:
- Tag points to commit D
- Commit D is on release/v1.2 ✓
- Previous tag v1.2.1 does NOT exist ✗
- Verification fails ✗
```

### Invalid Scenario 3: Non-Linear History ❌

```
release/v1.2 branch:
  A --- B --- C --- D
        ↑       ↑
      v1.2.0  v1.2.2 (wrong!)
      
  E --- F
        ↑
      v1.2.1 (on different branch)

Creating v1.2.2:
- Tag points to commit C
- Commit C is on release/v1.2 ✓
- Previous tag v1.2.1 exists ✓
- v1.2.1 (commit F) is NOT ancestor of C ✗
- Verification fails ✗
```

## Error Messages

The verification provides clear, actionable error messages:

### Branch Placement Errors

```
::error::Release branch release/v1.2 does not exist
```

```
::error::Tag v1.2.3 (commit abc123) is not on release branch release/v1.2
::error::The tagged commit must be reachable from the release branch head
```

### Ancestry Errors

```
::error::Previous release tag v1.2.2 does not exist
::error::Cannot verify version history continuity
```

```
::error::Previous release v1.2.2 is not an ancestor of v1.2.3
::error::Release history must be linear - each version must build on the previous
::error::Previous: def456, Current: abc123
```

## Testing Strategy

### Property Test (Property 14)

**Property**: Release Tag Verification

*For any* release tag vX.Y.Z where Z > 0, the tagged commit should be reachable from the release/vX.Y branch, and the previous release tag vX.Y.(Z-1) should be a direct ancestor of the tagged commit. For vX.Y.0 tags, only branch reachability is required.

**Validates**: Requirements 14.15, 14.16, 14.17, 14.18, 14.19

**Test Approach**:
- Generate release tag scenarios with varying patch versions
- Verify branch placement check is performed
- Verify ancestry check is performed for Z > 0
- Verify ancestry check is skipped for vX.Y.0
- Run 100 iterations

### Unit Tests

Added to task 9.5:
- Verify release-publish.yml includes verify-tag job that runs first
- Verify tag verification extracts version components from tag
- Verify tag verification checks commit is on release/vX.Y branch
- Verify tag verification uses git merge-base --is-ancestor for branch check
- Verify tag verification checks previous tag ancestry for Z > 0
- Verify tag verification skips ancestry check for vX.Y.0 tags
- Verify ci-checks job depends on verify-tag

## Benefits

1. **Version Integrity**: Ensures version history is linear and traceable
2. **Branch Correctness**: Prevents tags from being placed on wrong branches
3. **Release Safety**: Blocks releases that don't build on previous versions
4. **Automation**: Verification happens automatically, no manual checks needed
5. **Clear Errors**: Provides actionable error messages when verification fails
6. **History Preservation**: Maintains clean, understandable version history

## Implementation Tasks

### Task 6.2 Updates

Added implementation details for the verify-tag job:
- Fetch all history and extract version components from tag
- Verify tagged commit exists within corresponding release/vX.Y branch
- Check that tagged commit is reachable from release branch head
- For tags vX.Y.Z where Z > 0, verify previous tag vX.Y.(Z-1) exists
- For tags vX.Y.Z where Z > 0, verify previous tag is direct ancestor
- Skip ancestry check for vX.Y.0 tags
- Fail workflow if any verification check fails

### Task 8.15 (New)

Write property test for release tag verification:
- Use proptest to generate release tag scenarios
- Parse release-publish.yml workflow
- Verify tag verification checks commit is on release branch
- Verify ancestry check is performed for Z > 0
- Verify ancestry check is skipped for vX.Y.0 tags
- Configure test to run 100 iterations

## Git Commands Used

### Check if commit is on branch
```bash
git merge-base --is-ancestor $COMMIT $BRANCH
```
Returns 0 if commit is ancestor of branch, non-zero otherwise.

### Get commit for tag
```bash
git rev-list -n 1 $TAG
```
Returns the commit SHA that the tag points to.

### Check if branch exists
```bash
git rev-parse --verify $BRANCH
```
Returns 0 if branch exists, non-zero otherwise.

## Future Enhancements

Potential improvements for future iterations:

1. **Signed Tag Verification**: Require GPG signatures on release tags
2. **Commit Message Validation**: Verify release commits follow specific format
3. **Changelog Verification**: Ensure changelog is updated for each release
4. **Branch Protection**: Automatically protect release branches
5. **Tag Immutability Enforcement**: Prevent tag deletion or modification
6. **Release Notes Validation**: Require release notes for each version

## Related Documentation

- Requirements: `.kiro/specs/ci-cd-testing-enhancement/requirements.md` (Requirement 14.15-14.19)
- Design: `.kiro/specs/ci-cd-testing-enhancement/design.md` (Entry-Point Workflow 3, Property 14)
- Tasks: `.kiro/specs/ci-cd-testing-enhancement/tasks.md` (Tasks 6.2, 8.15, 9.5)

---

**Status**: Ready for implementation as part of Task 6.2
