# Implementation Tasks: Crusty Compiler Phase 2

## Overview

Phase 2 implements the essential tooling ecosystem for Crusty: **crustydoc** (documentation generator) and **crustyfmt** (code formatter). These tools complete the developer experience by providing professional documentation generation and consistent code formatting capabilities.

**Core Principle**: Leverage existing Rust tooling (rustdoc) and Phase 1 infrastructure (parser, AST, code generator) to minimize implementation effort while maximizing compatibility with the Rust ecosystem.

**Important**: Each task should be committed using Conventional Commits format:
- Format: `type(scope): subject`
- Types: feat, fix, docs, test, refactor, chore
- Scope: task number (e.g., `task-1.1`)
- Body: Include "Validates: Requirements X" to reference requirements
- Example: `feat(task-1.1): implement documentation extractor`

## Tasks

### Part 1: crustydoc Implementation

- [ ] 1. Implement documentation extraction infrastructure
  - [ ] 1.1 Create crustydoc module structure
    - Create `src/crustydoc.rs` module
    - Create `DocExtractor` struct for traversing AST
    - Create `DocItem` struct for storing documentation metadata
    - Create `DocMetadata` struct for file-level documentation
    - Commit with message: "feat(task-1.1): create crustydoc module structure"
    - _Requirements: 1, 2_
  
  - [ ] 1.2 Implement documentation extraction from AST
    - Implement `extract_docs()` method to traverse AST
    - Extract doc comments from functions, structs, enums, type aliases
    - Extract module-level doc comments
    - Associate doc comments with their items
    - Preserve Markdown formatting in doc comments
    - Commit with message: "feat(task-1.2): implement documentation extraction"
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 3.1, 3.2_
  
  - [ ] 1.3 Write unit tests for documentation extraction
    - Test extraction from functions with doc comments
    - Test extraction from structs with doc comments
    - Test extraction from enums with doc comments
    - Test extraction of module-level doc comments
    - Test Markdown preservation
    - Commit with message: "test(task-1.3): add documentation extraction tests"
    - _Requirements: 1, 3_

- [ ] 2. Implement rustdoc invocation
  - [ ] 2.1 Create rustdoc invoker
    - Implement `invoke_rustdoc()` function
    - Generate Rust code from Crusty AST (reuse Phase 1 code generator)
    - Write Rust code to temporary file
    - Invoke rustdoc as subprocess with appropriate flags
    - Capture rustdoc stdout and stderr
    - Report rustdoc success or failure
    - Commit with message: "feat(task-2.1): implement rustdoc invocation"
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7_
  
  - [ ] 2.2 Implement error mapping
    - Parse rustdoc error messages
    - Track line number mappings during transpilation
    - Map Rust line numbers to Crusty line numbers
    - Rewrite error messages with Crusty file paths
    - Preserve rustdoc error descriptions and suggestions
    - Commit with message: "feat(task-2.2): implement error mapping"
    - _Requirements: 8.6_
  
  - [ ] 2.3 Write unit tests for rustdoc invocation
    - Test successful rustdoc invocation
    - Test rustdoc error handling
    - Test error message mapping
    - Test line number mapping
    - Commit with message: "test(task-2.3): add rustdoc invocation tests"
    - _Requirements: 2, 8_

- [ ] 3. Implement crustydoc CLI
  - [ ] 3.1 Create crustydoc binary
    - Create `src/bin/crustydoc.rs` entry point
    - Use clap for command-line argument parsing
    - Support `crustydoc <file.crst>` for single file
    - Support `--output <dir>` for output directory
    - Support `--open` to open docs in browser
    - Support `-D missing-docs` for documentation warnings
    - Support `--document-private-items` flag
    - Support `-- <rustdoc-options>` for pass-through options
    - Commit with message: "feat(task-3.1): create crustydoc CLI"
    - _Requirements: 8.1, 8.2_
  
  - [ ] 3.2 Implement documentation generation workflow
    - Read Crusty source file
    - Parse into AST (reuse Phase 1 parser)
    - Extract documentation metadata
    - Generate Rust code with preserved doc comments
    - Invoke rustdoc with appropriate options
    - Map errors back to Crusty source
    - Report success or failure
    - Commit with message: "feat(task-3.2): implement documentation workflow"
    - _Requirements: 2, 8_
  
  - [ ] 3.3 Write integration tests for crustydoc
    - Test end-to-end documentation generation
    - Test with various Crusty syntax features
    - Test error reporting
    - Test command-line options
    - Commit with message: "test(task-3.3): add crustydoc integration tests"
    - _Requirements: 2, 8_

- [ ] 4. Implement documentation validation
  - [ ] 4.1 Add missing documentation detection
    - Traverse AST to find public items
    - Check if public items have doc comments
    - Report missing documentation when `-D missing-docs` flag is used
    - Commit with message: "feat(task-4.1): add missing documentation detection"
    - _Requirements: 8.6_
  
  - [ ] 4.2 Add cross-reference validation
    - Parse doc comments for type references
    - Validate that referenced types exist
    - Report broken cross-references
    - Commit with message: "feat(task-4.2): add cross-reference validation"
    - _Requirements: 4.1, 4.2, 4.3, 4.4_
  
  - [ ] 4.3 Write unit tests for validation
    - Test missing documentation detection
    - Test cross-reference validation
    - Test error reporting
    - Commit with message: "test(task-4.3): add validation tests"
    - _Requirements: 4, 8_

- [ ] 5. Write property tests for crustydoc
  - [ ] 5.1 Write property test for documentation completeness
    - **Property 1: All public items with doc comments appear in generated documentation**
    - **Validates: Requirements 1, 2, 10**
    - Commit with message: "test(task-5.1): add documentation completeness property test"
  
  - [ ] 5.2 Write property test for cross-reference validity
    - **Property 2: Cross-references resolve correctly**
    - **Validates: Requirements 4**
    - Commit with message: "test(task-5.2): add cross-reference validity property test"
  
  - [ ] 5.3 Write property test for code example validity
    - **Property 3: Code examples compile successfully**
    - **Validates: Requirements 11**
    - Commit with message: "test(task-5.3): add code example validity property test"

### Part 2: crustyfmt Implementation

- [ ] 6. Enhance Pretty Printer for formatting
  - [ ] 6.1 Add formatting configuration structure
    - Create `FormatConfig` struct in `src/pretty.rs`
    - Add fields for indent_width, use_tabs, max_line_length, brace_style
    - Add fields for space_before_brace, space_after_comma
    - Implement default configuration
    - Commit with message: "feat(task-6.1): add formatting configuration"
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6_
  
  - [ ] 6.2 Enhance Pretty Printer with formatting rules
    - Update `PrettyPrinter` to accept `FormatConfig`
    - Apply indentation rules based on config
    - Apply spacing rules around operators and delimiters
    - Apply line breaking rules for long statements
    - Apply brace style rules
    - Preserve all comments in original positions
    - Commit with message: "feat(task-6.2): enhance pretty printer with formatting"
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 7.1, 7.2, 7.3, 7.4_
  
  - [ ] 6.3 Write unit tests for formatting rules
    - Test indentation application
    - Test spacing around operators
    - Test line breaking
    - Test brace style options
    - Test comment preservation
    - Commit with message: "test(task-6.3): add formatting rules tests"
    - _Requirements: 5, 6, 7_

- [ ] 7. Implement configuration loading
  - [ ] 7.1 Create configuration loader
    - Create `load_config()` function in `src/crustyfmt.rs`
    - Search for `.crustyfmt.toml` in current directory and parent directories
    - Parse TOML configuration file
    - Apply defaults for missing configuration values
    - Validate configuration values
    - Commit with message: "feat(task-7.1): implement configuration loading"
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6_
  
  - [ ] 7.2 Write unit tests for configuration
    - Test TOML parsing
    - Test default configuration
    - Test configuration validation
    - Test configuration search
    - Commit with message: "test(task-7.2): add configuration tests"
    - _Requirements: 6_

- [ ] 8. Implement crustyfmt CLI
  - [ ] 8.1 Create crustyfmt binary
    - Create `src/bin/crustyfmt.rs` entry point
    - Use clap for command-line argument parsing
    - Support `crustyfmt <file.crst>` for single file
    - Support `crustyfmt <dir>` for directory formatting
    - Support `--check` flag for check mode (no modifications)
    - Support `--config <file>` for custom configuration
    - Support `--stdin` for stdin/stdout mode (editor integration)
    - Commit with message: "feat(task-8.1): create crustyfmt CLI"
    - _Requirements: 8.2, 12.1, 12.2, 12.3, 12.4, 12.5_
  
  - [ ] 8.2 Implement formatting workflow
    - Read Crusty source file (or stdin)
    - Parse into AST (reuse Phase 1 parser)
    - Load formatting configuration
    - Apply formatting rules via Pretty Printer
    - Write formatted output (or to stdout)
    - Report formatting errors
    - Commit with message: "feat(task-8.2): implement formatting workflow"
    - _Requirements: 5, 8_
  
  - [ ] 8.3 Implement check mode
    - Parse original source
    - Format source
    - Compare original and formatted
    - Exit with status 0 if identical, non-zero if different
    - Report which files need formatting
    - Do not modify any files
    - Commit with message: "feat(task-8.3): implement check mode"
    - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.5_
  
  - [ ] 8.4 Write integration tests for crustyfmt
    - Test end-to-end formatting
    - Test with various Crusty syntax features
    - Test check mode
    - Test stdin/stdout mode
    - Test directory formatting
    - Test configuration options
    - Commit with message: "test(task-8.4): add crustyfmt integration tests"
    - _Requirements: 5, 8, 12_

- [ ] 9. Implement Crusty-specific formatting
  - [ ] 9.1 Add formatting for type-scoped calls
    - Format @Type.method() calls consistently
    - Format explicit generic parameters @Type(T)
    - Format nested generic parameters @Type(Inner[T])
    - Commit with message: "feat(task-9.1): add type-scoped call formatting"
    - _Requirements: 9.1, 9.4_
  
  - [ ] 9.2 Add formatting for macro invocations
    - Format __macro__() calls consistently
    - Preserve double-underscore naming
    - Format macro arguments
    - Commit with message: "feat(task-9.2): add macro invocation formatting"
    - _Requirements: 9.2_
  
  - [ ] 9.3 Add formatting for labeled loops
    - Format .label: loop syntax consistently
    - Format break label and continue label
    - Commit with message: "feat(task-9.3): add labeled loop formatting"
    - _Requirements: 9.3_
  
  - [ ] 9.4 Add formatting for nested functions
    - Format nested function declarations
    - Format closure syntax
    - Commit with message: "feat(task-9.4): add nested function formatting"
    - _Requirements: 9.5_
  
  - [ ] 9.5 Add formatting for typedef and impl blocks
    - Format typedef declarations
    - Format struct implementation blocks
    - Format method definitions
    - Commit with message: "feat(task-9.5): add typedef and impl formatting"
    - _Requirements: 9.6, 9.7_
  
  - [ ] 9.6 Write unit tests for Crusty-specific formatting
    - Test all Crusty-specific syntax features
    - Test formatting consistency
    - Test comment preservation
    - Commit with message: "test(task-9.6): add Crusty-specific formatting tests"
    - _Requirements: 9_

- [ ] 10. Write property tests for crustyfmt
  - [ ] 10.1 Write property test for semantic preservation
    - **Property 4: Formatting preserves semantic meaning**
    - **Validates: Requirements 5, 7**
    - Commit with message: "test(task-10.1): add semantic preservation property test"
  
  - [ ] 10.2 Write property test for idempotence
    - **Property 5: Formatting is idempotent**
    - **Validates: Requirements 5**
    - Commit with message: "test(task-10.2): add idempotence property test"
  
  - [ ] 10.3 Write property test for comment preservation
    - **Property 6: All comments are preserved**
    - **Validates: Requirements 7**
    - Commit with message: "test(task-10.3): add comment preservation property test"

### Part 3: Integration and Polish

- [ ] 11. Integrate with build tools
  - [ ] 11.1 Add build.rs integration examples
    - Create example build.rs that invokes crustydoc
    - Create example build.rs that invokes crustyfmt
    - Document integration patterns
    - Commit with message: "docs(task-11.1): add build.rs integration examples"
    - _Requirements: 8.3, 8.4_
  
  - [ ] 11.2 Add pre-commit hook for crustyfmt
    - Update `.pre-commit-config.yaml` with crustyfmt hook
    - Configure hook to run crustyfmt --check
    - Document hook installation
    - Commit with message: "feat(task-11.2): add crustyfmt pre-commit hook"
    - _Requirements: 8.4_
  
  - [ ] 11.3 Update CI/CD pipeline
    - Add crustydoc generation to CI
    - Add crustyfmt check to CI
    - Fail CI if formatting is incorrect
    - Commit with message: "feat(task-11.3): update CI/CD for Phase 2 tools"
    - _Requirements: 8.5, 8.6_

- [ ] 12. Update documentation
  - [ ] 12.1 Update README with Phase 2 tools
    - Document crustydoc usage
    - Document crustyfmt usage
    - Add examples
    - Commit with message: "docs(task-12.1): update README for Phase 2"
    - _Requirements: All_
  
  - [ ] 12.2 Create crustydoc user guide
    - Document documentation comment syntax
    - Document command-line options
    - Provide examples
    - Document integration with build.rs
    - Commit with message: "docs(task-12.2): create crustydoc user guide"
    - _Requirements: 1, 2, 3, 4, 8, 10, 11_
  
  - [ ] 12.3 Create crustyfmt user guide
    - Document formatting rules
    - Document configuration options
    - Provide examples
    - Document editor integration
    - Commit with message: "docs(task-12.3): create crustyfmt user guide"
    - _Requirements: 5, 6, 7, 8, 9, 12_
  
  - [ ] 12.4 Update SYNTAX_REFERENCE.md
    - Add documentation comment syntax
    - Add formatting guidelines
    - Commit with message: "docs(task-12.4): update syntax reference"
    - _Requirements: 1, 5_

- [ ] 13. Create example projects
  - [ ] 13.1 Create documented example project
    - Create example project with comprehensive documentation
    - Include doc comments on all public items
    - Include code examples in documentation
    - Generate documentation with crustydoc
    - Commit with message: "docs(task-13.1): create documented example project"
    - _Requirements: 1, 2, 3, 4, 10, 11_
  
  - [ ] 13.2 Create formatting example project
    - Create example project demonstrating formatting
    - Show before/after formatting examples
    - Include custom .crustyfmt.toml configuration
    - Commit with message: "docs(task-13.2): create formatting example project"
    - _Requirements: 5, 6, 7, 9_

- [ ] 14. Final validation
  - [ ] 14.1 Run full test suite
    - Run all unit tests
    - Run all property tests
    - Run all integration tests
    - Verify 90%+ code coverage
    - Commit with message: "test(task-14.1): validate Phase 2 test suite"
    - _Requirements: All_
  
  - [ ] 14.2 Validate against requirements
    - Verify all 12 requirements are implemented
    - Verify all 6 correctness properties are tested
    - Check for any missing functionality
    - Commit with message: "docs(task-14.2): validate Phase 2 requirements"
    - _Requirements: All_
  
  - [ ] 14.3 Performance testing
    - Benchmark crustydoc on typical projects
    - Benchmark crustyfmt on typical files
    - Verify performance targets met
    - Commit with message: "test(task-14.3): add Phase 2 performance tests"
    - _Requirements: All (non-functional)_
  
  - [ ] 14.4 Manual testing
    - Test crustydoc with real-world Crusty code
    - Test crustyfmt with real-world Crusty code
    - Test editor integration
    - Test CI/CD integration
    - Commit with message: "test(task-14.4): complete Phase 2 manual testing"
    - _Requirements: All_

## Summary

**Total Tasks:** 14 major tasks, 47 subtasks
**Estimated Time:** 40-50 hours

**Critical Path:**
1. Tasks 1-2: crustydoc foundation (8-10 hours)
2. Tasks 3-5: crustydoc features and testing (8-10 hours)
3. Tasks 6-7: crustyfmt foundation (6-8 hours)
4. Tasks 8-10: crustyfmt features and testing (10-12 hours)
5. Tasks 11-14: Integration, documentation, and validation (8-10 hours)

**Dependencies:**
- All tasks depend on Phase 1 completion
- crustydoc tasks (1-5) can be done in parallel with crustyfmt tasks (6-10)
- Integration tasks (11-14) depend on both crustydoc and crustyfmt completion

**Key Milestones:**
- Milestone 1: crustydoc generates basic HTML documentation
- Milestone 2: crustyfmt formats Crusty code correctly
- Milestone 3: Both tools integrated with build system and CI/CD
- Milestone 4: All tests passing, documentation complete

---

**Created:** January 31, 2026  
**Status:** Ready for implementation
