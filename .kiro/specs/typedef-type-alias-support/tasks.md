# Tasks: Typedef Type Alias Support

## 1. Core Type Resolution Implementation
- [ ] 1.1 Add `resolve_type()` function to `TypeEnvironment` in `src/semantic.rs`
  - Implement recursive type alias resolution
  - Handle all type variants (Ident, Pointer, Reference, Array, Slice, Generic, Tuple, Function, Fallible)
  - Return resolved type or original if not an alias
  
- [ ] 1.2 Update `is_compatible()` function to resolve types before comparison
  - Call `resolve_type()` on both input types
  - Perform compatibility check on resolved types
  - Maintain existing compatibility logic

- [ ] 1.3 Write unit tests for `resolve_type()`
  - Test simple alias resolution (typedef int MyInt)
  - Test chained alias resolution (typedef int A; typedef A B)
  - Test complex type resolution (pointers, references, generics)
  - Test non-alias types return unchanged

## 2. Circular Reference Detection
- [ ] 2.1 Add `has_circular_reference()` function to `TypeEnvironment`
  - Implement visited set tracking
  - Check for cycles in alias chains
  - Handle complex types recursively

- [ ] 2.2 Update `analyze_typedef()` to check for circular references
  - Call `has_circular_reference()` before registering typedef
  - Report semantic error if circular reference detected
  - Prevent registration of circular typedefs

- [ ] 2.3 Write unit tests for circular reference detection
  - Test direct circular reference (typedef A A)
  - Test indirect circular reference (typedef A B; typedef B A)
  - Test multi-step circular reference (typedef A B; typedef B C; typedef C A)
  - Test non-circular chains pass validation

## 3. Integration Testing
- [ ] 3.1 Create comprehensive typedef test file
  - Test simple type aliases in variable declarations
  - Test typedef in function parameters
  - Test typedef in function return types
  - Test typedef with struct types
  - Test typedef with pointer types
  - Test typedef with reference types
  - Test typedef with generic types

- [ ] 3.2 Add parser tests for typedef (if missing)
  - Test parsing simple typedef
  - Test parsing typedef with complex types
  - Test parsing multiple typedefs

- [ ] 3.3 Add codegen tests for typedef (if missing)
  - Test code generation for simple typedef
  - Test code generation for complex typedef
  - Verify generated Rust syntax is correct

- [ ] 3.4 Verify all existing tests still pass
  - Run full test suite
  - Fix any regressions
  - Ensure backward compatibility

## 4. Property-Based Testing
- [ ] 4.1 Write property test for type alias transitivity
  - Generate random type alias chains
  - Verify resolve_type is transitive
  - **Validates: Requirements 1.3**

- [ ] 4.2 Write property test for compatibility symmetry
  - Generate random type pairs
  - Verify compatibility is symmetric with resolution
  - **Validates: Requirements 1.3, 2.3, 3.3, 4.3, 5.3**

- [ ] 4.3 Write property test for resolution idempotence
  - Generate random types
  - Verify resolve_type(resolve_type(T)) == resolve_type(T)
  - **Validates: Requirements 1.3, 2.3, 3.3, 4.3, 5.3**

- [ ] 4.4 Write property test for circular reference detection
  - Generate random circular alias chains
  - Verify all cycles are detected
  - **Validates: Non-functional requirement for error handling**

## 5. End-to-End Validation
- [ ] 5.1 Test typedef with real Crusty programs
  - Create example programs using typedef
  - Compile to Rust
  - Verify Rust code compiles
  - Run generated programs

- [ ] 5.2 Test error messages
  - Verify undefined type errors are clear
  - Verify circular reference errors are helpful
  - Test error messages with complex typedef chains

- [ ] 5.3 Performance testing
  - Measure compilation time with many typedefs
  - Verify no significant performance regression
  - Profile type resolution if needed

## 6. Documentation
- [ ] 6.1 Update SYNTAX_REFERENCE.md with typedef examples
  - Document typedef syntax
  - Show examples of simple and complex typedefs
  - Explain type alias behavior

- [ ] 6.2 Add typedef examples to example/ directory
  - Create example showing typedef usage
  - Demonstrate best practices
  - Show common patterns

- [ ] 6.3 Update README.md if needed
  - Mention typedef support in features list
  - Link to syntax reference
  - Add to feature matrix

## Task Dependencies
- Task 1.2 depends on 1.1 (need resolve_type before updating is_compatible)
- Task 2.2 depends on 2.1 (need has_circular_reference before updating analyze_typedef)
- Task 3 depends on 1 and 2 (need core implementation before integration testing)
- Task 4 depends on 1 and 2 (need core implementation before property testing)
- Task 5 depends on 1, 2, 3, 4 (need all implementation and testing complete)
- Task 6 can be done in parallel with testing

## Estimated Effort
- Task 1: 2-3 hours (core implementation)
- Task 2: 1-2 hours (circular reference detection)
- Task 3: 2-3 hours (integration testing)
- Task 4: 2-3 hours (property-based testing)
- Task 5: 1-2 hours (end-to-end validation)
- Task 6: 1 hour (documentation)

**Total: 9-14 hours**
