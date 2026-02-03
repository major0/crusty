#!/bin/sh
set -eu
POSIXLY_CORRECT='no bashing shell'

# Error handling function
error() { echo "::error::$*"; exit 1; }

# Generate changelog between tags
# Usage: generate-changelog.sh <current_tag>

CURRENT_TAG="${1}"

# Find previous release tag (not RC)
# Get all non-RC tags, sort them, and find the one before current tag
ALL_TAGS="$(git tag -l "v*" | grep -v "rc" | sort -V)"
PREV_TAG=""

# Iterate through tags to find the one before current
for tag in ${ALL_TAGS}; do
  if test "${tag}" = "${CURRENT_TAG}"; then
    break
  fi
  PREV_TAG="${tag}"
done

if test -z "${PREV_TAG}"; then
  # No previous tag, use first commit
  PREV_TAG="$(git rev-list --max-parents=0 HEAD)"
  echo "No previous release tag found, using first commit: ${PREV_TAG}"
else
  echo "Previous release tag: ${PREV_TAG}"
fi

echo "Generating changelog from ${PREV_TAG} to ${CURRENT_TAG}"

{
  echo "# Changelog for ${CURRENT_TAG}"
  echo ""
  echo "## Changes since ${PREV_TAG}"
  echo ""
  git log --pretty=format:"- %s (%h)" "${PREV_TAG}..${CURRENT_TAG}"
} > CHANGELOG.md || error "Failed to generate changelog"

echo "Changelog generated successfully"
cat CHANGELOG.md
