#!/bin/sh
set -eu
POSIXLY_CORRECT='no bashing shell'

# Error handling function
error() { echo "::error::$*" >&2; exit 1; }

# Update version alias tags (vX.Y and vX)
# Usage: update-version-alias.sh <tag> <output_file>

TAG="${1}"
OUTPUT_FILE="${2:-/dev/stdout}"

# Extract version components
VERSION="${TAG#v}"
MAJOR="$(echo "${VERSION}" | cut -d. -f1)"
MINOR="$(echo "${VERSION}" | cut -d. -f2)"
MAJOR_MINOR="${MAJOR}.${MINOR}"

echo "Updating version aliases for ${TAG} (Major: ${MAJOR}, Minor: ${MINOR})" >&2

# Update vX.Y alias tag
ALIAS_TAG="v${MAJOR_MINOR}"

# Find latest patch version for this major.minor
LATEST_PATCH="$(git tag -l "v${MAJOR_MINOR}.*" | grep -v "rc" | sort -V | tail -n1)"

if test "${LATEST_PATCH}" = "${TAG}"; then
  echo "Creating/updating alias tag ${ALIAS_TAG} to point to ${TAG}" >&2
  git tag -f "${ALIAS_TAG}" "${TAG}" || error "Failed to create alias tag ${ALIAS_TAG}"
  git push -f origin "${ALIAS_TAG}" || error "Failed to push alias tag ${ALIAS_TAG}"
  echo "Updated ${ALIAS_TAG} → ${TAG}" >&2
  echo "minor-alias=${ALIAS_TAG}" >> "${OUTPUT_FILE}"
else
  echo "Tag ${TAG} is not the latest for v${MAJOR_MINOR}, skipping alias update" >&2
  echo "minor-alias=" >> "${OUTPUT_FILE}"
fi

# Update vX alias tag
ALIAS_TAG="v${MAJOR}"

# Find latest version for this major version
LATEST_VERSION="$(git tag -l "v${MAJOR}.*" | grep -v "rc" | sort -V | tail -n1)"

if test "${LATEST_VERSION}" = "${TAG}"; then
  echo "Creating/updating alias tag ${ALIAS_TAG} to point to ${TAG}" >&2
  git tag -f "${ALIAS_TAG}" "${TAG}" || error "Failed to create alias tag ${ALIAS_TAG}"
  git push -f origin "${ALIAS_TAG}" || error "Failed to push alias tag ${ALIAS_TAG}"
  echo "Updated ${ALIAS_TAG} → ${TAG}" >&2
  echo "major-alias=${ALIAS_TAG}" >> "${OUTPUT_FILE}"
else
  echo "Tag ${TAG} is not the latest for v${MAJOR}, skipping alias update" >&2
  echo "major-alias=" >> "${OUTPUT_FILE}"
fi
