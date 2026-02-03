#!/bin/sh
set -eu
POSIXLY_CORRECT='no bashing shell'

# Error handling function
error() { echo "::error::$*"; exit 1; }

# Create release candidate tag
# Usage: create-rc-tag.sh <branch>

BRANCH="${1}"

# Extract X.Y from release/vX.Y
MAJOR_MINOR="$(echo "${BRANCH}" | sed 's/release\/v//')"

echo "Creating RC tag for version ${MAJOR_MINOR}"

# Find latest vX.Y.N tag (non-RC)
LATEST_RELEASE="$(git tag -l "v${MAJOR_MINOR}.*" | grep -v "rc" | sort -V | tail -n1)"

if test -z "${LATEST_RELEASE}"; then
  # No release yet, start with .0
  NEXT_PATCH=0
else
  # Extract N from vX.Y.N and increment
  CURRENT_PATCH="${LATEST_RELEASE#v"${MAJOR_MINOR}".}"
  NEXT_PATCH=$((CURRENT_PATCH + 1))
fi

NEXT_VERSION="${MAJOR_MINOR}.${NEXT_PATCH}"

# Find latest rcM for this version
LATEST_RC="$(git tag -l "v${NEXT_VERSION}-rc*" | sort -V | tail -n1)"

if test -z "${LATEST_RC}"; then
  # No RC yet, start with rc1
  RC_NUM=1
else
  # Extract M from vX.Y.Z-rcM and increment
  CURRENT_RC="${LATEST_RC#v"${NEXT_VERSION}"-rc}"
  RC_NUM=$((CURRENT_RC + 1))
fi

TAG="v${NEXT_VERSION}-rc${RC_NUM}"
git tag "${TAG}" || error "Failed to create tag ${TAG}"
git push origin "${TAG}" || error "Failed to push tag ${TAG}"

echo "Created release candidate tag: ${TAG}"
echo "tag=${TAG}"
