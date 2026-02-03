# GitHub Actions Scripts

This directory contains reusable shell scripts for GitHub Actions workflows. Extracting scripts from workflows provides several benefits:

- **Maintainability**: Scripts are easier to read, test, and modify
- **Reusability**: Scripts can be called from multiple workflows
- **Testability**: Scripts can be tested locally without running workflows
- **Linting**: Scripts can be linted with shellcheck for quality assurance

## Scripts

### validate-issue-reference.sh
Validates that PR commits contain valid issue references (close, fix, resolve, related-to).

**Usage:**
```bash
./validate-issue-reference.sh <base_ref> <head_ref>
```

**Environment:**
- Requires `GH_TOKEN` for GitHub CLI

### verify-release-tag.sh
Verifies release tag placement on correct branch and version ancestry.

**Usage:**
```bash
./verify-release-tag.sh <tag>
```

**Outputs:** major, minor, patch, version

### create-rc-tag.sh
Creates release candidate tags with automatic version incrementing.

**Usage:**
```bash
./create-rc-tag.sh <branch>
```

**Outputs:** tag

### update-version-alias.sh
Updates version alias tags (vX and vX.Y) to point to latest releases.

**Usage:**
```bash
./update-version-alias.sh <tag>
```

**Outputs:** minor-alias, major-alias

### generate-changelog.sh
Generates changelog between release tags.

**Usage:**
```bash
./generate-changelog.sh <current_tag>
```

**Outputs:** CHANGELOG.md file

### check-coverage.sh
Checks code coverage against thresholds.

**Usage:**
```bash
./check-coverage.sh <min_line> <min_branch> <min_function> <coverage_json>
```

**Outputs:** line-coverage, branch-coverage, function-coverage

## Linting

All scripts are linted with shellcheck. Configuration is in `.shellcheckrc`.

To lint locally:
```bash
shellcheck .github/scripts/*.sh
```

## Testing

Scripts can be tested locally by setting up the required environment:

```bash
# Example: Test coverage check
./check-coverage.sh 90 90 90 coverage.json
```

## Adding New Scripts

When adding new scripts:

1. Create the script in this directory
2. Make it executable: `chmod +x script.sh`
3. Add usage documentation to this README
4. Test locally before committing
5. Run shellcheck to ensure quality
