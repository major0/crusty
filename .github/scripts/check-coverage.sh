#!/bin/sh
set -eu
POSIXLY_CORRECT='no bashing shell'

# Error handling function
error() { echo "::error::$*" >&2; exit 1; }

# Convert float to integer for comparison
unfloat() {
  # Validate input is a number or float
  case "${1}" in
  ([0-9]|[0-9][0-9]*)
    # Valid integer pattern (single digit or multiple digits)
    ;;
  ([0-9]*.[0-9]|[0-9]*.[0-9][0-9]*)
    # Valid float pattern (at least one digit after decimal)
    ;;
  (*)
    error "unfloat: invalid input '${1}' - must be a number or float"
    ;;
  esac
  
  set -- "$(printf '%.02f' "${1}")"
  set -- "${1##0}"
  echo "${1%.*}${1#*.}"
}

# Check coverage thresholds
# Usage: check-coverage.sh <min_line> <min_branch> <min_function> <coverage_json> <output_file>

MIN_LINE="${1}"
MIN_BRANCH="${2}"
MIN_FUNCTION="${3}"
COVERAGE_JSON="${4:-coverage.json}"
OUTPUT_FILE="${5:-/dev/stdout}"

# Parse coverage metrics
LINE_COV="$(jq -r '.data[0].totals.lines.percent' "${COVERAGE_JSON}")"
BRANCH_COV="$(jq -r '.data[0].totals.branches.percent' "${COVERAGE_JSON}")"
FUNC_COV="$(jq -r '.data[0].totals.functions.percent' "${COVERAGE_JSON}")"

# Output informational messages to stderr
echo "Coverage metrics:" >&2
echo "  Line coverage: ${LINE_COV}%" >&2
echo "  Branch coverage: ${BRANCH_COV}%" >&2
echo "  Function coverage: ${FUNC_COV}%" >&2

# Output for GitHub Actions (to output file)
{
  echo "line-coverage=${LINE_COV}"
  echo "branch-coverage=${BRANCH_COV}"
  echo "function-coverage=${FUNC_COV}"
} >> "${OUTPUT_FILE}"

# Check thresholds using unfloat for comparison
LINE_COV_INT="$(unfloat "${LINE_COV}")"
MIN_LINE_INT="$(unfloat "${MIN_LINE}")"
if ! test "${LINE_COV_INT}" -ge "${MIN_LINE_INT}"; then
  error "Line coverage ${LINE_COV}% is below threshold ${MIN_LINE}%"
fi

BRANCH_COV_INT="$(unfloat "${BRANCH_COV}")"
MIN_BRANCH_INT="$(unfloat "${MIN_BRANCH}")"
if ! test "${BRANCH_COV_INT}" -ge "${MIN_BRANCH_INT}"; then
  error "Branch coverage ${BRANCH_COV}% is below threshold ${MIN_BRANCH}%"
fi

FUNC_COV_INT="$(unfloat "${FUNC_COV}")"
MIN_FUNCTION_INT="$(unfloat "${MIN_FUNCTION}")"
if ! test "${FUNC_COV_INT}" -ge "${MIN_FUNCTION_INT}"; then
  error "Function coverage ${FUNC_COV}% is below threshold ${MIN_FUNCTION}%"
fi

echo "All coverage thresholds met!" >&2
