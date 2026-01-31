# Implementation Tasks: Crusty Compiler Phase 3

## Overview

Phase 3 implements **bidirectional transpilation** between Crusty and Rust. This enables converting Rust code to Crusty syntax and validates that Crusty is a complete, stable syntactic layer over Rust through round-trip transpilation.

**Core Principle**: Prove Crusty syntax completeness by demonstrating that Crusty ↔ Rust transpilation preserves semantic meaning.

**Important**: Each task should be committed using Conventional Commits format:
- Format: `type(scope): subject`
- Types: feat, fix, docs, test, refactor, chore
- Scope: task number (e.g., `task-1.1`)
- Body: Include "Validates: Requirements X" to reference requirements
- Example: `feat(task-1.1): integrate syn crate for Rust parsing`

## Tasks

### Part 1: Rust Parser Integration

- [ ] 1. Set up Rust parsing infrastructure
  - [ ] 1.1 Add syn crate dependency
    - Add syn = "2.0" to Cargo.toml
    - Add quote = "1.0" for code generation helpers
    - Update dependencies
    - Commit with message: "feat(task-1.1): add syn crate dependency"
    - _Requirements: 1.2_
  
  - [ ] 1.2 Create rust_parser module
    - Create `src/rust_parser.rs` module
    - Create `RustParser` struct
    - Add module to lib.rs
    - Commit with message: "feat(task-1.2): create rust_parser module"
    - _Requirements: 1_
  
  - [ ] 1.3 Implement basic Rust file parsing
    - Implement `parse_file()` using syn::parse_file()
    - Handle parsing errors
    - Return syn::File on success
    - Commit with message: "feat(task-1.3): implement basic Rust parsing"
    - _Requirements: 1.1, 1.2, 1.5_
  
  - [ ] 1.4 Write unit tests for Rust parsing
    - Test parsing simple Rust programs
    - Test parsing error handling
    - Test with various Rust syntax features
    - Commit with message: "test(task-1.4): add Rust parsing tests"
    - _Requirements: 1_

- [ ] 2. Implement syn AST to unified AST conversion
  - [ ] 2.1 Implement File conversion
    - Implement `convert_syn_file()` function
    - Convert syn::File to ast::File
    - Convert all items in file
    - Commit with message: "feat(task-2.1): implement File conversion"
    - _Requirements: 2.1_
  
  - [ ] 2.2 Implement Item conversion
    - Implement `convert_syn_item()` function
    - Convert syn::ItemFn to ast::Function
    - Convert syn::ItemStruct to ast::Struct
    - Convert syn::ItemEnum to ast::Enum
    - Convert syn::ItemType to ast::Typedef
    - Convert syn::ItemMod to ast::Namespace
    - Convert syn::ItemUse to ast::Use
    - Commit with message: "feat(task-2.2): implement Item conversion"
    - _Requirements: 2.2_

  
  - [ ] 2.3 Implement Expression conversion
    - Implement `convert_syn_expr()` function
    - Convert syn::ExprLit to ast::Expression::Literal
    - Convert syn::ExprPath to ast::Expression::Ident
    - Convert syn::ExprBinary to ast::Expression::Binary
    - Convert syn::ExprUnary to ast::Expression::Unary
    - Convert syn::ExprCall to ast::Expression::Call
    - Convert syn::ExprMethodCall to ast::Expression::MethodCall
    - Convert syn::ExprField to ast::Expression::FieldAccess
    - Convert syn::ExprIndex to ast::Expression::Index
    - Convert syn::ExprMatch to ast::Expression::Switch (special handling)
    - Commit with message: "feat(task-2.3): implement Expression conversion"
    - _Requirements: 2.3_
  
  - [ ] 2.4 Implement Type conversion
    - Implement `convert_syn_type()` function
    - Convert syn::TypePath to ast::Type::Ident or ast::Type::Primitive
    - Convert syn::TypeReference to ast::Type::Reference
    - Convert syn::TypePtr to ast::Type::Pointer
    - Convert syn::TypeArray to ast::Type::Array
    - Convert syn::TypeSlice to ast::Type::Slice
    - Convert syn::TypeTuple to ast::Type::Tuple
    - Convert syn::TypeParen to inner type
    - Commit with message: "feat(task-2.4): implement Type conversion"
    - _Requirements: 2.4_
  
  - [ ] 2.5 Implement Statement conversion
    - Implement `convert_syn_stmt()` function
    - Convert syn::StmtLocal to ast::Statement::Let or ast::Statement::Var
    - Convert syn::StmtExpr to ast::Statement::Expr
    - Convert syn::StmtSemi to ast::Statement::Expr with semicolon
    - Commit with message: "feat(task-2.5): implement Statement conversion"
    - _Requirements: 2.5_
  
  - [ ] 2.6 Handle Rust impl blocks
    - Add ast::Impl to AST if not present
    - Convert syn::ItemImpl to ast::Impl
    - Handle inherent impl blocks (no trait)
    - Handle trait impl blocks
    - Convert impl methods to ast::Function
    - Commit with message: "feat(task-2.6): handle Rust impl blocks"
    - _Requirements: 2.2, 3.4_
  
  - [ ] 2.7 Handle Rust traits
    - Add ast::Trait to AST if not present
    - Convert syn::ItemTrait to ast::Trait
    - Convert trait methods to ast::Function
    - Preserve trait bounds
    - Commit with message: "feat(task-2.7): handle Rust traits"
    - _Requirements: 2.2, 7.1, 7.2_
  
  - [ ] 2.8 Write unit tests for AST conversion
    - Test File conversion
    - Test Item conversion for all item types
    - Test Expression conversion for all expression types
    - Test Type conversion for all type variants
    - Test Statement conversion
    - Test impl block conversion
    - Test trait conversion
    - Commit with message: "test(task-2.8): add AST conversion tests"
    - _Requirements: 2_

### Part 2: Crusty Code Generation from Rust

- [ ] 3. Extend code generator for Crusty output
  - [ ] 3.1 Add Crusty target to code generator
    - Update `CodeGenerator` to support `TargetLanguage::Crusty`
    - Add `generate_crusty()` method
    - Implement dispatch to Crusty-specific generators
    - Commit with message: "feat(task-3.1): add Crusty target to code generator"
    - _Requirements: 3_
  
  - [ ] 3.2 Implement Crusty function generation
    - Translate Rust fn to Crusty function syntax
    - Translate Rust -> to Crusty return type
    - Translate Rust : Type to Crusty Type name
    - Handle pub visibility
    - Commit with message: "feat(task-3.2): implement Crusty function generation"
    - _Requirements: 3.2_
  
  - [ ] 3.3 Implement match to switch translation
    - Translate Rust match to Crusty switch
    - Translate match arms to case statements
    - Translate _ pattern to default
    - Handle multiple patterns with OR (case 1 | 2:)
    - Add break statements after each case
    - Commit with message: "feat(task-3.3): implement match to switch translation"
    - _Requirements: 3.3_
  
  - [ ] 3.4 Implement impl block to struct methods translation
    - Translate Rust impl blocks to Crusty struct methods
    - Embed methods inside struct definition
    - Handle self, &self, &mut self parameters
    - Handle static methods (no self)
    - Commit with message: "feat(task-3.4): implement impl to struct methods translation"
    - _Requirements: 3.4_
  
  - [ ] 3.5 Implement Type::method() to @Type.method() translation
    - Translate Rust Type::method() to Crusty @Type.method()
    - Translate Rust Type::CONST to Crusty @Type.CONST
    - Handle nested paths (Type::Inner::method())
    - Commit with message: "feat(task-3.5): implement static call translation"
    - _Requirements: 3.5_
  
  - [ ] 3.6 Implement turbofish to parentheses translation
    - Translate Rust Type::<T> to Crusty Type(T)
    - Translate Rust Type::<T, U> to Crusty Type(T, U)
    - Translate nested generics Type::<Inner<T>> to Type(Inner[T])
    - Handle turbofish in method calls
    - Commit with message: "feat(task-3.6): implement turbofish translation"
    - _Requirements: 3.7_
  
  - [ ] 3.7 Implement macro translation
    - Translate Rust macro_name! to Crusty __macro_name__
    - Preserve macro arguments
    - Translate delimiter: () stays (), [] stays [], {} stays {}
    - Handle common macros (println!, vec!, assert!, etc.)
    - Commit with message: "feat(task-3.7): implement macro translation"
    - _Requirements: 3.6, 9.1, 9.2_
  
  - [ ] 3.8 Write unit tests for Crusty code generation
    - Test function generation
    - Test match to switch translation
    - Test impl to struct methods translation
    - Test static call translation
    - Test turbofish translation
    - Test macro translation
    - Commit with message: "test(task-3.8): add Crusty code generation tests"
    - _Requirements: 3_

- [ ] 4. Implement Rust-specific syntax translation
  - [ ] 4.1 Implement label translation
    - Translate Rust 'label: to Crusty .label:
    - Translate Rust break 'label to Crusty break label
    - Translate Rust continue 'label to Crusty continue label
    - Commit with message: "feat(task-4.1): implement label translation"
    - _Requirements: 4.1, 4.2, 4.3_
  
  - [ ] 4.2 Implement Result<T, E> to T? translation
    - Translate Rust Result<T, E> to Crusty T?
    - Preserve ? operator (pass through)
    - Handle Result in function return types
    - Commit with message: "feat(task-4.2): implement Result translation"
    - _Requirements: 4.4, 4.5_
  
  - [ ] 4.3 Implement trait to VTable translation
    - Detect simple traits that can be VTable structs
    - Translate trait methods to function pointers
    - Generate VTable struct definition
    - Mark complex traits for __rust__{ } preservation
    - Commit with message: "feat(task-4.3): implement trait to VTable translation"
    - _Requirements: 4.6, 7.1, 7.2, 7.5_
  
  - [ ] 4.4 Implement closure to nested function translation
    - Translate Rust closures to Crusty nested functions
    - Handle capture lists
    - Preserve closure semantics
    - Commit with message: "feat(task-4.4): implement closure translation"
    - _Requirements: 4.7_
  
  - [ ] 4.5 Write unit tests for Rust-specific translation
    - Test label translation
    - Test Result translation
    - Test trait to VTable translation
    - Test closure translation
    - Commit with message: "test(task-4.5): add Rust-specific translation tests"
    - _Requirements: 4_

- [ ] 5. Handle Rust standard library
  - [ ] 5.1 Implement std library type recognition
    - Recognize Vec, String, Option, Result, HashMap, etc.
    - Preserve std library types in Crusty
    - Translate std library method calls
    - Commit with message: "feat(task-5.1): implement std library recognition"
    - _Requirements: 6.1, 6.2_
  
  - [ ] 5.2 Implement use statement translation
    - Translate Rust use to Crusty #import
    - Translate Rust pub use to Crusty #export
    - Preserve import paths
    - Commit with message: "feat(task-5.2): implement use statement translation"
    - _Requirements: 6.3, 6.4, 6.5_
  
  - [ ] 5.3 Write unit tests for std library handling
    - Test std library type recognition
    - Test std library method calls
    - Test use statement translation
    - Commit with message: "test(task-5.3): add std library tests"
    - _Requirements: 6_

### Part 3: Round-Trip Validation

- [ ] 6. Implement round-trip test framework
  - [ ] 6.1 Create round-trip validator
    - Create `RoundTripValidator` struct
    - Implement `validate_crusty_round_trip()` method
    - Implement `validate_rust_round_trip()` method
    - Commit with message: "feat(task-6.1): create round-trip validator"
    - _Requirements: 5, 12_
  
  - [ ] 6.2 Implement AST comparison
    - Implement `compare_asts()` function
    - Compare AST structure
    - Compare types
    - Compare control flow
    - Allow minor differences (formatting, comments)
    - Commit with message: "feat(task-6.2): implement AST comparison"
    - _Requirements: 5.1, 5.2, 5.3, 12.2, 12.3_
  
  - [ ] 6.3 Implement semantic equivalence checking
    - Check that functions have same signatures
    - Check that types are equivalent
    - Check that control flow is preserved
    - Report differences clearly
    - Commit with message: "feat(task-6.3): implement semantic equivalence checking"
    - _Requirements: 5.1, 5.2_
  
  - [ ] 6.4 Write unit tests for round-trip validation
    - Test AST comparison
    - Test semantic equivalence checking
    - Test with various programs
    - Commit with message: "test(task-6.4): add round-trip validation tests"
    - _Requirements: 5, 12_

- [ ] 7. Implement round-trip testing
  - [ ] 7.1 Add round-trip integration tests
    - Test Crusty → Rust → Crusty round-trip
    - Test Rust → Crusty → Rust round-trip
    - Test with Phase 1 examples
    - Test with simple Rust programs
    - Commit with message: "test(task-7.1): add round-trip integration tests"
    - _Requirements: 5.1, 5.2_
  
  - [ ] 7.2 Add --round-trip CLI option
    - Add --round-trip flag to crustyc
    - Perform round-trip validation when flag is set
    - Report validation results
    - Exit with appropriate status code
    - Commit with message: "feat(task-7.2): add round-trip CLI option"
    - _Requirements: 14.4_
  
  - [ ] 7.3 Write property-based tests for round-trip
    - **Property 1: Round-trip preserves semantics**
    - **Property 2: Bidirectional consistency**
    - **Property 3: Structure preservation**
    - **Property 4: Comment preservation**
    - Run with 1000+ iterations
    - Commit with message: "test(task-7.3): add round-trip property tests"
    - _Requirements: 5, 12.4_

### Part 4: Advanced Features

- [ ] 8. Handle lifetimes
  - [ ] 8.1 Preserve lifetime annotations
    - Recognize Rust lifetime syntax ('a, 'static)
    - Preserve lifetimes in Crusty (same syntax)
    - Handle lifetime bounds
    - Commit with message: "feat(task-8.1): preserve lifetime annotations"
    - _Requirements: 8.1, 8.2, 8.3, 8.4_
  
  - [ ] 8.2 Write unit tests for lifetime handling
    - Test lifetime preservation
    - Test lifetime bounds
    - Test lifetime elision
    - Commit with message: "test(task-8.2): add lifetime tests"
    - _Requirements: 8_

- [ ] 9. Handle attributes
  - [ ] 9.1 Preserve Rust attributes
    - Preserve #[attribute] syntax (same in Crusty)
    - Preserve #[derive(...)] attributes
    - Preserve #[cfg(...)] attributes
    - Preserve #[test] attributes
    - Commit with message: "feat(task-9.1): preserve Rust attributes"
    - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5_
  
  - [ ] 9.2 Write unit tests for attribute handling
    - Test attribute preservation
    - Test derive attributes
    - Test cfg attributes
    - Commit with message: "test(task-9.2): add attribute tests"
    - _Requirements: 10_

- [ ] 10. Handle modules
  - [ ] 10.1 Translate mod blocks to namespace
    - Translate Rust mod { } to Crusty namespace { }
    - Translate Rust mod name; to Crusty namespace name;
    - Preserve module hierarchy
    - Commit with message: "feat(task-10.1): translate mod to namespace"
    - _Requirements: 11.1, 11.2, 11.3_
  
  - [ ] 10.2 Write unit tests for module handling
    - Test mod block translation
    - Test mod declaration translation
    - Test module hierarchy
    - Commit with message: "test(task-10.2): add module tests"
    - _Requirements: 11_

- [ ] 11. Handle unsupported features
  - [ ] 11.1 Detect unsupported Rust features
    - Detect async/await
    - Detect advanced trait features (GATs, etc.)
    - Detect procedural macros
    - Report clear error messages
    - Commit with message: "feat(task-11.1): detect unsupported features"
    - _Requirements: 13.1, 13.2, 13.3, 13.4_
  
  - [ ] 11.2 Implement __rust__{ } escape hatch
    - Preserve unsupported constructs in __rust__{ } blocks
    - Document which features use escape hatch
    - Provide workaround suggestions
    - Commit with message: "feat(task-11.2): implement __rust__ escape hatch"
    - _Requirements: 13.3, 13.5_
  
  - [ ] 11.3 Write unit tests for unsupported features
    - Test error messages
    - Test __rust__{ } preservation
    - Test workaround suggestions
    - Commit with message: "test(task-11.3): add unsupported feature tests"
    - _Requirements: 13_

### Part 5: CLI and Integration

- [ ] 12. Update CLI for bidirectional transpilation
  - [ ] 12.1 Add --from-lang option
    - Support --from-lang=crusty (default)
    - Support --from-lang=rust (new)
    - Update CLI help text
    - Commit with message: "feat(task-12.1): add --from-lang option"
    - _Requirements: 14.1_
  
  - [ ] 12.2 Add --to-lang option
    - Support --to-lang=rust (default for Crusty input)
    - Support --to-lang=crusty (new, for Rust input)
    - Update CLI help text
    - Commit with message: "feat(task-12.2): add --to-lang option"
    - _Requirements: 14.2, 14.3_
  
  - [ ] 12.3 Update transpiler orchestration
    - Route to appropriate parser based on --from-lang
    - Route to appropriate code generator based on --to-lang
    - Support all combinations (Crusty→Rust, Rust→Crusty, Crusty→Crusty, Rust→Rust)
    - Commit with message: "feat(task-12.3): update transpiler orchestration"
    - _Requirements: 14.1, 14.2, 14.3_
  
  - [ ] 12.4 Add verbose mode reporting
    - Report transpilation direction in verbose mode
    - Report which parser and code generator are used
    - Report round-trip validation results
    - Commit with message: "feat(task-12.4): add verbose mode reporting"
    - _Requirements: 14.5_
  
  - [ ] 12.5 Write CLI tests
    - Test --from-lang option
    - Test --to-lang option
    - Test all transpilation combinations
    - Test verbose mode
    - Commit with message: "test(task-12.5): add CLI tests"
    - _Requirements: 14_

- [ ] 13. Integrate with crustyfmt
  - [ ] 13.1 Format generated Crusty code
    - Invoke crustyfmt on generated Crusty code
    - Apply Crusty formatting rules
    - Preserve comments
    - Commit with message: "feat(task-13.1): format generated Crusty code"
    - _Requirements: 15.1, 15.2, 15.3, 15.4, 15.5_
  
  - [ ] 13.2 Write integration tests
    - Test that generated Crusty is well-formatted
    - Test comment preservation
    - Test documentation preservation
    - Commit with message: "test(task-13.2): add formatting integration tests"
    - _Requirements: 15_

### Part 6: Testing and Validation

- [ ] 14. Corpus testing
  - [ ] 14.1 Create test corpus
    - Collect Rust std library examples
    - Collect crates.io package examples
    - Collect real-world Rust projects
    - Create test suite
    - Commit with message: "test(task-14.1): create test corpus"
    - _Requirements: 12.5_
  
  - [ ] 14.2 Run corpus tests
    - Test transpilation on entire corpus
    - Measure success rate
    - Identify common failure patterns
    - Document unsupported features
    - Commit with message: "test(task-14.2): run corpus tests"
    - _Requirements: 12.5_
  
  - [ ] 14.3 Analyze and improve
    - Fix common failure cases
    - Improve error messages
    - Update documentation
    - Commit with message: "fix(task-14.3): improve based on corpus testing"
    - _Requirements: 12.5_

- [ ] 15. Performance optimization
  - [ ] 15.1 Profile transpilation performance
    - Measure Rust parsing time
    - Measure AST conversion time
    - Measure Crusty generation time
    - Identify bottlenecks
    - Commit with message: "test(task-15.1): profile transpilation performance"
    - _Requirements: All (non-functional)_
  
  - [ ] 15.2 Optimize critical paths
    - Optimize AST conversion
    - Cache parsed results
    - Parallelize where possible
    - Commit with message: "perf(task-15.2): optimize transpilation"
    - _Requirements: All (non-functional)_
  
  - [ ] 15.3 Verify performance targets
    - Verify < 100ms Rust parsing
    - Verify < 50ms AST conversion
    - Verify < 100ms Crusty generation
    - Verify < 500ms round-trip
    - Commit with message: "test(task-15.3): verify performance targets"
    - _Requirements: All (non-functional)_

- [ ] 16. Documentation and examples
  - [ ] 16.1 Update README
    - Document bidirectional transpilation
    - Document --from-lang and --to-lang options
    - Document --round-trip option
    - Provide examples
    - Commit with message: "docs(task-16.1): update README for Phase 3"
    - _Requirements: All_
  
  - [ ] 16.2 Create user guide
    - Document Rust → Crusty transpilation
    - Document round-trip validation
    - Document unsupported features
    - Provide migration guide
    - Commit with message: "docs(task-16.2): create Phase 3 user guide"
    - _Requirements: All_
  
  - [ ] 16.3 Create example projects
    - Create Rust → Crusty example
    - Create round-trip example
    - Create std library usage example
    - Commit with message: "docs(task-16.3): create Phase 3 examples"
    - _Requirements: All_
  
  - [ ] 16.4 Update SYNTAX_REFERENCE.md
    - Document bidirectional syntax mappings
    - Document translation rules
    - Document unsupported features
    - Commit with message: "docs(task-16.4): update syntax reference"
    - _Requirements: All_

- [ ] 17. Final validation
  - [ ] 17.1 Run full test suite
    - Run all unit tests
    - Run all property tests (1000+ iterations)
    - Run all integration tests
    - Run corpus tests
    - Verify 90%+ code coverage
    - Commit with message: "test(task-17.1): validate Phase 3 test suite"
    - _Requirements: All_
  
  - [ ] 17.2 Validate against requirements
    - Verify all 15 requirements are implemented
    - Verify all 4 correctness properties are tested
    - Check for any missing functionality
    - Commit with message: "docs(task-17.2): validate Phase 3 requirements"
    - _Requirements: All_
  
  - [ ] 17.3 Manual testing
    - Test with real Rust projects
    - Test round-trip with Phase 1 examples
    - Test error handling
    - Test performance
    - Commit with message: "test(task-17.3): complete Phase 3 manual testing"
    - _Requirements: All_
  
  - [ ] 17.4 Regression testing
    - Verify Phase 1 still works
    - Verify Phase 2 still works
    - Verify no performance regression
    - Commit with message: "test(task-17.4): verify no regressions"
    - _Requirements: All_

## Summary

**Total Tasks:** 17 major tasks, 70+ subtasks
**Estimated Time:** 60-80 hours

**Critical Path:**
1. Tasks 1-2: Rust parser integration (15-20 hours)
2. Tasks 3-5: Crusty code generation (15-20 hours)
3. Tasks 6-7: Round-trip validation (10-15 hours)
4. Tasks 8-11: Advanced features (10-15 hours)
5. Tasks 12-17: CLI, testing, and validation (10-15 hours)

**Dependencies:**
- All tasks depend on Phase 1 completion
- Task 13 depends on Phase 2 (crustyfmt)
- Tasks 6-7 depend on tasks 1-5
- Tasks 14-17 depend on all previous tasks

**Key Milestones:**
- Milestone 1: Rust code parses into unified AST
- Milestone 2: Unified AST generates Crusty code
- Milestone 3: Round-trip validation works
- Milestone 4: 95%+ corpus success rate
- Milestone 5: All tests passing, documentation complete

---

**Created:** January 31, 2026  
**Status:** Ready for implementation after Phase 2 completion
