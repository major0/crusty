#!/bin/sh
# Pre-commit hook wrapper for crustyc syntax checking
# Validates Crusty source files one at a time

set -e

exit_code=0
temp_dir="$(mktemp -d)"
trap 'rm -rf "$temp_dir"' EXIT

for file in "$@"; do
    basename="$(basename "${file}" .crst)"
    # Use || to handle failure without triggering set -e
    cargo run --bin crustyc -- --emit=ast "${file}" > "${temp_dir}/${basename}.ast" 2>&1 || {
        echo "Error: Syntax check failed for ${file}" >&2
        exit_code=1
    }
done

exit "${exit_code}"
