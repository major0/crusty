#!/bin/sh
set -eu
POSIXLY_CORRECT='no bashing shell'

# Error handling function
error() { echo "::error::$*"; exit 1; }

# Validate that PR commits contain valid issue references
# Usage: validate-issue-reference.sh <base_ref> <head_ref>

BASE_REF="${1:-origin/main}"
HEAD_REF="${2:-HEAD}"

# Get commit messages
git log --pretty=format:"%s%n%b" "${BASE_REF}..${HEAD_REF}" > commits.txt || error "Failed to get commit messages"

echo "Commits in PR:"
cat commits.txt

# Pattern for issue references
# Matches: close #123, closes #123, closed #123, fix #123, fixes #123, 
#          fixed #123, resolve #123, resolves #123, resolved #123, related-to #123
PATTERN="(close[sd]?|fix(e[sd])?|resolve[sd]?|related-to)\s+#([0-9]+)"

# Search for issue references
ISSUE_NUMS="$(grep -oiP "${PATTERN}" commits.txt | grep -oP "#\K[0-9]+" || echo "")"

if test -z "${ISSUE_NUMS}"; then
  rm -f commits.txt
  error "No issue reference found in PR commits. Release branch PRs must reference an issue using: close #N, fix #N, or related-to #N"
fi

echo "Found issue references: ${ISSUE_NUMS}"

# Verify at least one issue is open
VALID_ISSUE_FOUND=false
for ISSUE_NUM in ${ISSUE_NUMS}; do
  echo "Checking issue #${ISSUE_NUM}..."
  
  # Get issue state
  ISSUE_STATE="$(gh issue view "${ISSUE_NUM}" --json state --jq '.state' 2>/dev/null || echo "NOT_FOUND")"
  
  if test "${ISSUE_STATE}" = "NOT_FOUND"; then
    echo "::warning::Issue #${ISSUE_NUM} does not exist"
    continue
  fi
  
  if test "${ISSUE_STATE}" = "OPEN"; then
    echo "::notice::Valid open issue found: #${ISSUE_NUM}"
    VALID_ISSUE_FOUND=true
    break
  else
    echo "::warning::Issue #${ISSUE_NUM} is not open (state: ${ISSUE_STATE})"
  fi
done

rm -f commits.txt

if test "${VALID_ISSUE_FOUND}" = false; then
  error "No valid open issue reference found. At least one referenced issue must exist and be open"
fi

echo "Issue reference validation passed"
