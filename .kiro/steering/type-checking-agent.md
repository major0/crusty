---
inclusion: manual
---

# Type Checking Agent

## Purpose

Verify Rust type safety and fix type errors using the Rust compiler. The type checking agent runs `cargo check` to ensure all type constraints are satisfied and applies fixes for common type errors.

## Context

You have access to:
- **Modified Rust files**: All .rs files changed during implementation
- **Rust compiler**: rustc via cargo check
- **Implementation commit**: The commit message and changes from the implementation
- **Task details**: tasks.md, requirements.md, and design.md files
- **All Kiro tools**: File operations, git commands, Rust compiler

## Instructions

### Step 1: Run Cargo Check

Execute cargo check to verify type safety:

```bash
# Run type checking (faster than full build)
cargo check

# Check all targets
cargo check --all-targets

# Check with specific features
cargo check --all-features
```

**cargo check behavior**:
- Checks code for type errors without generating binaries
- Faster than `cargo build`
- Verifies all type constraints are satisfied
- Checks trait bounds, lifetimes, and borrowing rules

### Step 2: Analyze Type Errors

Review type errors from cargo check:

**Common Rust type errors**:
- **Type mismatches**: Expected type X, found type Y
- **Trait bounds not satisfied**: Type doesn't implement required trait
- **Lifetime errors**: Borrowed value doesn't live long enough
- **Ownership errors**: Value moved or borrowed incorrectly
- **Missing type annotations**: Compiler can't infer type
- **Method not found**: Type doesn't have the method

**Example type errors**:
```
error[E0308]: mismatched types
  --> src/parser.rs:45:9
   |
45 |     let x: i32 = "hello";
   |            ---   ^^^^^^^ expected `i32`, found `&str`
   |            |
   |            expected due to this

error[E0277]: the trait bound `MyType: Display` is not satisfied
  --> src/codegen.rs:78:5
   |
78 |     println!("{}", value);
   |                    ^^^^^ `MyType` cannot be formatted with the default formatter
```

### Step 3: Apply Automatic Fixes

For auto-fixable type errors:

**Missing type annotations**:
```rust
// Error: type annotations needed
// Before:
let value = parse_number();

// After:
let value: i32 = parse_number();
```

**Type conversions**:
```rust
// Error: expected `String`, found `&str`
// Before:
let s: String = "hello";

// After:
let s: String = "hello".to_string();
```

**Trait implementations**:
```rust
// Error: trait bound not satisfied
// Before:
#[derive(Debug)]
struct MyType { ... }

// After:
#[derive(Debug, Display)]
struct MyType { ... }
```

### Step 4: Re-run Type Checking

After applying fixes:

```bash
cargo check
```

**If no errors**: Proceed to commit fixes
**If errors remain**: Analyze remaining errors (max 2 attempts)

### Step 5: Handle No Errors Scenario

If cargo check succeeds with no errors:

1. Verify check actually ran
2. Report: "No type errors found - Rust type checking successful"
3. Do not create a commit
4. Exit successfully

### Step 6: Commit Type Fixes

If type errors were fixed:

```bash
git add .
git commit -m "chore(<scope>): fix type errors for <context>"
```

**Examples**:
- `chore(parser): fix type errors for parser module`
- `chore(codegen): fix type errors for code generation`
- `chore(semantic): fix type errors for semantic analysis`

**Scope guidelines**:
- Use the same scope as implementation commit
- Use module name (parser, codegen, semantic, etc.)

## Commit Format

```
chore(<scope>): fix type errors for <context>

<optional body>
- Fixed X type errors in Y files
- Added missing type annotations
- Fixed trait bound issues
- Resolved lifetime errors

<optional footer>
Rust version: <version>
```

## Success Criteria

1. ✅ Cargo check executed successfully
2. ✅ All type errors analyzed and categorized
3. ✅ Auto-fixable errors fixed
4. ✅ Changes reviewed and verified as type-safe
5. ✅ No functional changes introduced
6. ✅ Cargo check passes (no type errors)
7. ✅ Commit message follows format (if fixes applied)
8. ✅ Changes committed (or noted as not needed)
9. ✅ Remaining type errors reported to user

## Error Handling

### Cargo Check Fails

**Scenario**: cargo check fails to run (not type errors, but execution failure)

**Action**:
- Capture error output
- Identify likely cause
- Suggest remediation steps
- Do not proceed until cargo check runs successfully

### Type Errors Cannot Be Auto-Fixed

**Scenario**: Type errors exist but cannot be automatically fixed

**Action**:
- Commit any auto-fixes that were applied
- Report remaining errors to user with details
- Provide guidance on fixing remaining errors
- Exit successfully after committing auto-fixes

**Example response**:
```
Type checking completed with partial auto-fix.

Auto-fixed and committed:
- Added 5 missing type annotations
- Fixed 3 type conversions
- Added 2 trait derivations

Remaining issues (require manual fix):
- src/parser.rs:45 - Lifetime error: borrowed value doesn't live long enough
  Suggestion: Adjust lifetime annotations or restructure code
  
- src/codegen.rs:78 - Trait bound not satisfied: `MyType: Display`
  Suggestion: Implement Display trait for MyType or use Debug

These issues require architectural decisions and cannot be auto-fixed.
Please review and address manually.
```

### No Changes After Type Checking

**Scenario**: cargo check runs successfully with no errors

**Action**:
- Verify check ran (check output)
- Report: "No type errors found"
- Do not create a commit
- Exit successfully

## Notes

- **Type Safety First**: Rust's type system prevents many bugs at compile time
- **Preserve Functionality**: Type fixes should not change code behavior
- **Borrow Checker**: Rust's borrow checker enforces memory safety
- **Lifetimes**: Lifetime errors often require restructuring code
- **Trait Bounds**: Trait bounds ensure types have required capabilities
- **No Suppression**: Don't use type casts to bypass type errors
- **Cargo Check**: Faster than cargo build for type checking
- **CI/CD Integration**: Type checking is verified in CI/CD pipeline
