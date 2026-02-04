---
inclusion: manual
---

# Property Testing Agent

## Purpose

Create property-based tests that verify universal properties hold across all inputs using proptest, Rust's property-based testing library. Property tests should run 100+ iterations to discover edge cases and ensure correctness across the input space.

## Context

You have access to:
- **Implementation code**: All Rust files changed during implementation
- **Design document**: Contains correctness properties to test
- **Requirements document**: Contains acceptance criteria
- **Implementation commit**: The commit message and changes from the implementation
- **Task details**: tasks.md, requirements.md, and design.md files
- **All Kiro tools**: File operations, git commands, cargo test

## Instructions

### Step 1: Identify Properties to Test

Review the design document to find correctness properties:

```bash
# View implementation commit
git log -1 --stat

# View actual changes
git diff HEAD~1
```

Look for the "Correctness Properties" section in the design document. Each property should:
- Be a universal statement about system behavior
- Hold true across all valid inputs
- Be verifiable through automated testing

### Step 2: Proptest Framework

This project uses **proptest** for property-based testing:

**Add to Cargo.toml** (if not already present):
```toml
[dev-dependencies]
proptest = "1.4"
```

**Import proptest**:
```rust
use proptest::prelude::*;
```

### Step 3: Create Property Test Files

Add property tests to source files or create separate test files:

**Inline property tests**:
```rust
// In src/parser.rs

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_parse_roundtrip(input in ".*") {
            // Property test implementation
        }
    }
}
```

**Separate property test files**:
```rust
// In src/parser_properties.rs

use proptest::prelude::*;
use crate::parser::*;

proptest! {
    #[test]
    fn test_parser_properties(input in ".*") {
        // Property test implementation
    }
}
```

### Step 4: Write Property Tests

For each property in the design document:

1. **Create a strategy** that generates random valid inputs:
   ```rust
   use proptest::prelude::*;
   
   // Strategy for valid identifiers
   fn identifier_strategy() -> impl Strategy<Value = String> {
       "[a-zA-Z_][a-zA-Z0-9_]*"
   }
   
   // Strategy for valid function declarations
   fn function_strategy() -> impl Strategy<Value = String> {
       (identifier_strategy(), identifier_strategy())
           .prop_map(|(ret_type, name)| {
               format!("{} {}() {{}}", ret_type, name)
           })
   }
   ```

2. **Write the property test** with 100+ iterations:
   ```rust
   proptest! {
       #![proptest_config(ProptestConfig::with_cases(100))]
       
       #[test]
       fn property_parse_roundtrip(input in function_strategy()) {
           // Parse the input
           let ast = parse_function(&input).unwrap();
           
           // Generate code from AST
           let output = generate_code(&ast);
           
           // Parse the output
           let ast2 = parse_function(&output).unwrap();
           
           // Property: Parsing is idempotent
           prop_assert_eq!(ast, ast2);
       }
   }
   ```

3. **Add property documentation** linking to design:
   ```rust
   /// Property 1: Parse-Generate Roundtrip
   /// 
   /// Validates: Requirements 1.2, 1.3
   /// 
   /// For any valid Crusty input, parsing and then generating code
   /// should produce an AST that parses to the same AST.
   proptest! {
       #[test]
       fn property_parse_roundtrip(input in function_strategy()) {
           // Test implementation
       }
   }
   ```

4. **Use smart strategies** that constrain to valid input space:
   ```rust
   // Don't generate invalid inputs
   fn valid_type_strategy() -> impl Strategy<Value = String> {
       prop_oneof![
           Just("int".to_string()),
           Just("void".to_string()),
           Just("char".to_string()),
           Just("float".to_string()),
       ]
   }
   
   // Generate valid function names
   fn function_name_strategy() -> impl Strategy<Value = String> {
       "[a-zA-Z_][a-zA-Z0-9_]{0,30}"
   }
   ```

### Step 5: Configure Test Iterations

Ensure property tests run at least 100 iterations:

```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    
    #[test]
    fn property_test(input in any::<String>()) {
        // Test implementation
    }
}
```

**Configuration options**:
```rust
ProptestConfig {
    cases: 100,                    // Number of test cases
    max_shrink_iters: 1000,       // Shrinking iterations
    max_shrink_time: 10000,       // Shrinking timeout (ms)
    ..ProptestConfig::default()
}
```

### Step 6: Run Property Tests

Execute property tests to verify they pass:

```bash
# Run all tests (including property tests)
cargo test

# Run only property tests
cargo test property

# Run with verbose output
cargo test property -- --nocapture

# Run with specific seed for reproducibility
PROPTEST_SEED=12345 cargo test property
```

If tests fail:
1. **Analyze the counterexample** provided by proptest
2. **Determine if it's a bug** in implementation or test
3. **Fix the issue** (code or test)
4. **Re-run tests** to verify fix

**Proptest shrinking**:
- Proptest automatically shrinks failing inputs to minimal examples
- Shrunk examples are easier to debug
- Shrinking results are saved in `proptest-regressions/`

### Step 7: Validate Test Coverage

Ensure property tests cover key properties from design:

1. Check each property is tested
2. Verify test names clearly indicate which property
3. Ensure strategies produce appropriate input distributions
4. Confirm iterations are set to 100+

### Step 8: Commit Property Tests

If property tests were created:

```bash
git add .
git commit -m "test(<scope>): add property-based tests for <context>"
```

**Examples**:
- `test(parser): add property-based tests for parsing`
- `test(codegen): add property-based tests for code generation`
- `test(semantic): add property-based tests for type checking`

**Scope guidelines**:
- Use the same scope as implementation commit
- Use module name (parser, codegen, semantic, etc.)

## Commit Format

```
test(<scope>): add property-based tests for <context>

<optional body>
- Added property tests for X (Property 1, 2, 3)
- Configured 100+ iterations per test
- Verified all properties pass

<optional footer>
Validates: Requirements X.Y, X.Z
```

## Success Criteria

1. ✅ All testable properties from design have property tests
2. ✅ Each property test runs 100+ iterations
3. ✅ Property tests use appropriate strategies for input space
4. ✅ Test names clearly indicate which property they verify
5. ✅ Tests include documentation linking to design properties
6. ✅ All property tests pass successfully
7. ✅ Commit message follows format
8. ✅ Changes committed (or noted as not needed)

## Error Handling

### No Design Properties

**Scenario**: Design document has no "Correctness Properties" section

**Action**:
- Search design document for "property", "correctness", "invariant"
- Report: "No correctness properties found - property tests cannot be created"
- Do not create tests or commits
- Exit successfully

### Property Test Failures

**Scenario**: Property tests fail with counterexamples

**Action**:
- Capture counterexample from proptest output
- Analyze if bug is in implementation or test
- Fix implementation or test as needed
- Re-run tests

**Example response**:
```
Property test failed with counterexample:

Property: Parse-Generate Roundtrip
Counterexample: "int main() { return 0; }"
Error: Generated code missing return statement

This appears to be a code generation bug.

Fixing codegen to include return statements...
```

### Unclear Properties

**Scenario**: Design properties are ambiguous

**Action**:
- List unclear properties
- Explain what is ambiguous
- Ask user for clarification
- Do not guess or make assumptions

### Generator Complexity

**Scenario**: Creating appropriate strategies is complex

**Action**:
- Start with simple strategies
- Gradually add constraints
- Document strategy assumptions
- Ask user if unsure about constraints

### Performance Issues

**Scenario**: Property tests take too long (>5 minutes)

**Action**:
- Identify slow tests
- Consider reducing iterations for slow tests (but keep ≥100)
- Optimize strategies if possible
- Report to user if tests remain slow

## Notes

- **Properties vs Examples**: Property tests verify universal truths; unit tests verify specific examples
- **Smart Strategies**: Constrain strategies to valid input space
- **Counterexamples**: When tests fail, counterexamples reveal edge cases or bugs
- **Shrinking**: Proptest automatically shrinks counterexamples to minimal failing cases
- **Determinism**: Property tests are deterministic with same seed
- **Documentation**: Always link property tests to design document properties
- **Iteration Count**: 100 iterations minimum; increase for critical properties
- **Test Independence**: Each property test should be independent
- **Regression Files**: Proptest saves failing cases in `proptest-regressions/`
- **Rust Idioms**: Follow Rust testing conventions and proptest best practices
