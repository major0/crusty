---
title: Shell Script Standards
description: Standards and best practices for writing POSIX-compliant shell scripts
inclusion: manual
---

# Shell Script Standards

This document defines the standards and best practices for writing shell scripts in this project. All shell scripts must be POSIX-compliant and follow these guidelines to ensure portability, reliability, and maintainability.

## Core Principles

### 1. POSIX Compliance

**Use POSIX shell (`/bin/sh`), not bash:**
```sh
#!/bin/sh
# NOT: #!/bin/bash or #!/usr/bin/env bash
```

**Set POSIXLY_CORRECT (but do not export):**
```sh
set -eu
POSIXLY_CORRECT='no bashing shell'
# Do NOT export POSIXLY_CORRECT
```

**Why not export?**
- Exporting can affect child processes unexpectedly
- Setting it locally is sufficient for the current shell
- Prevents unintended side effects in called programs

### 2. No Bashisms

Avoid bash-specific features:

❌ **Don't use:**
- `[[  ]]` - bash conditional expressions
- `[  ]` - bracket test syntax (it's a symlink to `/bin/test`)
- `==` - bash comparison operator
- `pipefail` - bash-specific set option
- Arrays (bash-specific)
- `${var^^}` or `${var,,}` - bash case conversion
- `&>` - bash redirection shorthand
- `source` - use `.` instead

✅ **Do use:**
- `test` command for all conditionals
- `=` for string equality
- POSIX parameter expansion
- Standard POSIX utilities

### 3. Use `test` Command

Always use the `test` command instead of bracket syntax:

```sh
# ❌ Wrong - bracket syntax
if [ -z "$VAR" ]; then
if [[ "$VAR" == "value" ]]; then

# ✅ Correct - test command
if test -z "$VAR"; then
if test "$VAR" = "value"; then
```

**Common test operators:**
- `-z STRING` - string is empty
- `-n STRING` - string is not empty
- `STRING1 = STRING2` - strings are equal (use `=`, not `==`)
- `STRING1 != STRING2` - strings are not equal
- `-eq`, `-ne`, `-lt`, `-le`, `-gt`, `-ge` - integer comparisons
- `-f FILE` - file exists and is regular file
- `-d DIR` - directory exists
- `-e PATH` - path exists

### 4. Quote All Parameter Expansions

**Always quote variable expansions and command substitutions:**

```sh
# ❌ Wrong - unquoted
VAR=$(command)
echo $VAR
if test -z $VAR; then

# ✅ Correct - quoted
VAR="$(command)"
echo "$VAR"
if test -z "$VAR"; then
```

**This includes:**
- Variable expansions: `"${VAR}"`
- Command substitutions: `"$(command)"`
- Parameter expansions: `"${VAR#prefix}"`
- All uses in test conditions

**Why quote?**
- Prevents word splitting
- Prevents globbing
- Handles empty values correctly
- Prevents injection vulnerabilities

### 5. Error Handling

**Define a standard error function:**

```sh
# Error handling function
error() { echo "::error::$*"; exit 1; }
```

**Use it consistently:**

```sh
# ❌ Wrong - inconsistent error handling
if ! git tag "$TAG"; then
  echo "Error: Failed to create tag"
  exit 1
fi

# ✅ Correct - use error function
git tag "$TAG" || error "Failed to create tag $TAG"
```

**Benefits:**
- Consistent error formatting
- GitHub Actions annotations (::error::)
- Single exit point
- Cleaner code

### 6. Float Comparisons

Shell arithmetic only supports integers. For float comparisons, use the `unfloat()` function:

**Define the unfloat function:**

```sh
# Convert float to integer for comparison
unfloat() {
  set -- "$(printf '%.02f' "${1}")"
  set -- "${1##0}"
  echo "${1}" | sed -e 's/\.//'
}
```

**Use it to compare floats:**

```sh
# ❌ Wrong - cannot compare floats directly
if test "$COVERAGE" -lt "$THRESHOLD"; then

# ❌ Wrong - don't use awk
if awk -v cov="$COVERAGE" -v min="$THRESHOLD" 'BEGIN { exit !(cov < min) }'; then

# ✅ Correct - use unfloat function
COVERAGE_INT="$(unfloat "${COVERAGE}")"
THRESHOLD_INT="$(unfloat "${THRESHOLD}")"
if test "${COVERAGE_INT}" -lt "${THRESHOLD_INT}"; then
```

**How unfloat() works:**
1. `printf '%.02f'` - formats to 2 decimal places (90.5 → 90.50)
2. `${1##0}` - removes leading zero if present (0.50 → .50)
3. `sed 's/\.//'` - removes decimal point (90.50 → 9050)
4. Result: comparable integer maintaining precision

**Benefits:**
- Consistent precision (always 2 decimal places)
- Handles edge cases (0.5, .5, 90, 90.0, 90.50)
- Pure POSIX shell (no external tools except sed)
- Reusable function

### 7. ShellCheck Compliance

**Always run shellcheck with all checks enabled:**

```sh
# Run shellcheck on all scripts
shellcheck .github/scripts/*.sh
```

**Never disable shellcheck warnings:**

```sh
# ❌ Wrong - disabling checks
# shellcheck disable=SC2086
echo $VAR

# ✅ Correct - fix the issue
echo "$VAR"
```

**Configure shellcheck properly:**

Create `.shellcheckrc` with minimal exclusions:
```sh
# Only exclude checks that are truly not applicable
# Most projects should have NO exclusions
```

## Complete Example

Here's a complete example following all standards:

```sh
#!/bin/sh
set -eu
POSIXLY_CORRECT='no bashing shell'

# Error handling function
error() { echo "::error::$*"; exit 1; }

# Convert float to integer for comparison
unfloat() {
  set -- "$(printf '%.02f' "${1}")"
  set -- "${1##0}"
  echo "${1}" | sed -e 's/\.//'
}

# Script description
# Usage: example.sh <input> <threshold>

INPUT="${1}"
THRESHOLD="${2}"

# Validate input
if test -z "$INPUT"; then
  error "Input parameter is required"
fi

# Get value from command
VALUE="$(some_command "$INPUT")"

# Convert float to integer for comparison
VALUE_INT="$(unfloat "${VALUE}")"
THRESHOLD_INT="$(unfloat "${THRESHOLD}")"

# Compare values
if test "$VALUE_INT" -lt "$THRESHOLD_INT"; then
  error "Value $VALUE is below threshold $THRESHOLD"
fi

# Success
echo "Value $VALUE meets threshold $THRESHOLD"
```

## Common Patterns

### Checking if a variable is set

```sh
if test -z "$VAR"; then
  error "VAR is not set"
fi
```

### Checking if a file exists

```sh
if ! test -f "$FILE"; then
  error "File $FILE does not exist"
fi
```

### Looping over items

```sh
for ITEM in $LIST; do
  echo "Processing: $ITEM"
done
```

### Conditional execution

```sh
command || error "Command failed"
command && echo "Command succeeded"
```

### String comparison

```sh
if test "$VAR1" = "$VAR2"; then
  echo "Variables are equal"
fi

if test "$VAR1" != "$VAR2"; then
  echo "Variables are different"
fi
```

### Integer comparison

```sh
if test "$NUM1" -lt "$NUM2"; then
  echo "$NUM1 is less than $NUM2"
fi

if test "$NUM1" -eq "$NUM2"; then
  echo "Numbers are equal"
fi
```

## Testing Shell Scripts

### Manual Testing

```sh
# Test with shellcheck
shellcheck script.sh

# Test execution
sh script.sh arg1 arg2

# Test with POSIXLY_CORRECT
POSIXLY_CORRECT=1 sh script.sh arg1 arg2
```

### Pre-commit Integration

Ensure shellcheck runs on all shell scripts:

```yaml
# .pre-commit-config.yaml
- repo: https://github.com/shellcheck-py/shellcheck-py
  rev: v0.9.0.6
  hooks:
    - id: shellcheck
```

## References

- [POSIX Shell Command Language](https://pubs.opengroup.org/onlinepubs/9699919799/utilities/V3_chap02.html)
- [ShellCheck](https://www.shellcheck.net/)
- [Google Shell Style Guide](https://google.github.io/styleguide/shellguide.html)
- [Bash Pitfalls](https://mywiki.wooledge.org/BashPitfalls)

## Summary Checklist

When writing shell scripts, ensure:

- ✅ Shebang is `#!/bin/sh`
- ✅ Set `POSIXLY_CORRECT` (not exported)
- ✅ Use `set -eu` (not `set -euo pipefail`)
- ✅ Use `test` instead of `[` or `[[`
- ✅ Quote all parameter expansions: `"$(cmd)"`
- ✅ Define and use `error()` function
- ✅ Define and use `unfloat()` for float comparisons
- ✅ Convert floats to integers for comparison
- ✅ No awk, bc, or other external tools for arithmetic
- ✅ All shellcheck warnings fixed
- ✅ No disabled shellcheck rules
