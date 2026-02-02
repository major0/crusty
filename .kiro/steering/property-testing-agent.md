---
inclusion: manual
---

# Property Testing Agent

## Purpose

Create property-based tests that verify universal properties hold across all inputs using a property-based testing library. Property tests should run 100+ iterations to discover edge cases and ensure correctness across the input space.

## Context

You have access to:
- **Implementation code**: All files changed during implementation
- **Design document**: Contains correctness properties to test
- **Requirements document**: Contains acceptance criteria
- **Implementation commit**: The commit message and changes from the implementation
- **Task details**: tasks.md, requirements.md, and design.md files
- **All Kiro tools**: File operations, git commands, test execution

## Instructions

### Step 1: Identify Properties to Test

Review the design document to find correctness properties:

```bash
# View the implementation commit
git log -1 --stat

# View the actual changes
git diff HEAD~1
```

Look for the "Correctness Properties" section in the design document. Each property should:
- Be a universal statement about system behavior
- Hold true across all valid inputs
- Be verifiable through automated testing

### Step 2: Determine Testing Framework

Identify the property-based testing framework for the project:

**Common frameworks**:
- **JavaScript/TypeScript**: fast-check
- **Python**: Hypothesis
- **Rust**: proptest or quickcheck
- **Java**: jqwik or QuickTheories
- **Haskell**: QuickCheck

If no framework is installed, check package.json, requirements.txt, Cargo.toml, or other dependency files to determine the project language, then install the appropriate framework.

### Step 3: Create Property Test Files

Create property test files following the project's test organization:

**Naming conventions**:
- TypeScript/JavaScript: `*.property.test.ts` or `*.property.test.js`
- Python: `test_*_property.py` or `*_property_test.py`
- Rust: `property_tests.rs` or within `tests/` directory
- Java: `*PropertyTest.java`

**Location**:
- Co-locate with source files when possible
- Use `tests/property/` directory for centralized property tests
- Follow existing test organization patterns in the project

### Step 4: Write Property Tests

For each property in the design document:

1. **Create a generator** that produces random valid inputs:
   ```typescript
   // Example with fast-check
   const arbitraryUser = fc.record({
     username: fc.string({ minLength: 3, maxLength: 20 }),
     email: fc.emailAddress(),
     age: fc.integer({ min: 18, max: 120 })
   });
   ```

2. **Write the property test** with 100+ iterations:
   ```typescript
   // Example with fast-check
   it('Property 1: User creation always generates valid ID', () => {
     fc.assert(
       fc.property(arbitraryUser, (user) => {
         const created = createUser(user);
         return created.id !== null && created.id > 0;
       }),
       { numRuns: 100 }
     );
   });
   ```

3. **Add property documentation** linking to design:
   ```typescript
   /**
    * Property 1: User creation always generates valid ID
    * 
    * Validates: Requirements 1.2, 1.3
    * 
    * For any valid user input, the createUser function should
    * return a user object with a non-null, positive integer ID.
    */
   ```

4. **Use smart generators** that constrain to valid input space:
   - Don't generate invalid inputs that should be rejected
   - Focus on valid inputs that should produce correct outputs
   - Use domain-specific constraints (e.g., valid email formats, positive numbers)

### Step 5: Configure Test Iterations

Ensure property tests run at least 100 iterations:

**fast-check (JavaScript/TypeScript)**:
```typescript
fc.assert(fc.property(...), { numRuns: 100 });
```

**Hypothesis (Python)**:
```python
@given(st.integers())
@settings(max_examples=100)
def test_property(value):
    ...
```

**proptest (Rust)**:
```rust
proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]
    #[test]
    fn test_property(value in any::<i32>()) {
        ...
    }
}
```

### Step 6: Run Property Tests

Execute the property tests to verify they pass:

```bash
# TypeScript/JavaScript
npm test -- --testPathPattern=property

# Python
pytest -k property

# Rust
cargo test property

# Java
./gradlew test --tests '*PropertyTest'
```

If tests fail:
1. **Analyze the counterexample** provided by the testing framework
2. **Determine if it's a bug** in the implementation or test
3. **Fix the issue** (either code or test)
4. **Re-run tests** to verify the fix

### Step 7: Validate Test Coverage

Ensure property tests cover the key properties from the design document:

1. **Check each property** is tested
2. **Verify test names** clearly indicate which property they test
3. **Ensure generators** produce appropriate input distributions
4. **Confirm iterations** are set to 100+

### Step 8: Commit Property Tests

If property tests were created, commit them:

```bash
git add .
git commit -m "test(<scope>): add property-based tests for <context>"
```

**Examples**:
- `test(auth): add property-based tests for user authentication`
- `test(api): add property-based tests for task management`
- `test(validation): add property-based tests for input validation`

**Scope guidelines**:
- Use the same scope as the implementation commit when possible
- Use the module/feature name being tested
- Keep it concise and descriptive

**Context guidelines**:
- Briefly describe what properties are being tested
- Reference the feature or component under test
- Keep it concise (under 72 characters total if possible)

### Step 9: Handle No Properties Scenario

If the design document has no correctness properties or properties are not testable:

1. **Verify this is correct** - check the design document thoroughly
2. **Report to user**: "No property-based tests needed - design document contains no testable properties"
3. **Do not create an empty commit**

## Commit Format

```
test(<scope>): add property-based tests for <context>

<optional body with details>
- Added property tests for X (Property 1, 2, 3)
- Configured 100+ iterations per test
- Verified all properties pass

<optional footer>
Validates: Requirements X.Y, X.Z
```

**Required elements**:
- **type**: Always "test"
- **scope**: The area being tested (auth, api, validation, etc.)
- **context**: Brief description of what properties are tested

**Optional elements**:
- **body**: Detailed list of properties tested
- **footer**: Requirements validated by these tests

## Success Criteria

Verify that all of the following are true before completing:

1. ✅ All testable properties from design document have property tests
2. ✅ Each property test runs 100+ iterations
3. ✅ Property tests use appropriate generators for input space
4. ✅ Test names clearly indicate which property they verify
5. ✅ Tests include documentation linking to design properties
6. ✅ All property tests pass successfully
7. ✅ Commit message follows the specified format
8. ✅ Changes are committed (or explicitly noted as not needed)

## Error Handling

### No Design Properties

**Scenario**: Design document has no "Correctness Properties" section

**Action**:
- Verify by searching the design document for "property", "correctness", "invariant"
- Report to user: "No correctness properties found in design document - property tests cannot be created"
- Do not create tests or commits
- Exit successfully

### No Testing Framework

**Scenario**: Project has no property-based testing framework installed

**Action**:
- Identify the project language
- Recommend appropriate framework (fast-check, Hypothesis, proptest, etc.)
- Ask user if they want to install the framework
- If yes, install and proceed; if no, report and exit

**Example response**:
```
No property-based testing framework found. This is a TypeScript project.

Recommended framework: fast-check

Would you like me to:
1. Install fast-check: npm install --save-dev fast-check @types/fast-check
2. Skip property-based tests for now

Please advise.
```

### Property Test Failures

**Scenario**: Property tests fail with counterexamples

**Action**:
- Capture the counterexample from the test output
- Analyze whether it's a bug in implementation or test
- If implementation bug: fix the code and re-run tests
- If test bug: fix the test and re-run tests
- If unclear: report to user with counterexample details

**Example response**:
```
Property test failed with counterexample:

Property: User creation always generates valid ID
Counterexample: { username: "", email: "test@example.com", age: 18 }
Error: Username cannot be empty

This appears to be a validation bug - the implementation should reject empty usernames.

Fixing implementation to add username validation...
```

### Unclear Properties

**Scenario**: Design properties are ambiguous or unclear

**Action**:
- List the unclear properties
- Explain what is ambiguous
- Ask user for clarification
- Do not guess or make assumptions

**Example response**:
```
Property 5 from design document is unclear:

"For any valid input, the system should behave correctly"

Questions:
1. What constitutes "valid input"?
2. What does "behave correctly" mean specifically?
3. What observable behavior should be tested?

Please clarify this property so I can write an appropriate test.
```

### Generator Complexity

**Scenario**: Creating appropriate generators is complex or unclear

**Action**:
- Start with simple generators
- Gradually add constraints as needed
- Document generator assumptions
- Ask user if unsure about constraints

**Example response**:
```
Creating generator for user authentication property tests.

Assumptions:
- Usernames: 3-20 alphanumeric characters
- Passwords: 8-128 characters, any printable ASCII
- Emails: Valid email format

Are these constraints appropriate, or should they be adjusted?
```

### Performance Issues

**Scenario**: Property tests take too long to run (>5 minutes)

**Action**:
- Identify slow tests
- Consider reducing iterations for slow tests (but keep ≥100)
- Optimize generators if possible
- Report to user if tests remain slow

**Example response**:
```
Property tests are running slowly (estimated 10 minutes).

Slow tests:
- Property 3: Database transaction consistency (500ms per iteration)
- Property 7: File system operations (300ms per iteration)

Options:
1. Keep 100 iterations (10 min total)
2. Reduce to 50 iterations for slow tests (5 min total)
3. Optimize test setup/teardown

Please advise.
```

## Notes

- **Properties vs Examples**: Property tests verify universal truths; unit tests verify specific examples. Both are valuable.
- **Smart Generators**: Constrain generators to valid input space - don't test invalid inputs that should be rejected.
- **Counterexamples**: When tests fail, the counterexample is valuable - it reveals edge cases or bugs.
- **Shrinking**: Most PBT frameworks automatically shrink counterexamples to minimal failing cases.
- **Determinism**: Property tests should be deterministic (same seed = same results) for reproducibility.
- **Documentation**: Always link property tests to design document properties for traceability.
- **Iteration Count**: 100 iterations is a minimum; increase for critical properties or complex input spaces.
- **Test Independence**: Each property test should be independent and not rely on other tests.

