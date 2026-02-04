# Design Document: Pest Parser Rewrite

## Overview

This design document describes the architecture for rewriting the Crusty parser using the pest parser generator. The rewrite addresses fundamental issues with the current hand-written recursive descent parser, particularly around backtracking and ambiguous grammar handling. The new parser will use a formal PEG (Parsing Expression Grammar) defined in pest syntax, which eliminates ambiguity through ordered choice and provides better error messages.

The design maintains complete compatibility with the existing AST structure, ensuring that downstream compiler phases (semantic analysis, code generation, pretty printing) require no changes.

## Architecture

### High-Level Architecture

```
┌─────────────────┐
│  Source Code    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Pest Parser    │  ← Generated from grammar.pest
│  (Generated)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Parse Tree     │  ← pest::Pairs structure
│  (pest::Pairs)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  AST Builder    │  ← Custom Rust code
│  (parser.rs)    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  AST (ast.rs)   │  ← Existing AST structures
└─────────────────┘
```

### Component Breakdown

1. **Grammar File (`grammar.pest`)**: Formal PEG grammar defining Crusty syntax
2. **Parser Struct**: Rust struct with `#[derive(Parser)]` and `#[grammar = "grammar.pest"]`
3. **AST Builder**: Functions to convert pest's parse tree to existing AST nodes
4. **Error Handler**: Converts pest errors to existing `ParseError` type

### Key Design Decisions

1. **Separate Grammar from Code**: Grammar lives in `.pest` file, AST building in Rust
2. **Maintain AST Compatibility**: No changes to `ast.rs` structures
3. **Use PrattParser for Expressions**: Handles operator precedence cleanly
4. **Ordered Choice for Ambiguity**: PEG's ordered choice resolves cast vs. parenthesized expression ambiguity
5. **Silent Rules for Tokens**: Use `_` prefix for rules that don't need AST nodes (like delimiters)

## Components and Interfaces

### 1. Grammar File Structure

The grammar will be organized into logical sections:

```pest
// ============================================================================
// WHITESPACE AND COMMENTS
// ============================================================================

WHITESPACE = _{ " " | "\t" | "\r" | "\n" }
COMMENT = _{ line_comment | block_comment }
line_comment = _{ "//" ~ (!"\n" ~ ANY)* }
block_comment = _{ "/*" ~ (!"*/" ~ ANY)* ~ "*/" }

// ============================================================================
// KEYWORDS
// ============================================================================

kw_let = { "let" }
kw_var = { "var" }
kw_const = { "const" }
// ... (all keywords)

// ============================================================================
// LITERALS
// ============================================================================

int_literal = @{ ASCII_DIGIT+ }
float_literal = @{ ASCII_DIGIT+ ~ "." ~ ASCII_DIGIT+ }
string_literal = @{ "\"" ~ (!"\"" ~ ANY)* ~ "\"" }
char_literal = @{ "'" ~ (!"'" ~ ANY) ~ "'" }
bool_literal = { "true" | "false" }
null_literal = { "NULL" }

// ============================================================================
// IDENTIFIERS AND TYPES
// ============================================================================

ident = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }
macro_ident = @{ "__" ~ (ASCII_ALPHANUMERIC | "_")+ ~ "__" }

// ============================================================================
// TYPES
// ============================================================================

type_expr = { ... }
primitive_type = { "int" | "i32" | "i64" | ... }
pointer_type = { type_expr ~ "*" }
reference_type = { "&" ~ "mut"? ~ type_expr }
// ... (all type expressions)

// ============================================================================
// EXPRESSIONS
// ============================================================================

expr = { prefix? ~ primary ~ postfix? ~ (infix ~ prefix? ~ primary ~ postfix?)* }
primary = _{ literal | ident | paren_expr | cast_expr | ... }
// ... (all expressions)

// ============================================================================
// STATEMENTS
// ============================================================================

statement = { let_stmt | var_stmt | const_stmt | if_stmt | ... }
// ... (all statements)

// ============================================================================
// ITEMS (TOP-LEVEL DECLARATIONS)
// ============================================================================

item = { function | struct_def | enum_def | typedef | macro_def }
// ... (all items)

// ============================================================================
// FILE
// ============================================================================

file = { SOI ~ item* ~ EOI }
```

### 2. Parser Struct

```rust
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CrustyParser;
```

### 3. AST Builder Interface

```rust
pub struct Parser {
    // No internal state needed - pest handles everything
}

impl Parser {
    /// Create a new parser from source code
    pub fn new(source: &str) -> Result<Self, ParseError> {
        // Validate source can be tokenized
        Ok(Self {})
    }

    /// Parse a complete source file into a File AST
    pub fn parse_file(&mut self, source: &str) -> Result<File, ParseError> {
        let pairs = CrustyParser::parse(Rule::file, source)
            .map_err(|e| convert_pest_error(e))?;
        
        build_file(pairs)
    }
}

// AST building functions
fn build_file(pairs: Pairs<Rule>) -> Result<File, ParseError> { ... }
fn build_item(pair: Pair<Rule>) -> Result<Item, ParseError> { ... }
fn build_statement(pair: Pair<Rule>) -> Result<Statement, ParseError> { ... }
fn build_expression(pairs: Pairs<Rule>) -> Result<Expression, ParseError> { ... }
fn build_type(pair: Pair<Rule>) -> Result<Type, ParseError> { ... }
```

### 4. Error Conversion

```rust
fn convert_pest_error(error: pest::error::Error<Rule>) -> ParseError {
    let span = match error.location {
        pest::error::InputLocation::Pos(pos) => {
            // Convert pest position to our Span
            ...
        }
        pest::error::InputLocation::Span((start, end)) => {
            ...
        }
    };

    let message = format!("{}", error);
    let expected = extract_expected_tokens(&error);
    let found = extract_found_token(&error);

    ParseError::new(span, message, expected, found)
}
```

## Data Models

### Grammar Rules (pest)

The grammar will define rules that map closely to AST node types:

- **file**: Top-level rule matching entire source file
- **item**: Function, struct, enum, typedef, or macro definition
- **statement**: Let, var, const, if, while, for, return, etc.
- **expr**: Expressions with operator precedence
- **type_expr**: Type expressions (primitives, pointers, references, etc.)
- **literal**: Integer, float, string, char, bool, null literals
- **ident**: Identifiers
- **attribute**: Attribute syntax `#[...]`

### AST Nodes (existing)

No changes to existing AST structures in `ast.rs`. The AST builder will construct:

- `File { items, doc_comments }`
- `Item::{Function, Struct, Enum, Typedef, MacroDefinition}`
- `Statement::{Let, Var, Const, Expr, Return, If, While, For, ...}`
- `Expression::{Literal, Ident, Binary, Unary, Call, Cast, ...}`
- `Type::{Primitive, Ident, Pointer, Reference, Array, ...}`

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system—essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

Before writing correctness properties, let me analyze the acceptance criteria from the requirements:

### Acceptance Criteria Testing Prework

**Requirement 1: Formal Grammar Definition**

1.1. THE Grammar SHALL be defined in pest syntax in a `.pest` file
  Thoughts: This is about the implementation structure, not a functional requirement that can be tested with properties.
  Testable: no

1.2. THE Grammar SHALL include all Crusty language constructs
  Thoughts: This is about completeness of the grammar. We can test this by ensuring all existing test cases parse correctly.
  Testable: yes - property

1.3. THE Grammar SHALL use PEG ordered choice to resolve ambiguities
  Thoughts: This is about the grammar implementation technique, not a testable functional property.
  Testable: no

1.4. THE Grammar SHALL be documented with comments
  Thoughts: This is about code quality, not functional behavior.
  Testable: no

1.5. THE Grammar SHALL handle whitespace and comments implicitly
  Thoughts: We can test that whitespace and comments don't affect parsing results.
  Testable: yes - property

1.6. THE Grammar SHALL define precedence for operators through grammar structure
  Thoughts: We can test that operator precedence is correct by parsing expressions and checking the AST structure.
  Testable: yes - property

1.7. THE Grammar SHALL support all existing Crusty syntax
  Thoughts: This is the same as 1.2.
  Testable: yes - property (covered by 1.2)

**Requirement 2: Cast Expression Parsing**

2.1. WHEN parsing `(Type)(expr)`, THE Parser SHALL correctly identify it as a cast expression
  Thoughts: This is testing a specific pattern. We can generate random types and expressions and verify the AST node type.
  Testable: yes - property

2.2. WHEN parsing `(expr)`, THE Parser SHALL correctly identify it as a parenthesized expression
  Thoughts: This is testing a specific pattern. We can generate random expressions and verify they parse correctly.
  Testable: yes - property

2.3. WHEN parsing `(Type)`, THE Parser SHALL correctly identify it as a parenthesized type expression
  Thoughts: This is testing a specific pattern within expressions.
  Testable: yes - example

2.4. THE Parser SHALL distinguish between cast expressions and function calls
  Thoughts: This is about disambiguation. We can test that `(Type)(expr)` is a cast and `func(expr)` is a call.
  Testable: yes - property

2.5. THE Parser SHALL handle nested casts
  Thoughts: We can generate nested cast expressions and verify they parse correctly.
  Testable: yes - property

2.6. THE Parser SHALL handle casts with complex type expressions
  Thoughts: We can generate casts with pointer and reference types and verify they parse.
  Testable: yes - property

**Requirement 3: Error Message Quality**

3.1. WHEN a parse error occurs, THE Parser SHALL report the line number and column number
  Thoughts: For any invalid input, we can verify the error contains position information.
  Testable: yes - property

3.2. WHEN a parse error occurs, THE Parser SHALL report what was expected
  Thoughts: For any invalid input, we can verify the error contains expected token information.
  Testable: yes - property

3.3. WHEN a parse error occurs, THE Parser SHALL report what was found
  Thoughts: For any invalid input, we can verify the error contains found token information.
  Testable: yes - property

3.4. THE Parser SHALL provide error messages that are more descriptive than the current parser
  Thoughts: This is subjective and hard to test automatically.
  Testable: no

3.5. THE Parser SHALL leverage pest's built-in error reporting
  Thoughts: This is an implementation detail.
  Testable: no

3.6. WHEN multiple parse errors exist, THE Parser SHALL report the first error
  Thoughts: We can test that only one error is reported for invalid input.
  Testable: yes - example

**Requirement 4: AST Compatibility**

4.1. THE Parser SHALL produce AST nodes that match the existing `ast.rs` definitions
  Thoughts: For any valid input, we can verify the AST structure matches expected types.
  Testable: yes - property

4.2. THE Parser SHALL preserve all AST node types
  Thoughts: This is covered by 4.1.
  Testable: yes - property (covered by 4.1)

4.3. THE Parser SHALL preserve all AST node fields and their semantics
  Thoughts: This is covered by 4.1.
  Testable: yes - property (covered by 4.1)

4.4. THE Parser SHALL convert pest's parse tree to the existing AST structure
  Thoughts: This is an implementation detail.
  Testable: no

4.5. THE Parser SHALL maintain compatibility with existing compiler phases
  Thoughts: We can test this by running existing semantic analyzer tests.
  Testable: yes - property

**Requirement 5: Test Preservation**

5.1-5.6. THE Parser SHALL pass all existing tests
  Thoughts: These are specific test suites that must pass. Each test is an example.
  Testable: yes - example (for each test suite)

**Requirement 6: Language Feature Support**

6.1-6.15. THE Parser SHALL parse [various language features]
  Thoughts: For each feature, we can generate random valid instances and verify they parse correctly.
  Testable: yes - property (for each feature)

**Requirement 7: Performance**

7.1. THE Parser SHALL parse files at a rate comparable to or faster than the current parser
  Thoughts: This is a performance test, not a correctness property.
  Testable: no

7.2. THE Parser SHALL not introduce performance regressions greater than 20%
  Thoughts: This is a performance test.
  Testable: no

7.3. THE Parser SHALL handle large source files without excessive memory usage
  Thoughts: This is a performance/resource test.
  Testable: no

7.4. THE Parser SHALL leverage pest's optimized parsing algorithms
  Thoughts: This is an implementation detail.
  Testable: no

**Requirement 8: Maintainability**

8.1-8.6. [Maintainability requirements]
  Thoughts: These are all about code organization and documentation, not functional behavior.
  Testable: no

**Requirement 9: Integration**

9.1. THE Parser SHALL expose the same public API
  Thoughts: We can test that the API signature matches.
  Testable: yes - example

9.2. THE Parser SHALL return the same error types
  Thoughts: We can test that errors are of type `ParseError`.
  Testable: yes - property

9.3. THE Parser SHALL work with existing lexer error types
  Thoughts: This is about error type compatibility.
  Testable: yes - example

9.4. THE Parser SHALL not require changes to downstream phases
  Thoughts: This is covered by 4.5.
  Testable: yes - property (covered by 4.5)

9.5. THE Parser SHALL maintain the same module structure
  Thoughts: This is about code organization.
  Testable: no

**Requirement 10: Documentation**

10.1-10.5. [Documentation requirements]
  Thoughts: These are all about documentation quality, not functional behavior.
  Testable: no

### Property Reflection

After reviewing all testable properties, let me identify redundancies:

- Properties 1.2 and 1.7 are identical (grammar completeness)
- Properties 4.1, 4.2, and 4.3 all test AST structure compatibility
- Properties 4.5 and 9.4 both test downstream compatibility
- Properties 3.1, 3.2, and 3.3 can be combined into one comprehensive error reporting property

Consolidated properties:
- Grammar completeness (1.2)
- Whitespace/comment handling (1.5)
- Operator precedence (1.6)
- Cast expression parsing (2.1, 2.2, 2.4, 2.5, 2.6 - can be combined)
- Error reporting (3.1, 3.2, 3.3 - combined)
- AST compatibility (4.1 - covers 4.2, 4.3, 4.5, 9.4)
- Error type compatibility (9.2)
- Language feature support (6.1-6.15 - can be grouped)

### Correctness Properties

**Property 1: Grammar Completeness**
*For any* valid Crusty source code that parses with the old parser, the new pest parser should also parse it successfully and produce an equivalent AST structure.
**Validates: Requirements 1.2, 1.7, 5.1-5.6**

**Property 2: Whitespace Invariance**
*For any* valid Crusty source code, adding or removing whitespace (spaces, tabs, newlines) or comments should not change the resulting AST structure (excluding position information).
**Validates: Requirements 1.5**

**Property 3: Operator Precedence Correctness**
*For any* expression containing multiple operators, the AST structure should reflect the correct precedence and associativity (e.g., `1 + 2 * 3` should parse as `1 + (2 * 3)`).
**Validates: Requirements 1.6**

**Property 4: Cast Expression Disambiguation**
*For any* valid type T and expression E, the pattern `(T)(E)` should parse as a Cast expression node, while `(E)` should parse as the expression E itself (parentheses removed), and `func(E)` should parse as a Call expression.
**Validates: Requirements 2.1, 2.2, 2.4**

**Property 5: Nested Cast Handling**
*For any* valid types T1, T2 and expression E, the pattern `(T1)(T2)(E)` should parse as nested Cast expressions with correct structure.
**Validates: Requirements 2.5**

**Property 6: Complex Type Cast Handling**
*For any* complex type expression (pointers, references, arrays) and expression E, casts like `(int*)(E)` and `(&mut Type)(E)` should parse correctly as Cast expressions.
**Validates: Requirements 2.6**

**Property 7: Error Location Reporting**
*For any* invalid Crusty source code, the parser should produce an error that includes line number, column number, expected tokens, and found token information.
**Validates: Requirements 3.1, 3.2, 3.3**

**Property 8: AST Structure Compatibility**
*For any* valid Crusty source code, the AST produced by the new parser should have the same structure and node types as the AST that would be produced by the old parser (ignoring internal implementation details like spans).
**Validates: Requirements 4.1, 4.2, 4.3, 4.5, 9.4**

**Property 9: Error Type Consistency**
*For any* invalid Crusty source code, the parser should return an error of type `ParseError` with all required fields populated.
**Validates: Requirements 9.2**

**Property 10: Function Parsing**
*For any* valid function declaration (with various return types, parameter lists, and bodies), the parser should produce a Function AST node with correct fields.
**Validates: Requirements 6.1**

**Property 11: Struct Parsing**
*For any* valid struct definition (with fields, methods, and attributes), the parser should produce a Struct AST node with correct fields and methods.
**Validates: Requirements 6.2**

**Property 12: Expression Parsing**
*For any* valid expression (literals, binary ops, unary ops, calls, field access, etc.), the parser should produce the correct Expression AST node type.
**Validates: Requirements 6.8**

**Property 13: Statement Parsing**
*For any* valid statement (let, var, const, if, while, for, switch, return, etc.), the parser should produce the correct Statement AST node type.
**Validates: Requirements 6.7**

## Error Handling

### Error Types

The parser will continue to use the existing `ParseError` type:

```rust
pub struct ParseError {
    pub span: Span,
    pub message: String,
    pub expected: Vec<String>,
    pub found: String,
}
```

### Error Conversion Strategy

Pest provides rich error information that we'll convert to our format:

1. **Position Mapping**: Convert pest's position (byte offset) to line/column
2. **Expected Tokens**: Extract from pest's error variants
3. **Found Token**: Extract from pest's error context
4. **Message**: Use pest's formatted error message as base, enhance if needed

### Error Scenarios

1. **Lexical Errors**: Invalid characters, unterminated strings
   - Pest will catch these during parsing
   - Convert to ParseError with appropriate message

2. **Syntax Errors**: Unexpected tokens, missing delimiters
   - Pest provides expected/found information
   - Convert to ParseError with context

3. **Ambiguity Errors**: Should not occur due to PEG ordered choice
   - If they do, it's a grammar bug to fix

## Testing Strategy

### Dual Testing Approach

The parser will be tested using both unit tests and property-based tests:

**Unit Tests**:
- Specific examples of each language construct
- Edge cases (empty files, only comments, etc.)
- Error conditions (missing semicolons, unclosed braces, etc.)
- Integration with existing test suites

**Property-Based Tests**:
- Universal properties across all inputs
- Minimum 100 iterations per property test
- Each test tagged with: **Feature: parser-pest-rewrite, Property N: [property text]**

### Test Organization

1. **Grammar Tests**: Verify grammar rules parse correctly
   - Test each rule in isolation
   - Test rule combinations

2. **AST Building Tests**: Verify parse tree → AST conversion
   - Test each AST node type
   - Test nested structures

3. **Error Tests**: Verify error reporting
   - Test various syntax errors
   - Verify error message quality

4. **Regression Tests**: Ensure all existing tests pass
   - Run all existing parser test suites
   - Verify no functionality lost

5. **Property Tests**: Verify universal properties
   - Implement each correctness property as a property test
   - Use proptest or quickcheck for generation

### Property Test Configuration

Each property test will:
- Run minimum 100 iterations
- Generate random valid/invalid inputs
- Verify the property holds
- Be tagged with property number and text

Example:
```rust
#[test]
fn property_2_whitespace_invariance() {
    // Feature: parser-pest-rewrite, Property 2: Whitespace Invariance
    proptest!(|(code in valid_crusty_code(), ws in whitespace_variations())| {
        let ast1 = parse(code);
        let ast2 = parse(add_whitespace(code, ws));
        prop_assert_eq!(ast1, ast2);
    });
}
```

### Test Coverage Goals

- 100% of existing parser tests pass
- 100% of grammar rules covered by tests
- 100% of AST node types covered by tests
- All correctness properties implemented as property tests
- All error scenarios covered by unit tests

## Implementation Notes

### Grammar Design Patterns

1. **Silent Rules**: Use `_` prefix for rules that don't need AST nodes
   ```pest
   _{ "(" ~ expr ~ ")" }  // Parentheses don't create AST nodes
   ```

2. **Atomic Rules**: Use `@` for rules that should be treated as single tokens
   ```pest
   ident = @{ (ASCII_ALPHA | "_") ~ (ASCII_ALPHANUMERIC | "_")* }
   ```

3. **Ordered Choice**: Use `|` for alternatives, order matters
   ```pest
   primary = _{ cast_expr | paren_expr | literal | ident }
   // Try cast_expr first, then paren_expr, etc.
   ```

4. **Operator Precedence**: Use PrattParser for expressions
   ```rust
   let pratt = PrattParser::new()
       .op(Op::infix(Rule::add, Assoc::Left) | Op::infix(Rule::sub, Assoc::Left))
       .op(Op::infix(Rule::mul, Assoc::Left) | Op::infix(Rule::div, Assoc::Left))
       // ... more operators
   ```

### Cast Expression Disambiguation

The key to solving the cast expression ambiguity is ordered choice:

```pest
primary = _{ 
    cast_expr      // Try cast first: (Type)(expr)
    | paren_expr   // Then parenthesized: (expr)
    | literal
    | ident
    | ...
}

cast_expr = { "(" ~ type_expr ~ ")" ~ "(" ~ expr ~ ")" }
paren_expr = { "(" ~ expr ~ ")" }
```

Because PEG uses ordered choice, the parser will try `cast_expr` first. If it matches `(Type)(expr)`, it succeeds. If not, it backtracks and tries `paren_expr`.

### AST Building Strategy

1. **Top-Down Construction**: Start from `file` rule, recursively build AST
2. **Pattern Matching**: Match on pest Rule enum to determine AST node type
3. **Helper Functions**: Create helper functions for common patterns
4. **Error Propagation**: Use `?` operator to propagate errors up the call stack

Example:
```rust
fn build_expression(pairs: Pairs<Rule>) -> Result<Expression, ParseError> {
    let pratt = get_pratt_parser();
    
    pratt
        .map_primary(|primary| match primary.as_rule() {
            Rule::int_literal => Ok(Expression::Literal(Literal::Int(...))),
            Rule::ident => Ok(Expression::Ident(Ident::new(...))),
            Rule::cast_expr => build_cast_expr(primary),
            // ... more cases
        })
        .map_infix(|lhs, op, rhs| {
            Ok(Expression::Binary {
                op: convert_op(op),
                left: Box::new(lhs?),
                right: Box::new(rhs?),
            })
        })
        .parse(pairs)
}
```

### Migration Strategy

1. **Phase 1**: Create grammar file with basic structure
2. **Phase 2**: Implement AST builder for simple constructs
3. **Phase 3**: Add complex constructs (expressions, types)
4. **Phase 4**: Implement error handling
5. **Phase 5**: Run existing tests, fix issues
6. **Phase 6**: Add property tests
7. **Phase 7**: Performance testing and optimization

### Compatibility Considerations

- Keep existing `Parser::new()` and `parse_file()` API
- Return existing `ParseError` type
- Produce existing AST node types
- No changes required to downstream phases
- Existing tests should pass without modification

## Appendix: Grammar Sketch

Here's a high-level sketch of the grammar structure:

```pest
// File structure
file = { SOI ~ item* ~ EOI }

// Items (top-level declarations)
item = { attribute* ~ (function | struct_def | enum_def | typedef | macro_def) }

function = { visibility? ~ return_type ~ ident ~ "(" ~ params? ~ ")" ~ block }
struct_def = { "struct" ~ ident ~ "{" ~ (field | method)* ~ "}" }
enum_def = { "enum" ~ ident ~ "{" ~ enum_variants ~ "}" }
typedef = { "typedef" ~ type_expr ~ ident ~ ";" }
macro_def = { "#" ~ "define" ~ macro_ident ~ macro_params? ~ macro_body }

// Statements
statement = { 
    let_stmt | var_stmt | const_stmt | if_stmt | while_stmt | for_stmt 
    | switch_stmt | return_stmt | break_stmt | continue_stmt | expr_stmt 
}

// Expressions (using PrattParser)
expr = { prefix? ~ primary ~ postfix? ~ (infix ~ prefix? ~ primary ~ postfix?)* }

primary = _{ 
    cast_expr | paren_expr | literal | ident | call_expr | field_access 
    | index_expr | struct_init | array_lit | ...
}

// Types
type_expr = { 
    pointer_type | reference_type | array_type | tuple_type 
    | generic_type | primitive_type | ident_type
}

// Literals
literal = { int_literal | float_literal | string_literal | char_literal | bool_literal | null_literal }
```

This grammar structure provides:
- Clear organization by language construct
- Ordered choice for disambiguation
- Support for all Crusty features
- Foundation for PrattParser integration
- Extensibility for future features
