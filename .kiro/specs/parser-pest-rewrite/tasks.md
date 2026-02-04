# Implementation Plan: Pest Parser Rewrite

## Overview

This implementation plan breaks down the pest parser rewrite into discrete, incremental steps. Each task builds on previous work, with testing integrated throughout to catch errors early. The plan follows a bottom-up approach: grammar definition → basic AST building → complex constructs → error handling → integration → testing.

## Tasks

- [ ] 1. Set up pest infrastructure
  - Add pest and pest_derive dependencies to Cargo.toml
  - Create grammar.pest file in src/ directory
  - Create basic parser struct with #[derive(Parser)] attribute
  - Verify pest code generation works with a minimal grammar
  - _Requirements: 1.1, 9.5_

- [ ] 2. Define core grammar structure
  - [ ] 2.1 Define whitespace and comment rules
    - Implement WHITESPACE rule for spaces, tabs, newlines
    - Implement line_comment rule for // comments
    - Implement block_comment rule for /* */ comments
    - _Requirements: 1.5_
  
  - [ ] 2.2 Define keyword rules
    - Create rules for all Crusty keywords (let, var, const, if, else, while, for, return, break, continue, struct, enum, typedef, static, etc.)
    - Use exact string matching for keywords
    - _Requirements: 1.2_
  
  - [ ] 2.3 Define literal rules
    - Implement int_literal rule (ASCII_DIGIT+)
    - Implement float_literal rule (digits.digits)
    - Implement string_literal rule with escape sequences
    - Implement char_literal rule
    - Implement bool_literal rule (true/false)
    - Implement null_literal rule (NULL)
    - _Requirements: 1.2, 6.8_
  
  - [ ] 2.4 Define identifier rules
    - Implement ident rule for regular identifiers
    - Implement macro_ident rule for __NAME__ style macros
    - _Requirements: 1.2_

- [ ] 3. Define type expression grammar
  - [ ] 3.1 Define primitive type rules
    - Create rules for int, i32, i64, u32, u64, float, f32, f64, bool, char, void
    - _Requirements: 1.2, 6.9_
  
  - [ ] 3.2 Define complex type rules
    - Implement pointer_type rule (type ~ "*")
    - Implement reference_type rule ("&" ~ "mut"? ~ type)
    - Implement array_type rule (type ~ "[" ~ size? ~ "]")
    - Implement tuple_type rule ("(" ~ type ~ ("," ~ type)* ~ ")")
    - Implement generic_type rule (type ~ "<" ~ type ~ ("," ~ type)* ~ ">")
    - Implement slice_type rule ("[" ~ type ~ "]")
    - _Requirements: 1.2, 6.9_
  
  - [ ] 3.3 Define type_expr rule combining all type forms
    - Use ordered choice to try complex types before simple types
    - _Requirements: 1.2, 6.9_

- [ ] 4. Define expression grammar
  - [ ] 4.1 Define primary expression rules
    - Implement literal rule (all literal types)
    - Implement ident rule for variable references
    - Implement paren_expr rule for parenthesized expressions
    - Implement struct_init rule for struct initialization
    - Implement array_lit rule for array literals
    - Implement tuple_lit rule for tuple literals
    - _Requirements: 1.2, 6.8_
  
  - [ ] 4.2 Define cast expression rule (CRITICAL for ambiguity resolution)
    - Implement cast_expr rule: "(" ~ type_expr ~ ")" ~ "(" ~ expr ~ ")"
    - Place cast_expr BEFORE paren_expr in ordered choice
    - Test with examples: (int)(x), (Type*)(expr), (int)(5+3)
    - _Requirements: 2.1, 2.2, 2.4, 2.5, 2.6_
  
  - [ ] 4.3 Define call and access expression rules
    - Implement call_expr rule for function calls
    - Implement field_access rule for struct field access
    - Implement index_expr rule for array indexing
    - Implement method_call rule for method calls
    - Implement type_scoped_call rule for Type::method() syntax
    - _Requirements: 1.2, 6.8, 6.12_
  
  - [ ] 4.4 Define operator rules for PrattParser
    - Define infix operators: +, -, *, /, %, ==, !=, <, >, <=, >=, &&, ||, &, |, ^, <<, >>
    - Define prefix operators: -, !, &, *, ++, --
    - Define postfix operators: ++, --
    - Define assignment operators: =, +=, -=, *=, /=, %=, &=, |=, ^=, <<=, >>=
    - _Requirements: 1.6, 6.8_
  
  - [ ] 4.5 Define expr rule for PrattParser
    - Implement expr rule: prefix? ~ primary ~ postfix? ~ (infix ~ prefix? ~ primary ~ postfix?)*
    - This structure allows PrattParser to handle precedence
    - _Requirements: 1.6, 6.8_
  
  - [ ] 4.6 Define special expression rules
    - Implement ternary_expr rule for ? : operator
    - Implement sizeof_expr rule for sizeof(type)
    - Implement range_expr rule for .. and ..= operators
    - Implement macro_call rule for __MACRO__(args)
    - _Requirements: 1.2, 6.8, 6.15_

- [ ] 5. Define statement grammar
  - [ ] 5.1 Define variable declaration statements
    - Implement let_stmt rule (let type? ident = expr;)
    - Implement var_stmt rule (var type? ident = expr;)
    - Implement const_stmt rule (const type? ident = expr;)
    - _Requirements: 1.2, 6.7_
  
  - [ ] 5.2 Define control flow statements
    - Implement if_stmt rule (if (expr) block (else block)?)
    - Implement while_stmt rule (while (expr) block)
    - Implement for_stmt rule (for (init; cond; incr) block)
    - Implement for_in_stmt rule (for (var in expr) block)
    - Implement switch_stmt rule (switch (expr) { case ... })
    - _Requirements: 1.2, 6.7_
  
  - [ ] 5.3 Define jump statements
    - Implement return_stmt rule (return expr?;)
    - Implement break_stmt rule (break label?;)
    - Implement continue_stmt rule (continue label?;)
    - _Requirements: 1.2, 6.7_
  
  - [ ] 5.4 Define expression and block statements
    - Implement expr_stmt rule (expr;)
    - Implement block rule ({ statement* })
    - Implement labeled_loop rule (.label: loop { ... })
    - _Requirements: 1.2, 6.7, 6.11_
  
  - [ ] 5.5 Define nested function statement
    - Implement nested_function rule for function definitions inside functions
    - _Requirements: 1.2, 6.10_

- [ ] 6. Define item (top-level declaration) grammar
  - [ ] 6.1 Define attribute rule
    - Implement attribute rule (#[ident(args)])
    - Support attribute arguments (ident, literal, name=value)
    - _Requirements: 1.2, 6.5_
  
  - [ ] 6.2 Define function item rule
    - Implement function rule (visibility? return_type ident(params) block)
    - Support parameter list with types and names
    - Support self parameters for methods
    - _Requirements: 1.2, 6.1_
  
  - [ ] 6.3 Define struct item rule
    - Implement struct_def rule (struct ident { fields and methods })
    - Support field definitions with types and attributes
    - Support method definitions (functions inside struct)
    - Support static methods
    - _Requirements: 1.2, 6.2_
  
  - [ ] 6.4 Define enum item rule
    - Implement enum_def rule (enum ident { variants })
    - Support explicit variant values
    - Support attributes on enums
    - _Requirements: 1.2, 6.3_
  
  - [ ] 6.5 Define typedef item rule
    - Implement typedef rule (typedef type ident;)
    - _Requirements: 1.2, 6.4_
  
  - [ ] 6.6 Define macro definition rule
    - Implement macro_def rule (#define __NAME__ body)
    - Support macro parameters with different delimiters (parens, brackets, braces)
    - Support macro body as token sequence
    - _Requirements: 1.2, 6.6_
  
  - [ ] 6.7 Define file rule
    - Implement file rule (SOI ~ item* ~ EOI)
    - This is the top-level entry point for parsing
    - _Requirements: 1.2_

- [ ] 7. Implement AST builder infrastructure
  - [ ] 7.1 Create Parser struct with public API
    - Implement Parser::new(source: &str) -> Result<Self, ParseError>
    - Implement parse_file(&mut self, source: &str) -> Result<File, ParseError>
    - Maintain compatibility with existing API
    - _Requirements: 9.1_
  
  - [ ] 7.2 Implement error conversion function
    - Create convert_pest_error(error: pest::error::Error<Rule>) -> ParseError
    - Extract line and column numbers from pest error
    - Extract expected tokens from pest error
    - Extract found token from pest error
    - Format error message
    - _Requirements: 3.1, 3.2, 3.3, 9.2_
  
  - [ ] 7.3 Create helper functions for AST building
    - Implement extract_span(pair: &Pair<Rule>) -> Span
    - Implement extract_string(pair: &Pair<Rule>) -> String
    - Implement expect_rule(pair: &Pair<Rule>, expected: Rule) -> Result<(), ParseError>
    - _Requirements: 4.1_

- [ ] 8. Implement AST builders for literals and identifiers
  - [ ] 8.1 Implement literal AST builders
    - Create build_int_literal(pair: Pair<Rule>) -> Result<Literal, ParseError>
    - Create build_float_literal(pair: Pair<Rule>) -> Result<Literal, ParseError>
    - Create build_string_literal(pair: Pair<Rule>) -> Result<Literal, ParseError>
    - Create build_char_literal(pair: Pair<Rule>) -> Result<Literal, ParseError>
    - Create build_bool_literal(pair: Pair<Rule>) -> Result<Literal, ParseError>
    - _Requirements: 4.1, 6.8_
  
  - [ ] 8.2 Implement identifier AST builder
    - Create build_ident(pair: Pair<Rule>) -> Result<Ident, ParseError>
    - _Requirements: 4.1_

- [ ] 9. Implement AST builders for types
  - [ ] 9.1 Implement primitive type AST builder
    - Create build_primitive_type(pair: Pair<Rule>) -> Result<Type, ParseError>
    - Map keyword tokens to PrimitiveType enum variants
    - _Requirements: 4.1, 6.9_
  
  - [ ] 9.2 Implement complex type AST builders
    - Create build_pointer_type(pair: Pair<Rule>) -> Result<Type, ParseError>
    - Create build_reference_type(pair: Pair<Rule>) -> Result<Type, ParseError>
    - Create build_array_type(pair: Pair<Rule>) -> Result<Type, ParseError>
    - Create build_tuple_type(pair: Pair<Rule>) -> Result<Type, ParseError>
    - Create build_generic_type(pair: Pair<Rule>) -> Result<Type, ParseError>
    - _Requirements: 4.1, 6.9_
  
  - [ ] 9.3 Implement type_expr AST builder
    - Create build_type(pair: Pair<Rule>) -> Result<Type, ParseError>
    - Dispatch to appropriate builder based on rule type
    - _Requirements: 4.1, 6.9_

- [ ] 10. Implement AST builders for expressions
  - [ ] 10.1 Implement PrattParser setup
    - Create get_pratt_parser() -> PrattParser<Rule>
    - Define operator precedence levels (lowest to highest):
      - Assignment operators (=, +=, -=, etc.)
      - Logical OR (||)
      - Logical AND (&&)
      - Equality (==, !=)
      - Comparison (<, >, <=, >=)
      - Bitwise OR (|)
      - Bitwise XOR (^)
      - Bitwise AND (&)
      - Shift (<<, >>)
      - Addition/Subtraction (+, -)
      - Multiplication/Division/Modulo (*, /, %)
      - Prefix operators (-, !, &, *, ++, --)
      - Postfix operators (++, --)
    - _Requirements: 1.6, 6.8_
  
  - [ ] 10.2 Implement primary expression AST builders
    - Create build_literal_expr(pair: Pair<Rule>) -> Result<Expression, ParseError>
    - Create build_ident_expr(pair: Pair<Rule>) -> Result<Expression, ParseError>
    - Create build_paren_expr(pair: Pair<Rule>) -> Result<Expression, ParseError>
    - Create build_call_expr(pair: Pair<Rule>) -> Result<Expression, ParseError>
    - Create build_field_access_expr(pair: Pair<Rule>) -> Result<Expression, ParseError>
    - Create build_index_expr(pair: Pair<Rule>) -> Result<Expression, ParseError>
    - _Requirements: 4.1, 6.8_
  
  - [ ] 10.3 Implement cast expression AST builder
    - Create build_cast_expr(pair: Pair<Rule>) -> Result<Expression, ParseError>
    - Extract type and expression from cast pattern
    - Verify correct AST structure for nested casts
    - _Requirements: 2.1, 2.5, 2.6, 4.1_
  
  - [ ] 10.4 Implement struct and array literal AST builders
    - Create build_struct_init_expr(pair: Pair<Rule>) -> Result<Expression, ParseError>
    - Create build_array_lit_expr(pair: Pair<Rule>) -> Result<Expression, ParseError>
    - Create build_tuple_lit_expr(pair: Pair<Rule>) -> Result<Expression, ParseError>
    - _Requirements: 4.1, 6.8, 6.13_
  
  - [ ] 10.5 Implement special expression AST builders
    - Create build_ternary_expr(pair: Pair<Rule>) -> Result<Expression, ParseError>
    - Create build_sizeof_expr(pair: Pair<Rule>) -> Result<Expression, ParseError>
    - Create build_range_expr(pair: Pair<Rule>) -> Result<Expression, ParseError>
    - Create build_method_call_expr(pair: Pair<Rule>) -> Result<Expression, ParseError>
    - Create build_type_scoped_call_expr(pair: Pair<Rule>) -> Result<Expression, ParseError>
    - _Requirements: 4.1, 6.8, 6.12, 6.14_
  
  - [ ] 10.6 Implement expression AST builder with PrattParser
    - Create build_expression(pairs: Pairs<Rule>) -> Result<Expression, ParseError>
    - Use PrattParser with map_primary, map_prefix, map_postfix, map_infix
    - Handle all operator types and precedence
    - _Requirements: 1.6, 4.1, 6.8_

- [ ] 11. Implement AST builders for statements
  - [ ] 11.1 Implement variable declaration statement AST builders
    - Create build_let_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError>
    - Create build_var_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError>
    - Create build_const_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError>
    - _Requirements: 4.1, 6.7_
  
  - [ ] 11.2 Implement control flow statement AST builders
    - Create build_if_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError>
    - Create build_while_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError>
    - Create build_for_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError>
    - Create build_switch_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError>
    - _Requirements: 4.1, 6.7_
  
  - [ ] 11.3 Implement jump statement AST builders
    - Create build_return_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError>
    - Create build_break_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError>
    - Create build_continue_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError>
    - _Requirements: 4.1, 6.7_
  
  - [ ] 11.4 Implement block and expression statement AST builders
    - Create build_block(pair: Pair<Rule>) -> Result<Block, ParseError>
    - Create build_expr_stmt(pair: Pair<Rule>) -> Result<Statement, ParseError>
    - _Requirements: 4.1, 6.7_
  
  - [ ] 11.5 Implement statement dispatcher
    - Create build_statement(pair: Pair<Rule>) -> Result<Statement, ParseError>
    - Dispatch to appropriate builder based on rule type
    - _Requirements: 4.1, 6.7_

- [ ] 12. Implement AST builders for items
  - [ ] 12.1 Implement attribute AST builder
    - Create build_attribute(pair: Pair<Rule>) -> Result<Attribute, ParseError>
    - Handle attribute arguments (ident, literal, name=value)
    - _Requirements: 4.1, 6.5_
  
  - [ ] 12.2 Implement function item AST builder
    - Create build_function(pair: Pair<Rule>) -> Result<Function, ParseError>
    - Handle return type, parameters, body
    - Handle visibility modifiers
    - Handle attributes
    - _Requirements: 4.1, 6.1_
  
  - [ ] 12.3 Implement struct item AST builder
    - Create build_struct(pair: Pair<Rule>) -> Result<Struct, ParseError>
    - Handle fields with types and attributes
    - Handle methods (including static methods)
    - Handle struct attributes
    - _Requirements: 4.1, 6.2_
  
  - [ ] 12.4 Implement enum item AST builder
    - Create build_enum(pair: Pair<Rule>) -> Result<Enum, ParseError>
    - Handle variants with optional explicit values
    - Handle enum attributes
    - _Requirements: 4.1, 6.3_
  
  - [ ] 12.5 Implement typedef item AST builder
    - Create build_typedef(pair: Pair<Rule>) -> Result<Typedef, ParseError>
    - _Requirements: 4.1, 6.4_
  
  - [ ] 12.6 Implement macro definition item AST builder
    - Create build_macro_def(pair: Pair<Rule>) -> Result<MacroDefinition, ParseError>
    - Handle macro parameters and different delimiter types
    - Handle macro body as token sequence
    - _Requirements: 4.1, 6.6_
  
  - [ ] 12.7 Implement item dispatcher
    - Create build_item(pair: Pair<Rule>) -> Result<Item, ParseError>
    - Dispatch to appropriate builder based on rule type
    - _Requirements: 4.1_

- [ ] 13. Implement file AST builder
  - Create build_file(pairs: Pairs<Rule>) -> Result<File, ParseError>
  - Iterate through items and build AST
  - Handle empty files
  - _Requirements: 4.1_

- [ ] 14. Checkpoint - Basic parsing works
  - Ensure all tests pass, ask the user if questions arise.
  - Verify simple programs parse correctly
  - Verify AST structure matches expected format
  - Test with examples from existing test suite

- [ ] 15. Run existing parser tests
  - [ ] 15.1 Run parser_advanced_tests.rs
    - Fix any failures
    - Verify struct parsing with methods
    - Verify type-scoped calls
    - Verify generic parameters
    - _Requirements: 5.1_
  
  - [ ] 15.2 Run parser_error_tests.rs
    - Fix any failures
    - Verify error handling for empty source
    - Verify error handling for unclosed braces
    - Verify error handling for missing semicolons
    - _Requirements: 5.2_
  
  - [ ] 15.3 Run parser_coverage_tests.rs
    - Fix any failures
    - Verify all language constructs parse
    - _Requirements: 5.4_
  
  - [ ] 15.4 Run parser_edge_case_tests.rs
    - Fix any failures
    - Verify edge cases handled correctly
    - _Requirements: 5.5_
  
  - [ ] 15.5 Run parser_additional_coverage_tests.rs
    - Fix any failures
    - Verify additional coverage scenarios
    - _Requirements: 5.6_

- [ ] 16. Implement property-based tests
  - [ ] 16.1 Implement Property 1: Grammar Completeness
    - **Property 1: Grammar Completeness**
    - **Validates: Requirements 1.2, 1.7, 5.1-5.6**
    - Generate valid Crusty code from old parser tests
    - Verify new parser produces equivalent AST
    - Run 100+ iterations
  
  - [ ] 16.2 Implement Property 2: Whitespace Invariance
    - **Property 2: Whitespace Invariance**
    - **Validates: Requirements 1.5**
    - Generate valid code and whitespace variations
    - Verify AST structure unchanged (ignoring spans)
    - Run 100+ iterations
  
  - [ ] 16.3 Implement Property 3: Operator Precedence Correctness
    - **Property 3: Operator Precedence Correctness**
    - **Validates: Requirements 1.6**
    - Generate expressions with multiple operators
    - Verify AST structure reflects correct precedence
    - Test cases: 1+2*3, 1*2+3, 1+2+3, etc.
    - Run 100+ iterations
  
  - [ ] 16.4 Implement Property 4: Cast Expression Disambiguation
    - **Property 4: Cast Expression Disambiguation**
    - **Validates: Requirements 2.1, 2.2, 2.4**
    - Generate cast expressions (Type)(expr)
    - Generate parenthesized expressions (expr)
    - Generate function calls func(expr)
    - Verify correct AST node types
    - Run 100+ iterations
  
  - [ ] 16.5 Implement Property 5: Nested Cast Handling
    - **Property 5: Nested Cast Handling**
    - **Validates: Requirements 2.5**
    - Generate nested casts (T1)(T2)(expr)
    - Verify correct AST structure with nested Cast nodes
    - Run 100+ iterations
  
  - [ ] 16.6 Implement Property 6: Complex Type Cast Handling
    - **Property 6: Complex Type Cast Handling**
    - **Validates: Requirements 2.6**
    - Generate casts with pointer types (int*)(expr)
    - Generate casts with reference types (&mut T)(expr)
    - Verify correct parsing
    - Run 100+ iterations
  
  - [ ] 16.7 Implement Property 7: Error Location Reporting
    - **Property 7: Error Location Reporting**
    - **Validates: Requirements 3.1, 3.2, 3.3**
    - Generate invalid Crusty code
    - Verify error contains line, column, expected, found
    - Run 100+ iterations
  
  - [ ] 16.8 Implement Property 8: AST Structure Compatibility
    - **Property 8: AST Structure Compatibility**
    - **Validates: Requirements 4.1, 4.2, 4.3, 4.5, 9.4**
    - Generate valid code
    - Compare AST structure with old parser (if available)
    - Verify node types and fields match
    - Run 100+ iterations
  
  - [ ] 16.9 Implement Property 9: Error Type Consistency
    - **Property 9: Error Type Consistency**
    - **Validates: Requirements 9.2**
    - Generate invalid code
    - Verify error is ParseError type
    - Verify all fields populated
    - Run 100+ iterations
  
  - [ ] 16.10 Implement Property 10: Function Parsing
    - **Property 10: Function Parsing**
    - **Validates: Requirements 6.1**
    - Generate random function declarations
    - Verify Function AST node with correct fields
    - Run 100+ iterations
  
  - [ ] 16.11 Implement Property 11: Struct Parsing
    - **Property 11: Struct Parsing**
    - **Validates: Requirements 6.2**
    - Generate random struct definitions
    - Verify Struct AST node with fields and methods
    - Run 100+ iterations
  
  - [ ] 16.12 Implement Property 12: Expression Parsing
    - **Property 12: Expression Parsing**
    - **Validates: Requirements 6.8**
    - Generate random expressions
    - Verify correct Expression AST node types
    - Run 100+ iterations
  
  - [ ] 16.13 Implement Property 13: Statement Parsing
    - **Property 13: Statement Parsing**
    - **Validates: Requirements 6.7**
    - Generate random statements
    - Verify correct Statement AST node types
    - Run 100+ iterations

- [ ] 17. Run parser_properties.rs tests
  - Verify existing property tests pass
  - Fix any failures
  - _Requirements: 5.3_

- [ ] 18. Integration testing
  - [ ] 18.1 Test with semantic analyzer
    - Parse valid code and run semantic analysis
    - Verify no errors from AST structure changes
    - _Requirements: 4.5, 9.4_
  
  - [ ] 18.2 Test with code generator
    - Parse valid code and generate code
    - Verify generated code is correct
    - _Requirements: 4.5, 9.4_
  
  - [ ] 18.3 Test with pretty printer
    - Parse valid code and pretty print
    - Verify output is valid Crusty code
    - _Requirements: 4.5, 9.4_

- [ ] 19. Performance testing and optimization
  - Benchmark parsing speed on various file sizes
  - Compare with old parser performance
  - Optimize grammar rules if needed (use atomic rules, silent rules)
  - Verify no regression > 20%
  - _Requirements: 7.1, 7.2_

- [ ] 20. Documentation and cleanup
  - Add doc comments to all public functions
  - Add comments to grammar rules explaining complex patterns
  - Update README with pest parser information
  - Document any pest-specific idioms used
  - Remove old parser code (after verification)
  - _Requirements: 10.1, 10.2, 10.3, 10.4_

- [ ] 21. Final checkpoint - All tests pass
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
- The grammar is built incrementally: literals → types → expressions → statements → items
- AST builders follow the same incremental pattern
- Error handling is integrated throughout
- Integration testing ensures compatibility with existing compiler phases
