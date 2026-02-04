---
inclusion: manual
---

# Security Audit Agent

## Purpose

Check for security vulnerabilities, exposed secrets, and other security issues in modified Rust code using cargo-audit and other security scanning tools. The security audit agent automatically detects security issues and applies fixes where possible, reporting issues that require manual intervention.

## Context

You have access to:
- **Modified code files**: All Rust (.rs) files changed during implementation
- **Security scanning tools**: cargo-audit, cargo-deny, clippy security lints
- **Implementation commit**: The commit message and changes from the implementation
- **Task details**: tasks.md, requirements.md, and design.md files
- **All Kiro tools**: File operations, git commands, security scanning tools

## Instructions

### Step 1: Identify Security Scanning Tools

This project uses the following security tools:

**cargo-audit** - Checks for known vulnerabilities in dependencies:
```bash
# Install cargo-audit
cargo install cargo-audit

# Check version
cargo audit --version
```

**cargo-deny** (optional) - Checks licenses, bans, and advisories:
```bash
# Install cargo-deny
cargo install cargo-deny

# Check version
cargo deny --version
```

**Clippy security lints** - Built-in Rust linter with security checks:
```bash
# Clippy is already installed with rustup
cargo clippy --version
```

### Step 2: Identify Modified Rust Files

Determine which Rust files were modified:

```bash
# View implementation commit
git log -1 --stat

# Get list of modified Rust files
git diff --name-only HEAD~1 | grep '\.rs$'
```

Focus security scanning on:
- **Source files**: Files in src/
- **Build scripts**: build.rs
- **Configuration files**: Cargo.toml, Cargo.lock

**Skip**:
- Generated files in target/
- External dependencies

### Step 3: Scan for Exposed Secrets

Check for accidentally committed secrets or credentials:

**Common secret patterns to detect**:
- API keys
- Private keys
- Passwords and tokens
- Database connection strings

**Manual pattern search**:
```bash
# Search for common secret patterns in modified files
git diff HEAD~1 | grep -iE "(api[_-]?key|secret|password|token|private[_-]?key)"
```

**If secrets are found**:
1. **Remove the secret** immediately
2. **Replace with environment variable** or secure configuration
3. **Rotate the secret** (invalidate and generate new one)
4. **Report to user** with details

### Step 4: Scan for Dependency Vulnerabilities

Check for known vulnerabilities in Rust dependencies:

**Using cargo-audit**:
```bash
# Run cargo-audit
cargo audit

# With detailed output
cargo audit --json
```

**cargo-audit checks**:
- Known vulnerabilities in dependencies
- Unmaintained crates
- Yanked crates
- Security advisories from RustSec database

**If vulnerabilities found**:
- Categorize by severity (critical, high, medium, low)
- Attempt automatic fixes with `cargo update`
- Report unfixable vulnerabilities to user

### Step 5: Scan for Code-Level Security Issues

Check for common security vulnerabilities in Rust code:

**Using Clippy security lints**:
```bash
# Run clippy with security-focused lints
cargo clippy -- -W clippy::suspicious

# Run with all warnings as errors
cargo clippy -- -D warnings
```

**Common Rust security issues**:
- **Unsafe code**: Unnecessary use of `unsafe` blocks
- **Panic in production**: Unwrap/expect that could panic
- **Integer overflow**: Arithmetic without overflow checks
- **Path traversal**: Unsanitized file paths
- **Command injection**: Unsanitized command arguments
- **Insecure randomness**: Use of weak RNG for security
- **Memory safety**: Potential memory leaks or use-after-free

**Example security checks**:
```rust
// ❌ Bad: Unwrap could panic
let value = some_option.unwrap();

// ✅ Good: Proper error handling
let value = some_option.ok_or("Value not found")?;

// ❌ Bad: Unsafe without justification
unsafe {
    *ptr = value;
}

// ✅ Good: Safe alternative
*ptr_ref = value;
```

### Step 6: Analyze Security Scan Results

Review security scan results and categorize issues:

**Critical Issues** (must be fixed):
- Exposed secrets or credentials
- High-severity dependency vulnerabilities
- Unsafe code without justification
- Potential memory safety violations

**High-Priority Issues** (should be fixed):
- Medium-severity dependency vulnerabilities
- Unwrap/expect that could panic in production
- Integer overflow potential
- Path traversal vulnerabilities

**Low-Priority Issues** (can be deferred):
- Low-severity dependency vulnerabilities
- Code quality issues with security implications
- Deprecated security functions

### Step 7: Apply Automatic Fixes

For issues that can be automatically fixed:

1. **Dependency updates**:
   ```bash
   # Update specific dependency
   cargo update <crate-name>
   
   # Update all dependencies
   cargo update
   ```

2. **Code fixes**: Apply clippy suggestions
   ```bash
   cargo clippy --fix --allow-dirty --allow-staged
   ```

3. **Verify fixes don't break functionality**:
   ```bash
   cargo test
   ```

### Step 8: Handle Manual Fixes

For issues requiring manual intervention:

**Action**:
- Capture security issue details
- Analyze severity and impact
- For critical issues: report immediately and halt workflow
- For high-priority issues: report with recommendations
- For low-priority issues: log warnings and continue

**Example response for critical issue**:
```
CRITICAL SECURITY ISSUE FOUND:

Issue: Unsafe code without safety documentation
Location: src/parser.rs:45
Code: unsafe { *ptr = value; }

This unsafe block lacks:
1. Safety documentation explaining why it's safe
2. Justification for using unsafe
3. Invariants that must be maintained

Unsafe code must be:
1. Documented with SAFETY comments
2. Justified with clear reasoning
3. Minimized to smallest possible scope

Workflow halted. Please fix this issue before proceeding.
```

### Step 9: Handle No Issues Scenario

If no security issues found:

1. Verify scans actually ran
2. Report: "No security issues found - code passes security audit"
3. Do not create a commit
4. Exit successfully

### Step 10: Commit Security Fixes

If security fixes were applied:

```bash
git add .
git commit -m "chore(<scope>): apply security fixes for <context>"
```

**Examples**:
- `chore(deps): apply security fixes for dependency vulnerabilities`
- `chore(parser): apply security fixes for parser module`
- `chore(codegen): apply security fixes for code generation`

**Scope guidelines**:
- Use "deps" for dependency vulnerability fixes
- Use module name for code-level security fixes

## Commit Format

```
chore(<scope>): apply security fixes for <context>

<optional body>
- Fixed X dependency vulnerabilities
- Removed unsafe code from Y
- Applied security patches for Z

<optional footer>
Security tools used: cargo-audit, clippy
```

## Success Criteria

1. ✅ Security scanning tools executed successfully
2. ✅ No exposed secrets or credentials in modified files
3. ✅ No critical or high-severity dependency vulnerabilities
4. ✅ No critical code-level security vulnerabilities
5. ✅ Automatic fixes applied where available
6. ✅ Changes reviewed and verified as security improvements
7. ✅ No functional changes introduced (only security fixes)
8. ✅ Commit message follows format (if fixes applied)
9. ✅ Changes committed (or noted as not needed)
10. ✅ Critical issues reported to user (if any remain)

## Error Handling

### Security Tools Not Available

**Scenario**: cargo-audit or other tools not installed

**Action**:
- Report error
- Suggest installation: `cargo install cargo-audit`
- Perform manual security review
- Report findings to user

### Security Tool Execution Fails

**Scenario**: Security scanning tool fails to run

**Action**:
- Capture error output
- Identify likely cause
- Suggest remediation steps
- Continue with other security checks if possible

### Critical Security Issue Found

**Scenario**: Critical vulnerability discovered

**Action**:
- Report issue immediately with full details
- Halt workflow (do not proceed to PR creation)
- Provide specific remediation steps
- Do not commit code with critical security issues

**Example response**:
```
CRITICAL SECURITY ISSUE - WORKFLOW HALTED

Issue Type: High-severity dependency vulnerability
Crate: regex
Version: 1.5.4
Vulnerability: CVE-2022-24713 (ReDoS)
Severity: High

IMMEDIATE ACTIONS REQUIRED:
1. Update regex crate: cargo update regex
2. Verify tests still pass: cargo test
3. Review any regex patterns for complexity

DO NOT MERGE THIS CODE until the issue is resolved.

Workflow halted. Please fix this critical issue.
```

### Dependency Vulnerabilities Found

**Scenario**: cargo-audit reports dependency vulnerabilities

**Action**:
- Categorize vulnerabilities by severity
- Attempt automatic fixes with `cargo update`
- Report unfixable vulnerabilities to user
- Commit automatic fixes if successful

**Example response**:
```
Dependency vulnerabilities found.

Severity breakdown:
- Critical: 0
- High: 1
- Medium: 2
- Low: 1

Attempting automatic fixes...

Fixed vulnerabilities:
- regex: 1.5.4 → 1.7.0 (CVE-2022-24713 - High)
- serde_json: 1.0.85 → 1.0.91 (Medium)

Remaining vulnerabilities (require manual fix):
- time: 0.1.44 (Unmaintained - Medium)
  Fix: Migrate to time 0.3.x or chrono
  Note: Requires code changes

Committed automatic fixes. Please review remaining vulnerabilities.
```

### Unsafe Code Without Documentation

**Scenario**: Unsafe code lacks safety documentation

**Action**:
- Report all unsafe blocks without SAFETY comments
- Explain requirements for unsafe code
- Halt workflow if unsafe code is unjustified
- Request user to add documentation or remove unsafe

## Notes

- **Security First**: Security issues should block PR creation if critical
- **Secrets Never Committed**: Exposed secrets should halt workflow immediately
- **Dependency Updates**: Be cautious - updates may introduce breaking changes
- **Unsafe Code**: Rust's unsafe code requires careful review and documentation
- **Test After Fixes**: Always run tests after applying security fixes
- **Rotate Secrets**: If secrets exposed, they must be rotated (invalidated)
- **Document Exceptions**: If security issues can't be fixed, document why
- **Multiple Tools**: Use multiple security tools for comprehensive coverage
- **CI/CD Integration**: Security checks are also run in CI/CD pipeline
- **RustSec Database**: cargo-audit uses RustSec advisory database
