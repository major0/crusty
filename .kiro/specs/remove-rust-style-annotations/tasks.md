# Implementation Tasks: C-Style Variable Declarations

**Status:** In Progress  
**Related:** requirements.md, design.md

## Task Overview

Implement C-style variable declaration syntax where `Type name = value;` is the primary form and `let`/`var` are optional keywords.

---

## Phase 1: Documentation Fixes (IMMEDIATE)

### Task 1.1: Fix SYNTAX_REFERENCE.md Type Aliases Section
**Priority:** HIGH  
**Estimated Time:** 30 minutes  
**Status:** ✅ Completed

**Description:** Remove all casting in declaration examples from SYNTAX_REFERENCE.md

**Subtasks:**
- [x] 1.1.1: Update "Simple Type Aliases" section - replace `let x = (MyInt)42;` with `let x = 42;`
- [x] 1.1.2: Update "Pointer and Reference Type Aliases" section
- [x] 1.1.3: Update "Custom Type Aliases" section
- [x] 1.1.4: Update "Chained Type Aliases" section
- [x] 1.1.5: Update "Generic Type Aliases" section
- [x] 1.1.6: Add note that C-style declarations are planned but not yet implemented
- [x] 1.1.7: Verify all examples use only type inference (`let x = 42;`)

**Acceptance Criteria:**
- ✅ No examples show casting in declarations
- ✅ All examples use type inference
- ✅ Note added about future C-style support

---

## Phase 2: Parser Implementation (CORE FEATURE)

### Task 2.1: Update parse_let_statement() to Accept Optional Type
**Priority:** HIGH  
**Estimated Time:** 2 hours  
**Status:** ✅ Completed

**Description:** Modify `parse_let_statement()` to accept optional type before variable name

**Implementation:**
```rust
fn parse_let_statement() {
    expect(Let);
    
    // Check if next token is a type
    if is_type_token() {
        ty = parse_type();
        name = parse_identifier();
    } else {
        // Type inference
        name = parse_identifier();
        ty = None;
    }
    
    expect(Assign);
    init = parse_expression();
}
```

**Subtasks:**
- [x] 2.1.1: Add type detection logic after `Let` keyword
- [x] 2.1.2: Parse type if present using `parse_type()`
- [x] 2.1.3: Parse variable name after type
- [x] 2.1.4: Maintain backward compatibility with inference
- [x] 2.1.5: Add unit tests for `let int x = 42;`
- [x] 2.1.6: Add unit tests for `let MyInt x = 32;`
- [x] 2.1.7: Verify existing inference tests still pass

**Acceptance Criteria:**
- ✅ Parser accepts `let int x = 42;`
- ✅ Parser accepts `let MyInt x = 32;`
- ✅ Parser accepts `let x = 42;` (existing)
- ✅ All tests pass

---

### Task 2.2: Update parse_var_statement() to Accept Optional Type
**Priority:** HIGH  
**Estimated Time:** 1.5 hours  
**Status:** ✅ Completed

**Description:** Modify `parse_var_statement()` to accept optional type before variable name

**Implementation:**
```rust
fn parse_var_statement() {
    expect(Var);
    
    // Check if next token is a type
    if is_type_token() {
        ty = parse_type();
        name = parse_identifier();
    } else {
        // Type inference
        name = parse_identifier();
        ty = None;
    }
    
    expect(Assign);
    init = parse_expression();
}
```

**Subtasks:**
- [x] 2.2.1: Add type detection logic after `Var` keyword
- [x] 2.2.2: Parse type if present using `parse_type()`
- [x] 2.2.3: Parse variable name after type
- [x] 2.2.4: Maintain backward compatibility with inference
- [x] 2.2.5: Add unit tests for `var int x = 42;`
- [x] 2.2.6: Add unit tests for `var MyInt x = 32;`
- [x] 2.2.7: Verify existing inference tests still pass

**Acceptance Criteria:**
- ✅ Parser accepts `var int x = 42;`
- ✅ Parser accepts `var MyInt x = 32;`
- ✅ Parser accepts `var x = 42;` (existing)
- ✅ All tests pass

---

### Task 2.3: Update parse_const_statement() to Accept Optional Type
**Priority:** HIGH  
**Estimated Time:** 1.5 hours  
**Status:** ✅ Completed

**Description:** Modify `parse_const_statement()` to accept optional type before constant name

**Subtasks:**
- [x] 2.3.1: Add type detection logic after `Const` keyword
- [x] 2.3.2: Parse type if present using `parse_type()`
- [x] 2.3.3: Parse constant name after type
- [x] 2.3.4: Maintain backward compatibility with inference
- [x] 2.3.5: Add unit tests for `const int MAX = 100;`
- [x] 2.3.6: Add unit tests for `const MyInt MAX = 100;`
- [x] 2.3.7: Verify existing inference tests still pass

**Acceptance Criteria:**
- ✅ Parser accepts `const int MAX = 100;`
- ✅ Parser accepts `const MyInt MAX = 100;`
- ✅ Parser accepts `const MAX = 100;` (existing)
- ✅ All tests pass

---

### Task 2.4: Add parse_implicit_let_statement() Function
**Priority:** HIGH  
**Estimated Time:** 2 hours  
**Status:** ✅ Completed

**Description:** Add new function to parse `Type name = value;` (implicit let)

**Implementation:**
```rust
fn parse_implicit_let_statement() {
    // Parse type
    ty = parse_type();
    
    // Parse name
    name = parse_identifier();
    
    // Expect assignment
    expect(Assign);
    
    // Parse initializer
    init = parse_expression();
    
    expect(Semicolon);
    
    // Create Let statement with type
    return Statement::Let {
        name,
        ty: Some(ty),
        init: Some(init),
        mutable: false,
    };
}
```

**Subtasks:**
- [x] 2.4.1: Create `parse_implicit_let_statement()` function
- [x] 2.4.2: Parse type using `parse_type()`
- [x] 2.4.3: Parse variable name
- [x] 2.4.4: Parse assignment and initializer
- [x] 2.4.5: Return `Statement::Let` with `mutable: false`
- [x] 2.4.6: Add unit tests for `int x = 42;`
- [x] 2.4.7: Add unit tests for `MyInt x = 32;`
- [x] 2.4.8: Add unit tests for `int* ptr = NULL;`

**Acceptance Criteria:**
- ✅ Parser accepts `int x = 42;`
- ✅ Parser accepts `MyInt x = 32;`
- ✅ Parser treats as immutable (implicit let)
- ✅ All tests pass

---

### Task 2.5: Add looks_like_declaration() Helper Function
**Priority:** HIGH  
**Estimated Time:** 1.5 hours  
**Status:** ✅ Completed

**Description:** Add lookahead helper to distinguish declarations from expressions

**Implementation:**
```rust
fn looks_like_declaration() -> bool {
    // Look ahead: Type Identifier '='
    // If we see this pattern, it's a declaration
    // Otherwise, it's an expression
    
    if !is_type_token() {
        return false;
    }
    
    // Peek ahead past the type
    let next = peek_ahead(1);
    if !matches!(next, Identifier) {
        return false;
    }
    
    // Peek ahead past the identifier
    let next_next = peek_ahead(2);
    matches!(next_next, Assign)
}
```

**Subtasks:**
- [x] 2.5.1: Create `looks_like_declaration()` function
- [x] 2.5.2: Implement lookahead for pattern `Type Identifier '='`
- [x] 2.5.3: Handle edge case: `int(x)` (cast, not declaration)
- [x] 2.5.4: Handle edge case: `int + 5` (expression, not declaration)
- [x] 2.5.5: Add unit tests for lookahead logic
- [x] 2.5.6: Test with typedef names
- [x] 2.5.7: Test with pointer/reference types

**Acceptance Criteria:**
- ✅ Correctly identifies `int x = 42;` as declaration
- ✅ Correctly identifies `int(x)` as cast expression
- ✅ Correctly identifies `int + 5` as expression
- ✅ All edge cases handled

---

### Task 2.6: Update parse_statement() to Route to Implicit Let
**Priority:** HIGH  
**Estimated Time:** 1 hour  
**Status:** ✅ Completed

**Description:** Update `parse_statement()` to detect and route implicit let declarations

**Implementation:**
```rust
fn parse_statement() {
    match current_token {
        Let => parse_let_statement(),
        Var => parse_var_statement(),
        
        // Check for implicit let (Type name = value)
        _ if is_type_token() => {
            // Look ahead to check if this is a declaration
            if looks_like_declaration() {
                parse_implicit_let_statement()
            } else {
                parse_expression_statement()
            }
        }
        
        // ... other statements
    }
}
```

**Subtasks:**
- [x] 2.6.1: Add type token check in `parse_statement()`
- [x] 2.6.2: Call `looks_like_declaration()` for lookahead
- [x] 2.6.3: Route to `parse_implicit_let_statement()` if declaration
- [x] 2.6.4: Route to `parse_expression_statement()` if expression
- [x] 2.6.5: Add integration tests for routing logic
- [x] 2.6.6: Verify no regression in existing statement parsing

**Acceptance Criteria:**
- ✅ `int x = 42;` routes to implicit let
- ✅ `int(x)` routes to expression
- ✅ All existing statement types still work
- ✅ All tests pass

---

## Phase 3: Code Generator Updates

### Task 3.1: Update Code Generator for C-Style Output
**Priority:** HIGH  
**Estimated Time:** 2 hours  
**Status:** Not Started

**Description:** Update code generator to emit C-style syntax for explicit types

**Implementation:**
```rust
fn generate_let_statement(stmt: &Statement::Let) {
    match target {
        Rust => {
            write("let ");
            if mutable { write("mut "); }
            write(name);
            if let Some(ty) = ty {
                write(": ");
                write(generate_type(ty));
            }
            write(" = ");
            write(generate_expr(init));
        }
        Crusty => {
            // If type is present, use C-style
            if let Some(ty) = ty {
                write(generate_type(ty));
                write(" ");
                write(name);
            } else {
                // Use let for inference
                write("let ");
                write(name);
            }
            write(" = ");
            write(generate_expr(init));
        }
    }
}
```

**Subtasks:**
- [ ] 3.1.1: Update `generate_let_statement()` for Crusty target
- [ ] 3.1.2: Emit `Type name = value;` when type is present
- [ ] 3.1.3: Emit `let name = value;` when type is None
- [ ] 3.1.4: Handle mutable case: `var Type name = value;`
- [ ] 3.1.5: Update `generate_var_statement()` similarly
- [ ] 3.1.6: Update `generate_const_statement()` similarly
- [ ] 3.1.7: Add unit tests for code generation
- [ ] 3.1.8: Verify roundtrip: parse → generate → parse

**Acceptance Criteria:**
- C-style input generates C-style output
- Inference input generates inference output
- No casting in generated declarations
- Roundtrip works correctly
- All tests pass

---

## Phase 4: Test Updates

### Task 4.1: Update Typedef Integration Tests
**Priority:** MEDIUM  
**Estimated Time:** 1 hour  
**Status:** Not Started

**Description:** Update typedef tests to use C-style declarations

**Subtasks:**
- [ ] 4.1.1: Update `test_typedef_simple_variable_declaration`
- [ ] 4.1.2: Update `test_typedef_with_cast_syntax`
- [ ] 4.1.3: Update `test_typedef_function_parameter`
- [ ] 4.1.4: Update `test_typedef_return_type`
- [ ] 4.1.5: Update all other typedef tests
- [ ] 4.1.6: Run typedef test suite
- [ ] 4.1.7: Verify all tests pass

**Acceptance Criteria:**
- All typedef tests use C-style declarations
- No casting in declarations
- All tests pass

---

### Task 4.2: Update Nested Function Tests
**Priority:** MEDIUM  
**Estimated Time:** 1 hour  
**Status:** Not Started

**Description:** Update nested function tests to use C-style declarations

**Subtasks:**
- [ ] 4.2.1: Update capture analysis tests
- [ ] 4.2.2: Update scoping tests
- [ ] 4.2.3: Update code generation tests
- [ ] 4.2.4: Update integration tests
- [ ] 4.2.5: Run nested function test suite
- [ ] 4.2.6: Verify all tests pass

**Acceptance Criteria:**
- All nested function tests use C-style declarations
- All tests pass

---

### Task 4.3: Update Example Files
**Priority:** MEDIUM  
**Estimated Time:** 1 hour  
**Status:** Not Started

**Description:** Update all `.crst` example files to use C-style as primary

**Subtasks:**
- [ ] 4.3.1: Update `example/src/main.crst`
- [ ] 4.3.2: Update `example/src/typedef_demo.crst`
- [ ] 4.3.3: Update `example/src/functions.crst`
- [ ] 4.3.4: Update `example/src/structs.crst`
- [ ] 4.3.5: Update all other `.crst` files
- [ ] 4.3.6: Build example project
- [ ] 4.3.7: Verify examples compile and run

**Acceptance Criteria:**
- All examples use C-style as primary
- Type inference used where appropriate
- Examples compile and run successfully

---

### Task 4.4: Update Test Files
**Priority:** MEDIUM  
**Estimated Time:** 30 minutes  
**Status:** Not Started

**Description:** Update test `.crst` files to use C-style declarations

**Subtasks:**
- [ ] 4.4.1: Update `test_nested_capture_simple.crst`
- [ ] 4.4.2: Update `test_nested_function_capture.crst`
- [ ] 4.4.3: Update `test_simple.crst`
- [ ] 4.4.4: Update any other test `.crst` files
- [ ] 4.4.5: Verify all test files parse correctly

**Acceptance Criteria:**
- All test files use C-style declarations
- All files parse successfully

---

## Phase 5: Documentation Updates

### Task 5.1: Update SYNTAX_REFERENCE.md Variable Declarations Section
**Priority:** MEDIUM  
**Estimated Time:** 30 minutes  
**Status:** Not Started

**Description:** Update variable declarations section to show C-style as primary

**Subtasks:**
- [ ] 5.1.1: Add "Variable Declarations" section if not present
- [ ] 5.1.2: Show C-style as primary: `int x = 42;`
- [ ] 5.1.3: Show type inference as alternative: `let x = 42;`
- [ ] 5.1.4: Document mutable: `var int x = 42;`
- [ ] 5.1.5: Document constants: `const int MAX = 100;`
- [ ] 5.1.6: Add examples with typedef types
- [ ] 5.1.7: Remove any remaining casting examples

**Acceptance Criteria:**
- C-style shown as primary syntax
- Type inference documented as alternative
- No casting in declarations
- Clear examples for all forms

---

### Task 5.2: Update README.md Examples
**Priority:** LOW  
**Estimated Time:** 15 minutes  
**Status:** Not Started

**Description:** Update README examples to show C-style where appropriate

**Subtasks:**
- [ ] 5.2.1: Review all code examples in README
- [ ] 5.2.2: Update to C-style where appropriate
- [ ] 5.2.3: Keep type inference where it makes sense
- [ ] 5.2.4: Verify consistency with SYNTAX_REFERENCE.md

**Acceptance Criteria:**
- Examples use C-style as primary
- Consistent with SYNTAX_REFERENCE.md
- No casting in declarations

---

## Phase 6: Final Validation

### Task 6.1: Run Full Test Suite
**Priority:** HIGH  
**Estimated Time:** 15 minutes  
**Status:** Not Started

**Description:** Run complete test suite and verify all tests pass

**Subtasks:**
- [ ] 6.1.1: Run `cargo test --lib`
- [ ] 6.1.2: Verify 100% pass rate (excluding ignored)
- [ ] 6.1.3: Check for any warnings
- [ ] 6.1.4: Run property-based tests
- [ ] 6.1.5: Run integration tests

**Acceptance Criteria:**
- All tests pass
- No warnings
- Property tests pass
- Integration tests pass

---

### Task 6.2: Verify Example Project Builds
**Priority:** HIGH  
**Estimated Time:** 10 minutes  
**Status:** Not Started

**Description:** Build and run example project to verify end-to-end functionality

**Subtasks:**
- [ ] 6.2.1: `cd example && cargo build`
- [ ] 6.2.2: `cargo run`
- [ ] 6.2.3: Verify output is correct
- [ ] 6.2.4: Check for any warnings

**Acceptance Criteria:**
- Example builds successfully
- Example runs without errors
- Output is correct

---

### Task 6.3: Update Coverage Reports
**Priority:** LOW  
**Estimated Time:** 15 minutes  
**Status:** Not Started

**Description:** Update coverage reports with new test counts and status

**Subtasks:**
- [ ] 6.3.1: Update COVERAGE_REVIEW_SUMMARY.md
- [ ] 6.3.2: Update TEST_COVERAGE_REPORT.md
- [ ] 6.3.3: Update test counts
- [ ] 6.3.4: Update pass rates
- [ ] 6.3.5: Document new features tested

**Acceptance Criteria:**
- Coverage reports reflect current state
- Test counts accurate
- Pass rates accurate

---

### Task 6.4: Final Documentation Review
**Priority:** MEDIUM  
**Estimated Time:** 30 minutes  
**Status:** Not Started

**Description:** Review all documentation for consistency and accuracy

**Subtasks:**
- [ ] 6.4.1: Review requirements.md
- [ ] 6.4.2: Review design.md
- [ ] 6.4.3: Review SYNTAX_REFERENCE.md
- [ ] 6.4.4: Review README.md
- [ ] 6.4.5: Check for any inconsistencies
- [ ] 6.4.6: Verify all examples are correct

**Acceptance Criteria:**
- All documentation consistent
- No incorrect examples
- C-style shown as primary
- Type inference documented as alternative

---

## Summary

**Total Tasks:** 24  
**Completed:** 7 (Phase 1 + Phase 2)  
**In Progress:** 0  
**Not Started:** 17

**Estimated Total Time:** 20-25 hours  
**Time Spent:** 10.5 hours  
**Time Remaining:** 9.5-14.5 hours

**Critical Path:**
1. ✅ Phase 1: Documentation Fixes (30 min) - COMPLETED
2. ✅ Phase 2: Parser Implementation (10 hours) - COMPLETED
3. Phase 3: Code Generator Updates (2 hours) - READY TO START
4. Phase 4: Test Updates (3.5 hours)
5. Phase 5: Documentation Updates (45 min)
6. Phase 6: Final Validation (1.5 hours)

**Dependencies:**
- ✅ Phase 2 completed
- Phase 3 must complete before Phase 4
- Phase 4 must complete before Phase 6
- ✅ Phase 1 completed independently

**Next Steps:**
1. ✅ Task 1.1 (Fix SYNTAX_REFERENCE.md) - COMPLETED
2. ✅ Task 2.1-2.6 (Parser Implementation) - COMPLETED
3. **START HERE:** Task 3.1 (Update Code Generator)
4. Continue with Phases 4-6 in order

---

**Created:** January 31, 2026  
**Last Updated:** January 31, 2026  
**Status:** Phase 2 Complete - All parser tests passing (428/428)

