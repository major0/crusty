# Requirements Document

## Introduction

This specification defines the requirements for rewriting the Crusty parser from a hand-written recursive descent parser to a formal PEG (Parsing Expression Grammar) based parser using the pest parser generator. The current parser has fundamental issues with backtracking and ambiguous grammar, particularly with cast expressions like `(Type)(expr)`. The lexer's iterator-based design makes proper state restoration nearly impossible, leading to parsing bugs that cannot be fixed without a complete rewrite.

## Glossary

- **Parser**: The component that converts a stream of tokens into an Abstract Syntax Tree (AST)
- **PEG**: Parsing Expression Grammar, a formal grammar notation that eliminates ambiguity through ordered choice
- **pest**: A Rust parser generator that uses PEG grammars
- **AST**: Abstract Syntax Tree, the structured representation of source code
- **Cast_Expression**: A type cast operation in the form `(Type)(expr)`
- **Backtracking**: The ability of a parser to restore state and try alternative parsing paths
- **Grammar**: The formal specification of the language syntax
- **Lexer**: The component that converts source text into tokens (will be replaced by pest's built-in tokenization)
- **Parse_Tree**: The intermediate tree structure produced by pest before conversion to AST
- **Rule**: A named grammar production in pest syntax

## Requirements

### Requirement 1: Formal Grammar Definition

**User Story:** As a compiler developer, I want a formal PEG grammar definition, so that the parser behavior is unambiguous and maintainable.

#### Acceptance Criteria

1. THE Grammar SHALL be defined in pest syntax in a `.pest` file
2. THE Grammar SHALL include all Crusty language constructs (functions, structs, enums, typedefs, statements, expressions)
3. THE Grammar SHALL use PEG ordered choice to resolve ambiguities
4. THE Grammar SHALL be documented with comments explaining complex rules
5. THE Grammar SHALL handle whitespace and comments implicitly through pest's WHITESPACE rule
6. THE Grammar SHALL define precedence for operators through grammar structure
7. THE Grammar SHALL support all existing Crusty syntax including attributes, macros, and nested functions

### Requirement 2: Cast Expression Parsing

**User Story:** As a compiler developer, I want proper handling of cast expressions, so that all valid Crusty code parses correctly without ambiguity.

#### Acceptance Criteria

1. WHEN parsing `(Type)(expr)`, THE Parser SHALL correctly identify it as a cast expression
2. WHEN parsing `(expr)`, THE Parser SHALL correctly identify it as a parenthesized expression
3. WHEN parsing `(Type)`, THE Parser SHALL correctly identify it as a parenthesized type expression
4. THE Parser SHALL distinguish between cast expressions and function calls with parenthesized arguments
5. THE Parser SHALL handle nested casts like `(Type1)(Type2)(expr)`
6. THE Parser SHALL handle casts with complex type expressions like `(int*)(expr)` and `(&mut Type)(expr)`

### Requirement 3: Error Message Quality

**User Story:** As a Crusty user, I want clear parse error messages, so that I can fix syntax errors quickly.

#### Acceptance Criteria

1. WHEN a parse error occurs, THE Parser SHALL report the line number and column number
2. WHEN a parse error occurs, THE Parser SHALL report what was expected
3. WHEN a parse error occurs, THE Parser SHALL report what was found
4. THE Parser SHALL provide error messages that are more descriptive than the current parser
5. THE Parser SHALL leverage pest's built-in error reporting capabilities
6. WHEN multiple parse errors exist, THE Parser SHALL report the first error encountered

### Requirement 4: AST Compatibility

**User Story:** As a compiler developer, I want the parser to maintain existing AST structures, so that downstream compiler phases remain unchanged.

#### Acceptance Criteria

1. THE Parser SHALL produce AST nodes that match the existing `ast.rs` definitions
2. THE Parser SHALL preserve all AST node types (Item, Statement, Expression, Type, etc.)
3. THE Parser SHALL preserve all AST node fields and their semantics
4. THE Parser SHALL convert pest's parse tree to the existing AST structure
5. THE Parser SHALL maintain compatibility with existing semantic analyzer, code generator, and pretty printer

### Requirement 5: Test Preservation

**User Story:** As a compiler developer, I want all existing parser tests to pass, so that I can verify the rewrite maintains correctness.

#### Acceptance Criteria

1. THE Parser SHALL pass all existing unit tests in `parser_advanced_tests.rs`
2. THE Parser SHALL pass all existing error tests in `parser_error_tests.rs`
3. THE Parser SHALL pass all existing property-based tests in `parser_properties.rs`
4. THE Parser SHALL pass all existing coverage tests in `parser_coverage_tests.rs`
5. THE Parser SHALL pass all existing edge case tests in `parser_edge_case_tests.rs`
6. THE Parser SHALL pass all existing additional coverage tests in `parser_additional_coverage_tests.rs`

### Requirement 6: Language Feature Support

**User Story:** As a compiler developer, I want the parser to support all existing Crusty language features, so that no functionality is lost in the rewrite.

#### Acceptance Criteria

1. THE Parser SHALL parse function declarations with return types, parameters, and bodies
2. THE Parser SHALL parse struct definitions with fields and methods
3. THE Parser SHALL parse enum definitions with variants and explicit values
4. THE Parser SHALL parse typedef declarations
5. THE Parser SHALL parse attributes (e.g., `#[derive(Debug)]`)
6. THE Parser SHALL parse macro definitions (`#define`)
7. THE Parser SHALL parse all statement types (let, var, const, if, while, for, switch, return, break, continue)
8. THE Parser SHALL parse all expression types (literals, binary ops, unary ops, calls, field access, index, cast, ternary, etc.)
9. THE Parser SHALL parse all type expressions (primitives, pointers, references, arrays, tuples, generics)
10. THE Parser SHALL parse nested functions
11. THE Parser SHALL parse labeled loops
12. THE Parser SHALL parse method calls and type-scoped calls
13. THE Parser SHALL parse struct initialization expressions
14. THE Parser SHALL parse range expressions
15. THE Parser SHALL parse macro invocations

### Requirement 7: Performance

**User Story:** As a compiler developer, I want the parser to have acceptable performance, so that compilation times remain reasonable.

#### Acceptance Criteria

1. THE Parser SHALL parse files at a rate comparable to or faster than the current parser
2. THE Parser SHALL not introduce performance regressions greater than 20% on typical source files
3. THE Parser SHALL handle large source files (>10,000 lines) without excessive memory usage
4. THE Parser SHALL leverage pest's optimized parsing algorithms

### Requirement 8: Maintainability

**User Story:** As a compiler developer, I want the parser to be maintainable, so that future language changes are easy to implement.

#### Acceptance Criteria

1. THE Grammar SHALL be organized into logical sections (items, statements, expressions, types, etc.)
2. THE Grammar SHALL use descriptive rule names that match AST node types
3. THE Grammar SHALL include comments explaining non-obvious rules
4. THE Parser SHALL separate grammar definition (`.pest` file) from AST building (Rust code)
5. THE Parser SHALL use helper functions to reduce code duplication in AST building
6. THE Parser SHALL have clear error handling for malformed parse trees

### Requirement 9: Integration

**User Story:** As a compiler developer, I want the parser to integrate cleanly with the existing compiler pipeline, so that the rewrite is transparent to other components.

#### Acceptance Criteria

1. THE Parser SHALL expose the same public API as the current parser (`Parser::new()` and `parse_file()`)
2. THE Parser SHALL return the same error types as the current parser (`ParseError`)
3. THE Parser SHALL work with the existing lexer error types for consistency
4. THE Parser SHALL not require changes to the semantic analyzer, code generator, or pretty printer
5. THE Parser SHALL maintain the same module structure (`parser.rs`)

### Requirement 10: Documentation

**User Story:** As a compiler developer, I want the parser to be well-documented, so that I can understand and modify it in the future.

#### Acceptance Criteria

1. THE Grammar SHALL include comments explaining each major rule
2. THE Parser SHALL include doc comments on public functions
3. THE Parser SHALL include examples of grammar patterns in comments
4. THE Parser SHALL document any pest-specific idioms or patterns used
5. THE Parser SHALL include a README or design document explaining the grammar structure
