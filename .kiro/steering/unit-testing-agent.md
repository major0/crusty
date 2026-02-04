---
inclusion: manual
---

# Unit Testing Agent

## Purpose

Create unit tests for specific examples and edge cases that verify correct behavior of individual functions, modules, and components. Unit tests complement property-based tests by testing concrete scenarios, boundary conditions, and error handling using Rust's built-in test framework.

## Context

You have access to:
- **Implementation code**: All Rust files changed during implementation
- **Design document**: Contains architecture and component specifications
- **Requirements document**: Contains acceptance criteria and functional requirements
- **Implementation commit**: The commit message and changes from the implementation
- **Task details**: tasks.md, requirements.md, and design.md files
- **All Kiro tools**: File operations, git commands, cargo test

## Instructions

### Step 1: Identify What to Test

Review the implementation to identify testable units:

```bash
# View implementation commit
git log -1 --stat

# View actual changes
git diff HEAD~1
```

Look for:
- **New functions**: Each public function should have unit tests
- **New modules**: Each module should have tests
- **Edge cases**: Boundary values, empty inputs, None/Some handling
- **Error conditions**: Invalid inputs, Result::Err cases, panics

### Step 2: Rust Testing Framework

This project uses Rust's built-in testing framework:

**Test organization**:
- Inline tests: `#[cfg(test)]` modules in source files
- Separate test files: Files in `tests/` directory
- Integration tests: Files in `tests/` for public API testing

**Test attributes**:
- `#[test]` - Mark function as test
- `#[should_panic]` - Test should panic
- `#[ignore]` - Skip test unless explicitly run

### Step 3: Create Unit Test Modules

Add test modules to source files or create separate test files:

**Inline tests** (preferred for unit tests):
```rust
// In src/parser.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_function() {
        let input = "int add(int a, int b) { return a + b; }";
        let result = parse_function(input);
        assert!(result.is_ok());
    }
}
```

**Separate test files** (for integration tests):
```rust
// In tests/parser_tests.rs

use crustyc::parser::*;

#[test]
fn test_parse_complex_function() {
    // Test implementation
}
```

### Step 4: Write Unit Tests

For each testable unit, write focused tests:

1. **Test happy path** - Verify correct behavior with valid inputs:
   ```rust
   #[test]
   fn test_parse_valid_function() {
       let input = "void main() {}";
       let result = parse_function(input);
       assert!(result.is_ok());
       let func = result.unwrap();
       assert_eq!(func.name, "main");
       assert_eq!(func.return_type, "void");
   }
   ```

2. **Test edge cases** - Verify behavior at boundaries:
   ```rust
   #[test]
   fn test_parse_empty_function() {
       let input = "void empty() {}";
       let result = parse_function(input);
       assert!(result.is_ok());
   }
   
   #[test]
   fn test_parse_function_with_many_params() {
       let input = "int func(int a, int b, int c, int d) {}";
       let result = parse_function(input);
       assert!(result.is_ok());
       assert_eq!(result.unwrap().params.len(), 4);
   }
   ```

3. **Test error conditions** - Verify proper error handling:
   ```rust
   #[test]
   fn test_parse_invalid_syntax() {
       let input = "invalid syntax here";
       let result = parse_function(input);
       assert!(result.is_err());
   }
   
   #[test]
   #[should_panic(expected = "unexpected token")]
   fn test_parse_panics_on_invalid_token() {
       parse_function_unchecked("@#$%");
   }
   ```

4. **Test special cases** - Empty inputs, None values, special characters:
   ```rust
   #[test]
   fn test_parse_empty_input() {
       let result = parse_function("");
       assert!(result.is_err());
   }
   
   #[test]
   fn test_parse_with_special_chars() {
       let input = "void func_with_underscore() {}";
       let result = parse_function(input);
       assert!(result.is_ok());
   }
   ```

### Step 5: Write Descriptive Test Names

Use clear, descriptive test names:

**Good test names**:
- `test_parse_valid_function`
- `test_parse_returns_error_on_invalid_syntax`
- `test_codegen_handles_empty_block`
- `test_semantic_analysis_detects_type_mismatch`

**Bad test names**:
- `test1`
- `works`
- `parser_test`
- `test_function`

### Step 6: Keep Tests Focused and Minimal

Follow these principles:

1. **One assertion per test** (when possible)
2. **Test one thing** - Each test verifies one behavior
3. **Avoid over-testing** - Don't test standard library functionality
4. **No mocks for simple tests** - Use real implementations
5. **Use test helpers** - Extract common setup into helper functions

### Step 7: Run Unit Tests

Execute tests to verify they pass:

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_parse_valid_function

# Run tests in specific module
cargo test parser::tests

# Run with output
cargo test -- --nocapture

# Run ignored tests
cargo test -- --ignored
```

If tests fail:
1. Analyze failure message
2. Check implementation for bugs
3. Check test for incorrect expectations
4. Fix issue
5. Re-run tests

### Step 8: Validate Test Coverage

Ensure unit tests cover important scenarios:

1. Check all public functions are tested
2. Check edge cases are covered
3. Check error conditions are tested
4. Verify test names are clear
5. Ensure tests are independent

### Step 9: Commit Unit Tests

If unit tests were created:

```bash
git add .
git commit -m "test(<scope>): add unit tests for <context>"
```

**Examples**:
- `test(parser): add unit tests for function parsing`
- `test(codegen): add unit tests for code generation`
- `test(semantic): add unit tests for type checking`

**Scope guidelines**:
- Use the same scope as implementation commit
- Use module name (parser, codegen, semantic, etc.)

## Commit Format

```
test(<scope>): add unit tests for <context>

<optional body>
- Added tests for X function (happy path, edge cases, errors)
- Added tests for Y module (various scenarios)
- Verified all tests pass

<optional footer>
Validates: Requirements X.Y, X.Z
```

## Success Criteria

1. ✅ All new public functions have unit tests
2. ✅ All new modules have tests
3. ✅ Edge cases tested (boundaries, empty inputs, None values)
4. ✅ Error conditions tested (invalid inputs, panics)
5. ✅ Test names are clear and descriptive
6. ✅ Tests are focused and minimal
7. ✅ All unit tests pass successfully
8. ✅ Commit message follows format
9. ✅ Changes committed (or noted as not needed)

## Error Handling

### No Testable Code

**Scenario**: Implementation contains no testable logic

**Action**:
- Verify by reviewing implementation changes
- Report: "No unit tests needed - implementation contains no testable logic"
- Do not create tests or commits
- Exit successfully

### Test Failures

**Scenario**: Unit tests fail after creation

**Action**:
- Capture test failure output
- Analyze if bug is in implementation or test
- Fix implementation or test as needed
- Re-run tests

**Example response**:
```
Unit test failed:

Test: test_parse_valid_function
Error: assertion failed: result.is_ok()
  left: Err(ParseError("unexpected token"))

This appears to be an implementation bug in the parser.

Fixing implementation...
```

### Unclear Requirements

**Scenario**: Expected behavior is unclear

**Action**:
- List what is unclear
- Ask user for clarification
- Do not guess or make assumptions

### Existing Tests

**Scenario**: Tests already exist for functionality

**Action**:
- Review existing tests
- If coverage sufficient: skip creating new tests
- If coverage insufficient: add tests for uncovered scenarios
- Update existing tests if needed

## Notes

- **Unit vs Property Tests**: Unit tests verify specific examples; property tests verify universal truths
- **Test Independence**: Each test should be independent
- **Minimal Mocking**: Use real implementations when possible
- **Descriptive Names**: Test names should explain what is tested
- **Edge Cases Matter**: Boundary values and None handling often reveal bugs
- **Error Testing**: Testing error conditions is as important as success cases
- **Keep It Simple**: Simple, focused tests are easier to maintain
- **Test Real Functionality**: Tests must validate real functionality, not mocked behavior
- **Rust Conventions**: Follow Rust testing conventions and idioms
- **Documentation Tests**: Consider adding doc tests for public APIs
