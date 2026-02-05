# Implementation Plan: Rust-PEG Parser Rewrite

## Overview

This implementation plan breaks down the rust-peg parser rewrite into discrete, incremental steps. Each task builds on previous work, with testing integrated throughout to catch errors early. The plan follows a bottom-up approach: grammar definition with embedded actions → basic constructs → complex constructs → error handling → integration → testing.

## Tasks

- [x] 1. Set up rust-peg infrastructure
  - Add peg dependency to Cargo.toml
  - Create basic peg! macro structure in parser.rs
  - Create minimal grammar with simple rule to verify code generation works
  - Test that rust-peg compiles and generates parser code
  - _Requirements: 1.1, 9.5_

- [ ] 2. Define core grammar structure with whitespace and comments
  - [x] 2.1 Define whitespace and comment rules
    - Implement _ rule for optional whitespace (quiet!)
    - Implement __ rule for required whitespace
    - Implement line_comment rule for // comments
    - Implement block_comment rule for /* */ comments
    - _Requirements: 1.5_
  
  - [x] 2.2 Define keyword rules with lookahead
    - Create rules for all Crusty keywords (let, var, const, if, else, while, for, return, break, continue, struct, enum, typedef, static, etc.)
    - Use !ident_char() lookahead to ensure keywords don't match as prefixes
    - _Requirements: 1.2_
  
  - [x] 2.3 Define literal rules with actions
    - Implement int_literal rule returning Literal::Int
    - Implement float_literal rule returning Literal::Float
    - Implement string_literal rule with escape sequences returning Literal::String
    - Implement char_literal rule returning Literal::Char
    - Implement bool_literal rule returning Literal::Bool
    - Implement null_literal rule returning Literal::Null
    - _Requirements: 1.2, 6.8_
  
  - [x] 2.4 Define identifier rules with actions
    - Implement ident_char helper rule
    - Implement keyword helper rule to exclude keywords
    - Implement ident rule returning Ident
    - Implement macro_ident rule for __NAME__ style macros
    - _Requirements: 1.2_

- [ ] 3. Define type expression grammar with precedence! macro
  - [x] 3.1 Define primitive type rules with actions
    - Create rules for int, i32, i64, u32, u64, float, f32, f64, bool, char, void
    - Return Type::Primitive variants
    - _Requirements: 1.2, 6.9_
  
  - [x] 3.2 Define complex type rules using precedence! macro
    - Implement pointer_type in precedence! (t:@ "*")
    - Implement reference_type in precedence! ("&" "mut"? _ t:@)
    - Implement array_type in precedence! (t:@ "[" _ size? _ "]")
    - Implement tuple_type rule
    - Implement generic_type rule (ident "<" types ">")
    - Implement slice_type rule
    - _Requirements: 1.2, 6.9_
  
  - [x] 3.3 Define type_expr rule with precedence! macro
    - Use precedence! to handle type operators
    - Include all type forms with correct precedence
    - Return Type enum variants
    - _Requirements: 1.2, 6.9_

- [ ] 4. Define expression grammar with precedence! macro
  - [x] 4.1 Define primary expression rules with actions
    - Implement literal_expr rule returning Expression::Literal
    - Implement ident_expr rule returning Expression::Ident
    - Implement paren_expr rule (returns inner expression)
    - Implement struct_init rule returning Expression::StructInit
    - Implement array_lit rule returning Expression::ArrayLit
    - Implement tuple_lit rule returning Expression::TupleLit
    - _Requirements: 1.2, 6.8_
  
  - [x] 4.2 Define cast expression rule (CRITICAL for ambiguity resolution)
    - Implement cast_expr rule: "(" _ t:type_expr() _ ")" _ "(" _ e:expr() _ ")"
    - Return Expression::Cast with type and expression
    - Place cast_expr BEFORE paren_expr in primary() ordered choice
    - Test with examples: (int)(x), (Type*)(expr), (int)(5+3)
    - _Requirements: 2.1, 2.2, 2.4, 2.5, 2.6_
  
  - [x] 4.3 Define call and access expression rules with actions
    - Implement call_expr rule returning Expression::Call
    - Implement field_access rule returning Expression::FieldAccess
    - Implement index_expr rule returning Expression::Index
    - Implement method_call rule returning Expression::MethodCall
    - Implement type_scoped_call rule returning Expression::TypeScopedCall
    - _Requirements: 1.2, 6.8, 6.12_
  
  - [x] 4.4 Define primary() rule with ordered choice
    - Combine all primary expressions with ordered choice
    - Ensure cast_expr is tried first
    - Return Expression enum variants
    - _Requirements: 1.6, 6.8_
  
  - [x] 4.5 Define expr rule using precedence! macro
    - Define all operator precedence levels (comma lowest to prefix/postfix highest)
    - Comma operator: , (lowest precedence, left-associative)
    - Assignment operators: =, +=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=
    - Logical operators: ||, &&
    - Comparison operators: ==, !=, <, >, <=, >=
    - Bitwise operators: |, ^, &, <<, >>
    - Arithmetic operators: +, -, *, /, %
    - Prefix operators: -, !, &, *, ++, --
    - Postfix operators: ++, --
    - Each operator returns appropriate Expression::Binary, Expression::Unary, or Expression::Comma
    - _Requirements: 1.6, 6.8, 6.16_
  
  - [x] 4.6 Define special expression rules with actions
    - Implement ternary_expr rule returning Expression::Ternary
    - Implement sizeof_expr rule returning Expression::Sizeof
    - Implement range_expr rule returning Expression::Range
    - Implement macro_call rule returning Expression::MacroCall
    - _Requirements: 1.2, 6.8, 6.15_

- [ ] 5. Define statement grammar with actions
  - [x] 5.1 Define variable declaration statements with actions
    - Implement let_stmt rule returning Statement::Let
    - Implement var_stmt rule returning Statement::Var
    - Implement const_stmt rule returning Statement::Const
    - _Requirements: 1.2, 6.7_
  
  - [x] 5.2 Define control flow statements with actions
    - Implement if_stmt rule returning Statement::If
    - Implement while_stmt rule returning Statement::While
    - Implement for_stmt rule returning Statement::For (with comma-separated initializers and increments)
    - Implement for_init_list rule for comma-separated variable declarations in for loops
    - Implement expr_list rule for comma-separated expressions in for loop increments
    - Implement for_in_stmt rule returning Statement::ForIn
    - Implement switch_stmt rule returning Statement::Switch
    - _Requirements: 1.2, 6.7, 6.17_
  
  - [x] 5.3 Define jump statements with actions
    - Implement return_stmt rule returning Statement::Return
    - Implement break_stmt rule returning Statement::Break
    - Implement continue_stmt rule returning Statement::Continue
    - _Requirements: 1.2, 6.7_
  
  - [x] 5.4 Define expression and block statements with actions
    - Implement expr_stmt rule returning Statement::Expr
    - Implement block rule returning Block
    - Implement labeled_loop rule returning Statement::LabeledLoop
    - _Requirements: 1.2, 6.7, 6.11_
  
  - [-] 5.5 Define nested function statement with actions
    - Implement nested_function rule returning Statement::NestedFunction
    - _Requirements: 1.2, 6.10_
  
  - [x] 5.6 Define statement() rule with ordered choice
    - Combine all statement types with ordered choice
    - Return Statement enum variants
    - _Requirements: 1.2, 6.7_

- [ ] 6. Define item (top-level declaration) grammar with actions
  - [ ] 6.1 Define attribute rule with actions
    - Implement attribute rule returning Attribute
    - Support attribute arguments (ident, literal, name=value)
    - _Requirements: 1.2, 6.5_
  
  - [ ] 6.2 Define function item rule with actions
    - Implement function rule returning Item::Function
    - Support parameter list with types and names
    - Support self parameters for methods
    - Support visibility modifiers
    - Support attributes
    - _Requirements: 1.2, 6.1_
  
  - [ ] 6.3 Define struct item rule with actions
    - Implement struct_def rule returning Item::Struct
    - Support field definitions with types and attributes
    - Support method definitions (functions inside struct)
    - Support static methods
    - _Requirements: 1.2, 6.2_
  
  - [ ] 6.4 Define enum item rule with actions
    - Implement enum_def rule returning Item::Enum
    - Support explicit variant values
    - Support attributes on enums
    - _Requirements: 1.2, 6.3_
  
  - [ ] 6.5 Define typedef item rule with actions
    - Implement typedef rule returning Item::Typedef
    - _Requirements: 1.2, 6.4_
  
  - [ ] 6.6 Define macro definition rule with actions
    - Implement macro_def rule returning Item::MacroDefinition
    - Support macro parameters with different delimiters (parens, brackets, braces)
    - Support macro body as token sequence
    - _Requirements: 1.2, 6.6_
  
  - [ ] 6.7 Define item() rule with ordered choice
    - Combine all item types with ordered choice
    - Handle attributes on items
    - Return Item enum variants
    - _Requirements: 1.2_
  
  - [ ] 6.8 Define file rule with actions
    - Implement file rule returning File
    - Parse multiple items
    - Handle empty files
    - _Requirements: 1.2_

- [ ] 7. Implement Parser public API and error handling
  - [ ] 7.1 Create Parser struct with public API
    - Implement Parser::new(source: &str) -> Result<Self, ParseError>
    - Implement parse_file(&mut self, source: &str) -> Result<File, ParseError>
    - Call generated crusty_parser::file() function
    - Maintain compatibility with existing API
    - _Requirements: 9.1_
  
  - [ ] 7.2 Implement error conversion function
    - Create convert_peg_error(error: peg::error::ParseError<LineCol>, source: &str) -> ParseError
    - Extract line and column numbers from error.location
    - Extract expected tokens from error.expected
    - Extract found token from source at error location
    - Format error message with context
    - _Requirements: 3.1, 3.2, 3.3, 9.2_
  
  - [ ] 7.3 Add expected! macros for better error messages
    - Add expected!("semicolon") after semicolons
    - Add expected!("closing brace") after braces
    - Add expected!("expression") where expressions are required
    - Improve error message quality throughout grammar
    - _Requirements: 3.1, 3.2, 3.3_

- [ ] 8. Test basic parsing functionality
  - Test parsing simple literals (integers, strings, booleans)
  - Test parsing simple identifiers
  - Test parsing simple types (primitives, pointers, references)
  - Test parsing simple expressions (binary ops, unary ops)
  - Test parsing comma expressions (i++, --n)
  - Test parsing for loops with comma-separated initializers and increments
  - Test parsing simple statements (let, var, return)
  - Test parsing simple functions
  - Verify AST structure matches expected format
  - _Requirements: 4.1, 6.16, 6.17_

- [ ] 9. Run existing parser tests
  - [ ] 9.1 Run parser_advanced_tests.rs
    - Fix any failures
    - Verify struct parsing with methods
    - Verify type-scoped calls
    - Verify generic parameters
    - _Requirements: 5.1_
  
  - [ ] 9.2 Run parser_error_tests.rs
    - Fix any failures
    - Verify error handling for empty source
    - Verify error handling for unclosed braces
    - Verify error handling for missing semicolons
    - _Requirements: 5.2_
  
  - [ ] 9.3 Run parser_coverage_tests.rs
    - Fix any failures
    - Verify all language constructs parse
    - _Requirements: 5.4_
  
  - [ ] 9.4 Run parser_edge_case_tests.rs
    - Fix any failures
    - Verify edge cases handled correctly
    - _Requirements: 5.5_
  
  - [ ] 9.5 Run parser_additional_coverage_tests.rs
    - Fix any failures
    - Verify additional coverage scenarios
    - _Requirements: 5.6_

- [ ] 10. Implement property-based tests
  - [ ] 10.1 Implement Property 1: Grammar Completeness
    - **Property 1: Grammar Completeness**
    - **Validates: Requirements 1.2, 1.7, 5.1-5.6**
    - Generate valid Crusty code from old parser tests
    - Verify new parser produces equivalent AST
    - Run 100+ iterations
  
  - [ ] 10.2 Implement Property 2: Whitespace Invariance
    - **Property 2: Whitespace Invariance**
    - **Validates: Requirements 1.5**
    - Generate valid code and whitespace variations
    - Verify AST structure unchanged (ignoring spans)
    - Run 100+ iterations
  
  - [ ] 10.3 Implement Property 3: Operator Precedence Correctness
    - **Property 3: Operator Precedence Correctness**
    - **Validates: Requirements 1.6**
    - Generate expressions with multiple operators
    - Verify AST structure reflects correct precedence
    - Test cases: 1+2*3, 1*2+3, 1+2+3, etc.
    - Run 100+ iterations
  
  - [ ] 10.4 Implement Property 4: Cast Expression Disambiguation
    - **Property 4: Cast Expression Disambiguation**
    - **Validates: Requirements 2.1, 2.2, 2.4**
    - Generate cast expressions (Type)(expr)
    - Generate parenthesized expressions (expr)
    - Generate function calls func(expr)
    - Verify correct AST node types
    - Run 100+ iterations
  
  - [ ] 10.5 Implement Property 5: Nested Cast Handling
    - **Property 5: Nested Cast Handling**
    - **Validates: Requirements 2.5**
    - Generate nested casts (T1)(T2)(expr)
    - Verify correct AST structure with nested Cast nodes
    - Run 100+ iterations
  
  - [ ] 10.6 Implement Property 6: Complex Type Cast Handling
    - **Property 6: Complex Type Cast Handling**
    - **Validates: Requirements 2.6**
    - Generate casts with pointer types (int*)(expr)
    - Generate casts with reference types (&mut T)(expr)
    - Verify correct parsing
    - Run 100+ iterations
  
  - [ ] 10.7 Implement Property 7: Error Location Reporting
    - **Property 7: Error Location Reporting**
    - **Validates: Requirements 3.1, 3.2, 3.3**
    - Generate invalid Crusty code
    - Verify error contains line, column, expected, found
    - Run 100+ iterations
  
  - [ ] 10.8 Implement Property 8: AST Structure Compatibility
    - **Property 8: AST Structure Compatibility**
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.5, 9.4**
    - Generate valid code
    - Compare AST structure with old parser (if available)
    - Verify node types and fields match
    - Run 100+ iterations
  
  - [ ] 10.9 Implement Property 9: Error Type Consistency
    - **Property 9: Error Type Consistency**
    - **Validates: Requirements 9.2**
    - Generate invalid code
    - Verify error is ParseError type
    - Verify all fields populated
    - Run 100+ iterations
  
  - [ ] 10.10 Implement Property 10: Function Parsing
    - **Property 10: Function Parsing**
    - **Validates: Requirements 6.1**
    - Generate random function declarations
    - Verify Function AST node with correct fields
    - Run 100+ iterations
  
  - [ ] 10.11 Implement Property 11: Struct Parsing
    - **Property 11: Struct Parsing**
    - **Validates: Requirements 6.2**
    - Generate random struct definitions
    - Verify Struct AST node with fields and methods
    - Run 100+ iterations
  
  - [ ] 10.12 Implement Property 12: Expression Parsing
    - **Property 12: Expression Parsing**
    - **Validates: Requirements 6.8**
    - Generate random expressions
    - Verify correct Expression AST node types
    - Run 100+ iterations
  
  - [ ] 10.13 Implement Property 13: Statement Parsing
    - **Property 13: Statement Parsing**
    - **Validates: Requirements 6.7**
    - Generate random statements
    - Verify correct Statement AST node types
    - Run 100+ iterations
  
  - [ ] 10.14 Implement Property 14: Comma Operator Parsing
    - **Property 14: Comma Operator Parsing**
    - **Validates: Requirements 6.16, 6.17**
    - Generate comma expressions (e.g., i++, --n)
    - Generate for loops with comma-separated initializers and increments
    - Verify correct AST structure with Expression::Comma nodes
    - Verify comma has lowest precedence (lower than assignment)
    - Test cases: `i++, j--`, `for (i=0, j=10; i<j; i++, j--)`
    - Run 100+ iterations

- [ ] 11. Run parser_properties.rs tests
  - Verify existing property tests pass
  - Fix any failures
  - _Requirements: 5.3_

- [ ] 12. Integration testing
  - [ ] 12.1 Test with semantic analyzer
    - Parse valid code and run semantic analysis
    - Verify no errors from AST structure changes
    - _Requirements: 4.5, 9.4_
  
  - [ ] 12.2 Test with code generator
    - Parse valid code and generate code
    - Verify generated code is correct
    - _Requirements: 4.5, 9.4_
  
  - [ ] 12.3 Test with pretty printer
    - Parse valid code and pretty print
    - Verify output is valid Crusty code
    - _Requirements: 4.5, 9.4_

- [ ] 13. Performance testing and optimization
  - Benchmark parsing speed on various file sizes
  - Compare with old parser performance
  - Optimize grammar rules if needed (use quiet! for whitespace, cache! for expensive rules)
  - Verify no regression > 20%
  - _Requirements: 7.1, 7.2_

- [ ] 14. Documentation and cleanup
  - Add doc comments to all public functions
  - Add comments to grammar rules explaining complex patterns
  - Update README with rust-peg parser information
  - Document any rust-peg-specific idioms used (precedence!, quiet!, expected!)
  - Remove old parser code (after verification)
  - _Requirements: 10.1, 10.2, 10.3, 10.4_

- [ ] 15. Final checkpoint - All tests pass
  - Ensure all tests pass, ask the user if questions arise.
  - Verify all existing tests pass
  - Verify all property tests pass
  - Verify integration tests pass
  - Verify performance is acceptable
  - Verify documentation is complete

## Notes

- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
- The grammar is built incrementally with embedded actions: literals → types → expressions → statements → items
- Actions build AST nodes directly within grammar rules
- Error handling uses rust-peg's expected! macro
- Integration testing ensures compatibility with existing compiler phases
- Rust-peg's precedence! macro handles operator precedence cleanly
- No intermediate parse tree - AST is built directly by grammar actions
