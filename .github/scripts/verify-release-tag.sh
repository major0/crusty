#!/bin/sh
set -eu
POSIXLY_CORRECT='no bashing shell'

# Error handling function
error() { echo "::error::$*" >&2; exit 1; }

# Verify release tag placement and ancestry
# Usage: verify-release-tag.sh <tag> <output_file>

TAG="${1}"
OUTPUT_FILE="${2:-/dev/stdout}"

# Extract version components
VERSION="${TAG#v}"
MAJOR="$(echo "${VERSION}" | cut -d. -f1)"
MINOR="$(echo "${VERSION}" | cut -d. -f2)"
PATCH="$(echo "${VERSION}" | cut -d. -f3)"

echo "Tag: ${TAG}" >&2
echo "Version: ${VERSION} (Major: ${MAJOR}, Minor: ${MINOR}, Patch: ${PATCH})" >&2

# Verify tag is on release branch
RELEASE_BRANCH="release/v${MAJOR}.${MINOR}"

echo "Verifying tag ${TAG} is on branch ${RELEASE_BRANCH}" >&2

# Check if release branch exists
if ! git rev-parse --verify "origin/${RELEASE_BRANCH}" >/dev/null 2>&1; then
  error "Release branch ${RELEASE_BRANCH} does not exist"
fi

# Get the commit that the tag points to
TAG_COMMIT="$(git rev-list -n 1 "${TAG}")"

# Check if tag commit is reachable from release branch
if ! git merge-base --is-ancestor "${TAG_COMMIT}" "origin/${RELEASE_BRANCH}"; then
  error "Tag ${TAG} (commit ${TAG_COMMIT}) is not on release branch ${RELEASE_BRANCH}. The tagged commit must be reachable from the release branch head"
fi

echo "::notice::Tag ${TAG} is correctly placed on branch ${RELEASE_BRANCH}" >&2

# Verify version ancestry
# Skip ancestry check for vX.Y.0 (first release in series)
if test "${PATCH}" = "0"; then
  echo "::notice::Skipping ancestry check for initial release ${TAG}" >&2
  {
    echo "major=${MAJOR}"
    echo "minor=${MINOR}"
    echo "patch=${PATCH}"
    echo "version=${VERSION}"
  } >> "${OUTPUT_FILE}"
  exit 0
fi

# Find previous version
PREV_PATCH=$((PATCH - 1))
PREV_TAG="v${MAJOR}.${MINOR}.${PREV_PATCH}"

echo "Checking if previous tag ${PREV_TAG} exists and is ancestor of ${TAG}" >&2

# Check if previous tag exists
if ! git rev-parse --verify "${PREV_TAG}" >/dev/null 2>&1; then
  error "Previous release tag ${PREV_TAG} does not exist. Cannot verify version history continuity"
fi

# Get commits for both tags
PREV_TAG_COMMIT="$(git rev-list -n 1 "${PREV_TAG}")"

# Verify previous tag is ancestor of current tag
if ! git merge-base --is-ancestor "${PREV_TAG_COMMIT}" "${TAG_COMMIT}"; then
  error "Previous release ${PREV_TAG} is not an ancestor of ${TAG}. Release history must be linear - each version must build on the previous. Previous: ${PREV_TAG_COMMIT}, Current: ${TAG_COMMIT}"
fi

echo "::notice::Version ancestry verified: ${PREV_TAG} is ancestor of ${TAG}" >&2

# Output version components for use in workflow
{
  echo "major=${MAJOR}"
  echo "minor=${MINOR}"
  echo "patch=${PATCH}"
  echo "version=${VERSION}"
} >> "${OUTPUT_FILE}"
