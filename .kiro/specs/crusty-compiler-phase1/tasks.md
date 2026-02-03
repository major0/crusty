# Implementation Plan: Crusty Transpiler Phase 1

## Overview

Phase 1 establishes Crusty as a working **one-way transpiler** from Crusty syntax to Rust. This phase implements the core language features, syntax, and transpilation infrastructure needed to write Crusty programs that compile to valid Rust code.

**Scope**: Lexer, parser, AST, semantic analysis, code generation (Crusty → Rust), CLI, module support, and all core language features.

**Out of Scope**: 
- Tooling (crustydoc, crustyfmt) → Phase 2
- Bidirectional transpilation (Rust → Crusty) → Phase 3
- Round-trip validation → Phase 3

**Core Principle**: Crusty is a **syntactic transpilation layer, not a semantic one**. See [SYNTAX_PHILOSOPHY.md](SYNTAX_PHILOSOPHY.md) for detailed rationale on what is and isn't transformed.

**Important**: Each task and sub-task should be committed using Conventional Commits format:
- Format: `type(scope): subject`
- Types: feat, fix, docs, test, refactor, chore
- Scope: task number (e.g., `task-2.1`)
- Body: Include "Validates: Requirements X.Y" to reference requirements
- Example: `feat(task-2.1): implement parser structure`

**Commit Workflow**:
1. Complete task/sub-task
2. Update tasks.md to mark task complete
3. Stage changes: `git add .`
4. Commit: `git commit -m "type(task-X.Y): description"`
5. Push to trigger CI: `git push`

## Tasks

- [x] 1. Set up development infrastructure
  - [x] 1.1 Create GitHub CI/CD pipeline
    - Create .github/workflows/ci.yml file
    - Configure workflow to trigger on push and pull requests to main branch
    - Add job matrix for Linux, macOS, and Windows
    - Add steps for: checkout, Rust toolchain setup, dependency caching, build, test, clippy, fmt check
    - Configure clippy to fail on warnings
    - Configure fmt to check formatting without modifying files
    - Add build status badge to README.md
    - _Requirements: 1.1-1.18_
  
  - [x] 1.2 Set up pre-commit hooks
    - Create .pre-commit-config.yaml file
    - Add hook for crustyc syntax validation on .crst files
    - Add hook for cargo fmt check on .rs files
    - Add hook for cargo clippy on .rs files
    - Document installation instructions in README.md
    - Test hooks locally before committing
    - _Requirements: 3.1-3.18_
  
  - [x] 1.3 Add MIT License
    - Create LICENSE.txt file with MIT License text
    - Add copyright notice with project name and year
    - Add copyright headers to all source files
    - Update README.md to mention MIT License
    - _Requirements: 4.1-4.8_
  
  - [x] 1.4 Create EditorConfig
    - Create .editorconfig file in root directory
    - Add formatting rules for .crst files (4 spaces, UTF-8, LF)
    - Add formatting rules for .rs files (4 spaces, UTF-8, LF)
    - Add formatting rules for .toml files (2 spaces)
    - Add formatting rules for .md files (2 spaces, no trim trailing whitespace)
    - Add formatting rules for .yml/.yaml files (2 spaces)
    - Document EditorConfig support in README.md
    - _Requirements: 5.1-5.16_
  
  - [x] 1.5 Commit infrastructure setup
    - Stage all infrastructure files
    - Create commit with message: "chore(task-1): set up development infrastructure"
    - Include commit body describing CI/CD, pre-commit, license, and EditorConfig setup
    - Reference requirements: "Validates: Requirements 1, 2, 3, 4, 5"
    - Push to trigger CI pipeline
    - Verify CI pipeline runs successfully
    - _Requirements: 2.1-2.21_

- [x] 2. Set up project structure and dependencies
  - Create Rust project with cargo init
  - Add dependencies to Cargo.toml: clap (CLI), syn (Rust parsing), prettyplease (Rust formatting), proptest (property testing), codespan-reporting (error messages), toml (config parsing)
  - Create module structure: cli, lexer, parser, ast, semantic, codegen, error, utils
  - Set up basic test infrastructure with unit test and property test examples
  - Commit with message: "feat(task-2): set up project structure and dependencies"
  - _Requirements: All (foundational)_

- [x] 2.6 Implement build.rs integration and multi-file support
  - [x] 2.6.1 Add --out-dir CLI option
    - Add --out-dir option to specify output directory for generated Rust files
    - Create output directory if it doesn't exist
    - Preserve source directory structure in output directory
    - _Requirements: 14.1, 14.2, 14.3_
  
  - [x] 2.6.2 Implement batch transpilation mode
    - Support transpiling multiple .crst files in a single invocation
    - Accept directory path as input to discover all .crst files
    - Transpile all discovered files to output directory
    - Report progress and errors for each file
    - _Requirements: 15.1, 15.2, 15.3, 15.4_
  
  - [x] 2.6.3 Implement module resolution
    - Note: Module resolution is handled by Rust's module system
    - #import and #export directives translate directly to Rust use statements
    - rustc resolves all module imports and dependencies
    - crustyc only needs to translate syntax (already implemented in parser/codegen)
    - _Requirements: 15.5, 15.6, 15.7, 15.8_
  
  - [x] 2.6.4 Create reference build.rs script
    - Create example build.rs that invokes crustyc
    - Discover all .crst files in src/ directory
    - Transpile to OUT_DIR preserving directory structure
    - Set up cargo:rerun-if-changed for incremental builds
    - Document build.rs integration patterns
    - _Requirements: 19.1, 19.2, 19.3, 19.4, 19.5, 19.6, 19.7_
  
  - [x] 2.6.5 Write unit tests for build.rs integration
    - Test --out-dir option
    - Test batch transpilation
    - Test module resolution
    - Test build.rs script functionality
    - _Requirements: 14.1-14.3, 15.1-15.8, 19.1-19.7_

- [x] 2.7 Create example directory structure
  - [x] 2.7.1 Create example directory and Cargo.toml
    - Create example/ directory in repository root
    - Create example/Cargo.toml with crustyc as build-dependency
    - Configure example as a binary crate
    - _Requirements: 6.1, 6.2, 6.3_
  
  - [x] 2.7.2 Create build.rs script for examples
    - Create example/build.rs script
    - Implement logic to discover all .crst files in example/src/
    - Invoke crustyc to transpile .crst files to OUT_DIR
    - Set up cargo:rerun-if-changed for incremental builds
    - _Requirements: 6.4, 6.5, 6.6, 6.7, 6.8_
  
  - [x] 2.7.3 Create basic example files
    - Create example/src/ directory
    - Create example/src/main.crst with hello world program
    - Create example/src/functions.crst with function examples
    - Create example/README.md with build and run instructions
    - _Requirements: 6.9, 6.10, 6.11, 6.12, 6.13_
  
  - [x] 2.7.4 Integrate example into CI/CD
    - Update .github/workflows/ci.yml to build example/
    - Add step to run example binary
    - Verify example builds and runs successfully in CI
    - _Requirements: 6.14, 6.15, 6.16_
  
  - [x] 2.7.5 Add advanced example files
    - Create example/src/structs.crst with struct examples
    - Create example/src/methods.crst with struct method examples
    - Create example/src/generics.crst with generic type parameter examples
    - Create example/src/attributes.crst with attribute examples
    - Create example/src/macros.crst with macro usage examples using double-underscore naming
    - Create example/src/ranges.crst with range syntax examples
    - Create example/src/slices.crst with slice examples
    - _Requirements: 6.17-6.24_
  
  - [x] 2.7.6 Update example README with advanced features
    - Update example/README.md with new examples
    - Document advanced features demonstrated
    - Add build and run instructions for each example
    - _Requirements: 6.25, 6.26_
  
  - [x] 2.7.7 Commit advanced example updates
    - Stage all new example files
    - Create commit with message: "feat(task-2.7): add advanced feature examples"
    - Reference requirements: "Validates: Requirements 6.17-6.26"
    - Note: This task adds advanced examples to the basic example directory
    - _Requirements: 6.17-6.34_

- [x] 3. Implement error handling infrastructure
  - [x] 3.1 Define error types and structures
    - Create CompilerError enum with variants for Lex, Parse, Semantic, CodeGen, Io, RustcInvocation
    - Create LexError, ParseError, SemanticError structs with span and message fields
    - Create Span and Position structs for source location tracking
    - Implement Display and Error traits for all error types
    - Commit with message: "feat(task-3.1): define error types and structures"
    - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_
  
  - [x] 3.2 Write property test for error reporting
    - **Property 2: Invalid syntax produces error reports with location**
    - **Validates: Requirements 6.2, 10.1**
    - Commit with message: "test(task-3.2): add property test for error reporting"

- [x] 4. Implement lexer for tokenization
  - [x] 4.1 Define token types and lexer structure
    - Create TokenKind enum with all keywords, operators, delimiters, literals, identifiers
    - Create Token struct with kind, span, and text fields
    - Create Lexer struct with source, position, line, column fields
    - Commit with message: "feat(task-4.1): define token types and lexer structure"
    - _Requirements: 6.1, 6.4-6.19_
  
  - [x] 4.2 Implement lexer methods
    - Implement Lexer::new() constructor
    - Implement next_token() for advancing through source
    - Implement peek_token() for lookahead
    - Implement helper methods for recognizing keywords, identifiers, numbers, strings, operators
    - Handle whitespace and comments (line and block)
    - Commit with message: "feat(task-4.2): implement lexer methods"
    - _Requirements: 6.1, 49.4_
  
  - [x] 4.3 Write unit tests for lexer
    - Test tokenization of keywords, operators, literals
    - Test error cases (unterminated strings, invalid characters)
    - Test comment handling
    - Commit with message: "test(task-4.3): add unit tests for lexer"
    - _Requirements: 6.1, 6.2_


- [x] 5. Implement AST data structures
  - [x] 5.1 Define core AST types
    - Create File, Item, Function, Struct, Enum, Typedef, Namespace, Use, Extern, Const, Static types
    - Create Statement enum with Let, Var, Const, Expr, Return, If, While, For, ForIn, Switch, Break, Continue variants
    - Create Expression enum with Literal, Ident, Binary, Unary, Call, FieldAccess, Index, Cast, Sizeof, Ternary, StructInit, ArrayLit, TupleLit, Range, MacroCall, RustBlock, ErrorProp, MethodCall variants
    - Create Type enum with Primitive, Ident, Pointer, Reference, Array, Slice, Tuple, Generic, Function, Fallible, Auto variants
    - Commit with message: "feat(task-5.1): define core AST types"
    - _Requirements: 6.3, 18.1-18.9, 19.1-19.10, 20.1-20.7, 21.1-21.14_
  
  - [x] 5.2 Define supporting AST types
    - Create Param, Field, EnumVariant, SwitchCase, Visibility, BinaryOp, UnaryOp, Literal, Ident types
    - Create Block type for statement sequences
    - Add doc_comments field to relevant AST nodes
    - Commit with message: "feat(task-5.2): define supporting AST types"
    - _Requirements: 6.3, 49.1, 49.9_
  
  - [x] 5.3 Write unit tests for AST construction
    - Test creating various AST nodes
    - Test AST node equality and cloning
    - Commit with message: "test(task-5.3): add unit tests for AST construction"
    - _Requirements: 6.3_

- [x] 6. Implement basic Crusty parser
  - [x] 6.1 Create parser structure and initialization
    - Create Parser struct with lexer and current_token fields
    - Implement Parser::new() that initializes lexer
    - Implement advance() method to move to next token
    - Implement expect() method for consuming expected tokens
    - Implement peek() method for lookahead
    - Commit with message: "feat(task-6.1): create parser structure and initialization"
    - _Requirements: 6.1_
  
  - [x] 6.2 Implement top-level item parsing
    - Implement parse_file() to parse entire source file into File AST
    - Implement parse_item() to dispatch to specific item parsers
    - Implement parse_function() for function declarations
    - Implement parse_struct() for struct definitions
    - Implement parse_enum() for enum declarations
    - Implement parse_typedef() for type aliases
    - Commit with message: "feat(task-6.2): implement top-level item parsing"
    - _Requirements: 6.1, 6.4, 6.5, 6.6, 6.7, 6.15, 30.1-30.7, 31.1-31.5_
  
  - [x] 6.3 Implement statement parsing
    - Implement parse_statement() to dispatch to specific statement parsers
    - Implement parsing for let, var, const declarations
    - Implement parsing for if/else-if/else statements
    - Implement parsing for while loops
    - Implement parsing for C-style for loops
    - Implement parsing for for-in loops
    - Implement parsing for return, break, continue statements
    - Implement parsing for labeled loops (.label: loop { ... })
    - Implement parsing for labeled break (break label) and continue (continue label)
    - Commit with message: "feat(task-6.3): implement statement parsing"
    - _Requirements: 6.8, 6.9, 6.10, 6.11, 6.12, 6.13, 6.14, 6.15, 34.1-34.5_
  
  - [x] 6.4 Implement expression parsing with precedence
    - Implement parse_expression() with operator precedence climbing
    - Implement parsing for literals (int, float, string, char, bool)
    - Implement parsing for identifiers
    - Implement parsing for binary operators (+, -, *, /, %, ==, !=, <, >, <=, >=, &&, ||, &, |, ^, <<, >>)
    - Implement parsing for unary operators (!, -, &, *, ++, --)
    - Implement parsing for function calls
    - Implement parsing for field access (. and ->)
    - Implement parsing for array indexing
    - Implement parsing for ternary operator (? :)
    - Implement parsing for type-scoped static method calls (@Type.method())
    - _Requirements: 1.14, 13.5, 21.7, 21.8, 23.1-23.21_
  
  - [x] 6.5 Implement type parsing
    - Implement parse_type() for type expressions
    - Implement parsing for primitive types (int, i32, i64, u32, u64, float, f32, f64, bool, char, void)
    - Implement parsing for pointer types
    - Implement parsing for reference types (& and var &/&mut)
    - Implement parsing for array types
    - Implement parsing for tuple types
    - Implement parsing for generic types (Type<T>)
    - _Requirements: 13.1-13.8, 14.1-14.6, 30.1-30.4, 32.1-32.7_
  
  - [x] 6.6 Write property test for valid parsing
    - **Property 1: Valid Crusty programs parse successfully**
    - **Validates: Requirements 6.1**
  
  - [x] 6.7 Write unit tests for parser
    - Test parsing of functions, structs, enums
    - Test parsing of statements and expressions
    - Test parsing of types
    - Test error recovery
    - _Requirements: 1.1, 1.2_

- [x] 6.8 Checkpoint - Ensure lexer and parser tests pass
  - Ensure all tests pass, ask the user if questions arise.


- [x] 7. Implement symbol table and type environment
  - [x] 7.1 Create symbol table structure
    - Create SymbolTable struct with scopes stack
    - Create Scope struct with symbols HashMap
    - Create Symbol struct with name, type, kind, mutable fields
    - Implement enter_scope(), exit_scope(), insert(), lookup() methods
    - _Requirements: 2.1, 2.2, 2.3_
  
  - [x] 7.2 Create type environment structure
    - Create TypeEnvironment struct with types HashMap
    - Create TypeInfo struct with name and kind fields
    - Implement register_type(), get_type(), is_compatible() methods
    - _Requirements: 2.2, 13.9_
  
  - [x] 7.3 Write unit tests for symbol table
    - Test scope management
    - Test symbol insertion and lookup
    - Test duplicate detection
    - _Requirements: 2.1, 2.3_

- [x] 8. Implement semantic analyzer
  - [x] 8.1 Create semantic analyzer structure
    - Create SemanticAnalyzer struct with symbol_table, type_env, errors fields
    - Implement analyze() method that validates entire File AST
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_
  
  - [x] 8.2 Implement item analysis
    - Implement analyze_item() to dispatch to specific analyzers
    - Implement analyze_function() to validate function signatures and bodies
    - Implement analyze_struct() to validate struct definitions
    - Implement analyze_enum() to validate enum definitions
    - Register types and symbols in appropriate tables
    - _Requirements: 2.1, 2.3, 26.6, 26.7_
  
  - [x] 8.3 Implement statement analysis
    - Implement analyze_statement() to validate statements
    - Check variable declarations and assignments
    - Validate control flow statements
    - Ensure variables are declared before use
    - _Requirements: 2.1, 2.2_
  
  - [x] 8.4 Implement expression type checking
    - Implement analyze_expression() that returns inferred type
    - Check type compatibility for binary operations
    - Check type compatibility for function calls
    - Validate field access and array indexing
    - Check cast validity
    - _Requirements: 2.2, 2.3, 13.9, 21.6, 21.7_
  
  - [x] 8.5 Implement unsupported feature detection
    - Detect and reject C unions
    - Detect and reject goto statements
    - Detect and reject #include directives
    - Report clear error messages explaining why features are unsupported
    - _Requirements: 6.19, 6.20, 6.21, 10.3, 17.1-17.7, 47.9_
  
  - [x] 8.6 Write property test for type checking
    - **Property 28: Type checking matches Rust semantics**
    - **Validates: Requirements 18.9**
  
  - [x] 8.7 Write unit tests for semantic analysis
    - Test type checking for various expressions
    - Test error detection (undefined variables, type mismatches)
    - Test unsupported feature detection
    - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 9. Implement Rust code generator
  - [x] 9.1 Create code generator structure
    - Create CodeGenerator struct with target language and indent level
    - Implement generate() method that produces Rust source from AST
    - Implement helper methods for indentation and formatting
    - _Requirements: 3.1, 3.2_
  
  - [x] 9.2 Implement item code generation
    - Implement generate_item() to dispatch to specific generators
    - Implement generate_function() for function definitions
    - Translate C-style function syntax to Rust syntax
    - Translate void return type to no return annotation
    - Translate static functions to private Rust functions
    - Translate non-static functions to public Rust functions
    - _Requirements: 3.3, 3.4, 3.5, 3.6, 3.7, 3.8_
  
  - [x] 9.3 Implement statement code generation
    - Implement generate_statement() for all statement types
    - Translate let, var, const declarations to Rust
    - Translate if/else-if/else to Rust syntax
    - Translate while loops to Rust
    - Translate for loops (both C-style and for-in)
    - Translate break and continue statements
    - Translate labeled loops (.label: to 'label:)
    - Translate labeled break/continue (break label to break 'label, continue label to continue 'label)
    - Note: The dot is a prefix for label declarations only, not part of the label name
    - _Requirements: 3.9, 3.10, 3.11, 3.12, 3.13, 6.13, 6.14, 6.15, 29.7, 29.8, 29.9, 38.3-38.11_
  
  - [x] 9.4 Implement expression code generation
    - Implement generate_expression() for all expression types
    - Translate binary and unary operators
    - Translate function calls and method calls
    - Translate field access and array indexing
    - Translate C-style casts to Rust 'as' operator
    - Translate sizeof to std::mem::size_of
    - Translate increment/decrement operators with correct semantics
    - Translate type-scoped static calls (@Type.method()) to Rust Type::method()
    - _Requirements: 3.13, 21.5, 21.13, 22.6, 23.10-23.13_
  
  - [x] 9.5 Implement type code generation
    - Implement generate_type() for all type variants
    - Translate primitive types
    - Translate pointer types to references where safe
    - Translate reference syntax (& and var &/&mut)
    - Translate array and slice types
    - Translate tuple types (pass through unchanged)
    - Translate generic types (pass through unchanged)
    - _Requirements: 3.14, 3.15, 14.7, 30.10, 30.11, 32.7_
  
  - [x] 9.6 Implement struct and enum code generation
    - Implement generate_struct() for struct definitions
    - Translate struct fields with visibility
    - Translate struct methods to impl blocks
    - Implement generate_enum() for enum definitions
    - Translate C-style enums to Rust enums with discriminants
    - _Requirements: 3.14, 16.8, 26.8, 26.9_


  - [x] 9.7 Write property tests for code generation
    - **Property 4: Generated Rust code is syntactically valid**
    - **Validates: Requirements 8.1**
    - **Property 6: Transparent syntax preservation**
    - **Validates: Requirements 19.7, 20.4, 23.6, 25.8, 26.8**
    - **Property 7: Variable declarations translate correctly**
    - **Validates: Requirements 35.7, 35.8, 35.9**
    - **Property 8: Reference syntax translates correctly**
    - **Validates: Requirements 36.10, 36.11**
    - **Property 23: Label syntax translates correctly**
    - **Validates: Requirements 6.13, 6.14, 6.15**
  
  - [x] 9.8 Write unit tests for code generation
    - Test generation of functions, structs, enums
    - Test generation of statements and expressions
    - Test generation of types
    - Test specific translation rules
    - _Requirements: 3.1, 3.2, 3.3-3.16_

- [x] 10. Implement pretty printer and formatting
  - [x] 10.1 Integrate prettyplease for Rust formatting
    - Use prettyplease crate to format generated Rust code
    - Ensure output follows Rust style conventions
    - _Requirements: 3.16_
  
  - [x] 10.2 Implement Crusty pretty printer
    - Create PrettyPrinter for Crusty source code
    - Implement formatting rules for Crusty syntax
    - Support round-trip: AST → Crusty source → AST
    - _Requirements: 11.1_
  
  - [x] 10.3 Write property test for pretty printing
    - **Property 27: Pretty-print then parse is identity (CRITICAL)**
    - **Validates: Requirements 16.1, 16.2**
  
  - [x] 10.4 Write property test for formatting
    - **Property 5: Generated Rust code follows formatting conventions**
    - **Validates: Requirements 8.16**

- [x] 11. Checkpoint - Ensure code generation tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 12. Implement CLI and file I/O
  - [x] 12.1 Create CLI argument parser
    - Use clap crate to define command-line options
    - Support -o/--output for output file path
    - Support --emit for output mode (rust, binary, ast)
    - Support --from-lang for source language (crusty, rust)
    - Support -v/--verbose for detailed output
    - Support --no-compile to skip rustc invocation
    - Support --version and -h/--help
    - _Requirements: 6.1-6.6, 7.1-7.13_
  
  - [x] 12.2 Implement file I/O operations
    - Implement reading source files from disk
    - Implement writing generated code to output files
    - Handle file not found errors
    - Handle file access errors
    - Handle file write errors
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6_
  
  - [x] 12.3 Implement transpiler orchestration
    - Create run_compiler() function that coordinates transpilation pipeline
    - Read source file
    - Parse source (Crusty or Rust based on --from-lang)
    - Run semantic analysis
    - Generate target code
    - Write output file
    - Optionally invoke rustc
    - _Requirements: 3.1, 3.2, 4.1, 4.2, 4.3, 4.4_
  
  - [x] 12.4 Write property test for file I/O
    - **Property 29: Valid file paths are read successfully**
    - **Validates: Requirements 11.1**
  
  - [x] 12.5 Write unit tests for CLI
    - Test argument parsing
    - Test file I/O operations
    - Test error handling
    - _Requirements: 6.1-6.6, 7.1-7.13_

- [x] 13. Implement rustc invocation
  - [x] 13.1 Create rustc invoker
    - Implement function to invoke rustc as subprocess
    - Pass generated Rust code to rustc
    - Capture rustc output (stdout and stderr)
    - Report compilation success or failure
    - _Requirements: 4.1, 4.2, 4.3, 4.4_
  
  - [x] 13.2 Handle rustc errors
    - Parse rustc error messages
    - Report rustc errors to user
    - Preserve error locations and descriptions
    - _Requirements: 4.3, 5.1, 5.2_
  
  - [x] 13.3 Write unit tests for rustc invocation
    - Test successful compilation
    - Test compilation failures
    - Test error message handling
    - _Requirements: 4.1, 4.2, 4.3_

- [x] 14. Implement advanced parsing features
  - [x] 14.1 Add support for struct methods
    - Parse C++-style method definitions within structs
    - Parse self, &self, var &self parameters
    - Parse static methods (no self parameter)
    - Parse type-scoped static method calls with @ prefix and dot notation (@Type.method())
    - Support nested type paths with @ prefix and dot notation (@Foo.Bar.boo() matching Rust's Foo::Bar.boo())
    - _Requirements: 16.1-16.7, 21.7, 21.8, 23.7, 23.8_
  
  - [x] 14.2 Add support for explicit generic type parameters
    - Parse explicit generic type parameters using parentheses syntax (@Type(T))
    - Parse nested generic type parameters using alternating parentheses and brackets (@Type(Inner[T]))
    - Parse multiple type parameters separated by commas (@Type(T1, T2))
    - Require @ prefix for all type-scoped calls with or without explicit generic parameters
    - Support omitting generic parameters when types can be fully inferred (@Type.method())
    - Reject type-scoped calls without @ prefix as syntax errors
    - Validate that parentheses and brackets alternate correctly in nested generics
    - Validate that the number of type parameters matches the generic type definition
    - _Requirements: 38.1-38.28_
  
  - [x] 14.3 Add support for attributes
    - Parse #[attribute] syntax
    - Parse #[derive(...)] for trait derivation
    - Parse #[test], #[cfg(...)] attributes
    - Support attributes on structs, enums, functions, fields
    - _Requirements: 19.1-19.9_
  
  - [x] 14.4 Add support for macros
    - Parse Crusty macro invocation syntax with double-underscore naming (__macro_name__(args), __macro_name__[args], __macro_name__{args})
    - Support common macros with double-underscore naming (__println__(...), __vec__[...], __assert__(...), __panic__(...))
    - Parse macro invocations in expression and statement contexts
    - Note: Crusty macros do NOT use ! suffix - it is added during transpilation to Rust
    - _Requirements: 23.1-23.6_
  
  - [x] 14.5 Add support for ranges and slices
    - Parse range syntax (start..end, start..=end, .., start.., ..end)
    - Parse slice type syntax (&[Type], var &[Type])
    - Parse slice indexing (arr[start..end])
    - _Requirements: 20.1-20.11_
  
  - [x] 14.6 Add support for array and tuple literals
    - Parse array literal syntax [value1, value2, value3]
    - Parse array initialization syntax [value; count]
    - Parse tuple literal syntax (value1, value2, value3)
    - Parse tuple indexing (.0, .1, .2)
    - _Requirements: 14.1-14.10, 15.1-15.6_

  - [x] 14.7 Write property test for explicit generic parameters
    - **Property 24: Explicit generic parameters translate correctly**
    - **Validates: Requirements 38.18, 38.19, 38.20, 38.21**

  - [x] 14.8 Write unit tests for advanced parsing
    - Test struct method parsing
    - Test explicit generic parameter parsing with parentheses and brackets
    - Test generic parameter nesting and alternation
    - Test omitting generic parameters when types can be inferred
    - Test attribute parsing
    - Test macro parsing
    - Test range and slice parsing
    - Test array and tuple literal parsing
    - _Requirements: 14.1-14.10, 15.1-15.7, 16.1-16.7, 18.1-18.6, 19.1-19.9, 20.1-20.11, 38.1-38.28_

- [x] 14.9 Update example directory with advanced features
  - [x] 14.9.1 Add struct method examples
    - Create example/src/methods.crst with struct method examples
    - Include static method calls using @Type.method() syntax
    - Include instance method calls
    - _Requirements: 6.17, 6.18_
  
  - [x] 14.9.2 Add generic type parameter examples
    - Create example/src/generics.crst with generic examples
    - Include explicit generic parameters using parentheses/brackets syntax
    - Include type inference examples
    - _Requirements: 6.19, 6.20_
  
  - [x] 14.9.3 Add attribute and macro examples
    - Create example/src/attributes.crst with attribute examples
    - Create example/src/macros.crst with macro usage examples using double-underscore naming
    - Include __println__, __vec__, __assert__ examples (no ! suffix in Crusty)
    - _Requirements: 6.21, 6.22_
  
  - [x] 14.9.4 Add range and slice examples
    - Create example/src/ranges.crst with range syntax examples
    - Create example/src/slices.crst with slice examples
    - _Requirements: 6.23, 6.24_
  
  - [x] 14.9.5 Update example README
    - Update example/README.md with new examples
    - Document advanced features demonstrated
    - Add build and run instructions for each example
    - _Requirements: 6.25, 6.26_
  
  - [x] 14.9.6 Commit example updates
    - Stage all new example files
    - Create commit with message: "feat(task-14.9): add advanced feature examples"
    - Reference requirements: "Validates: Requirements 6.17-6.26"
    - _Requirements: 6.17-6.34_
  
  - [x] 14.9.7 Note: This task depends on Task 2.7 (Create example directory)
    - Note: Completed as part of Task 2.7.5-2.7.7

- [x] 15. Implement #define macro support
  - [x] 15.1 Add #define parsing with delimiter types
    - Parse #define directive with double-underscore macro names (__MACRO_NAME__)
    - Parse macro parameters with different delimiter types (parentheses, brackets, braces, none)
    - Detect and store delimiter type (MacroDelimiter::None, Parens, Brackets, Braces)
    - Parse macro body as token sequence
    - Support macros with and without parameters
    - Create MacroDefinition AST node with delimiter field
    - Validate macro names have double-underscore prefix and suffix
    - _Requirements: 26.1, 26.2, 26.3, 26.4, 26.5, 26.6, 26.7, 26.8_
  
  - [x] 15.2 Implement macro invocation parsing
    - Recognize double-underscore pattern in identifiers as potential macro invocations
    - Build macro registry during parsing to track delimiter types
    - Parse macro invocations with correct delimiter based on #define declaration
    - Create Expression::MacroCall for macro invocations, NOT Expression::Call
    - Report error when macro is invoked with wrong delimiter type
    - _Requirements: 26.12, 26.13, 26.14_
  
  - [x] 15.3 Implement #define to macro_rules! translation
    - Translate double-underscore macro name to Rust snake_case macro_rules! name (removing underscores, adding !)
    - Translate parameters to Rust pattern variables ($param:expr)
    - Wrap macro body in Rust macro syntax
    - Translate delimiter type to appropriate Rust macro invocation syntax
    - Translate ternary operators to if-else expressions
    - Translate macro invocations in body (remove __, add !)
    - _Requirements: 26.15, 26.16, 26.17, 26.18, 26.19_
  
  - [x] 15.4 Add macro validation
    - Validate #define syntax
    - Verify macro names have double-underscore prefix and suffix
    - Verify macro parameters are used consistently
    - Check for valid macro body structure
    - Prohibit function definitions with double-underscore pattern (reserved for macros)
    - Skip type checking for Expression::MacroCall nodes in semantic analyzer
    - _Requirements: 26.20, 26.21, 26.22, 26.23, 26.24_
  
  - [x] 15.5 Write property test for #define translation
    - **Property 22: #define macros translate to macro_rules!**
    - **Validates: Requirements 26.15, 26.16, 26.17**
  
  - [x] 15.6 Write unit tests for #define macros with delimiter types
    - Test parsing of simple macros with double-underscores
    - Test parsing of macros with different delimiter types (parens, brackets, braces, none)
    - Test parsing of macros with parameters
    - Test translation to macro_rules! (removing double-underscores)
    - Test macro invocations within macro bodies
    - Test error when function uses double-underscore pattern
    - Test macro invocation parsing with correct delimiter types
    - Test error when macro invoked with wrong delimiter type
    - _Requirements: 26.1-26.26_

- [x] 16. Implement advanced code generation features
  - [x] 16.1 Add struct method code generation
    - Translate struct methods to Rust impl blocks
    - Translate self parameters correctly
    - Translate static methods (associated functions)
    - Translate @Type.method() calls to Rust Type::method()
    - _Requirements: 16.8, 16.9, 16.10, 16.11, 21.13_
  
  - [x] 16.2 Add explicit generic parameter code generation
    - Translate parentheses syntax to Rust turbofish with angle brackets (Type(T) → Type::<T>)
    - Translate nested generics with alternating parentheses/brackets to nested angle brackets (Type(Inner[T]) → Type::<Inner<T>>)
    - Translate multiple type parameters (Type(T1, T2) → Type::<T1, T2>)
    - Omit turbofish syntax when generic parameters are not specified, relying on Rust's type inference
    - Handle optional @ prefix correctly
    - _Requirements: 38.18, 38.19, 38.20, 38.21_
  
  - [x] 16.3 Add typedef code generation
    - Translate typedef to Rust type aliases
    - Handle struct typedef patterns
    - _Requirements: 25.9, 25.10_
  
  - [x] 16.4 Add NULL and Option code generation
    - Translate NULL to @Option.None (which becomes Option::None in Rust)
    - Translate NULL comparisons to is_none()/is_some() (special case transformation)
    - _Requirements: 28.4, 28.5, 28.6, 28.7, 28.8_
    - _Note: NULL is the ONLY semantic transformation - it's a C keyword with no Rust equivalent_
  
  - [x] 16.5 Add struct initializer code generation
    - Translate C-style designated initializers to Rust struct literals
    - Handle partial initialization
    - Handle nested struct initialization
    - _Requirements: 33.6, 33.7, 33.8, 33.9_
  
  - [x] 16.6 Add switch statement code generation
    - Translate switch statements to Rust match expressions
    - Translate case labels to match arms
    - Translate multiple case values to OR patterns
    - Translate default case to wildcard pattern
    - _Requirements: 45.7, 45.8, 45.9, 45.10_
  
  - [x] 16.7 Add error handling code generation
    - Translate Type? to Result<Type, E> (syntax transformation)
    - expr? passes through to Rust ? operator (no transformation needed)
    - Method names pass through unchanged (.is_err(), .is_ok(), .unwrap())
    - Function names pass through unchanged (Ok(), Err())
    - _Requirements: 46.8, 46.9, 46.10 (updated to syntax-only)_
    - _Note: Crusty follows transparent syntax transpilation - only syntax transforms, not semantics_
  
  - [x] 16.8 Write property tests for advanced code generation
    - **Property 9: Type casts translate to 'as' operator**
    - **Validates: Requirements 27.5**
    - **Property 10: Sizeof translates to std::mem functions**
    - **Validates: Requirements 28.6**
    - **Property 11: Increment/decrement operators translate with correct semantics**
    - **Validates: Requirements 29.8, 29.9**
    - **Property 12: Typedef translates to type alias**
    - **Validates: Requirements 31.9**
    - **Property 13: C-style enums translate to Rust enums with discriminants**
    - **Validates: Requirements 32.8**
    - **Property 14: NULL translates to Option types**
    - **Validates: Requirements 34.4, 34.5**
    - **Property 15: Struct initializers translate to Rust struct literals**
    - **Validates: Requirements 39.6**
    - **Property 16: Struct methods translate to impl blocks**
    - **Validates: Requirements 21.9**
    - **Property 18: For loops translate appropriately**
    - **Validates: Requirements 38.4, 38.5, 38.7**
    - **Property 19: Switch statements translate to match expressions**
    - **Validates: Requirements 45.7**
    - **Property 20: Error handling syntax translates correctly**
    - **Validates: Requirements 46.8, 46.9, 46.10**
    - **Property 24: Explicit generic parameters translate correctly**
    - **Validates: Requirements 38.18, 38.19, 38.20, 38.21**
  
  - [x] 16.9 Write unit tests for advanced code generation
    - Test NULL and Option translation
    - Test struct initializer translation
    - Test switch statement translation
    - Test error handling translation
    - _Requirements: 28.4-28.8, 33.6-33.9, 45.7-45.10, 46.8-46.13_

- [-] 17. Implement nested functions (closures)
  - [x]17.1 Add nested function parsing
    - Extend Parser to recognize function declarations within function bodies
    - Create NestedFunction AST node with signature and body
    - Support multiple nested functions in the same enclosing function
    - Verify nested functions use same syntax as top-level functions
    - _Requirements: 59.1, 59.2, 59.3_
  
  - [x]17.2 Implement capture analysis in semantic analyzer
    - Track variables in scope when nested function is declared
    - Build capture list for each nested function
    - Determine which variables are captured (used in nested function body)
    - Classify captures as immutable (read-only) or mutable (modified)
    - Verify nested functions only access variables defined before declaration
    - Verify variables defined after nested function are not accessible
    - _Requirements: 59.4, 59.5, 59.6, 59.7, 59.24_
  
  - [x]17.3 Add nested function type checking
    - Verify nested functions can be assigned to variables
    - Verify nested functions can be passed as function parameters
    - Verify nested functions can be returned from functions
    - Support function pointer types for parameters accepting nested functions
    - Verify type compatibility when passing nested functions as arguments
    - Verify multiple nested functions can capture same variables
    - _Requirements: 59.8, 59.9, 59.10, 59.22, 59.23, 59.25_
  
  - [x]17.4 Implement nested function code generation
    - Translate nested functions to Rust closures
    - Generate closure with appropriate trait (Fn, FnMut, FnOnce)
    - Translate immutable captures to Fn closures
    - Translate mutable captures to FnMut closures
    - Translate move semantics to FnOnce closures when appropriate
    - Infer closure trait based on capture analysis
    - Handle nested functions with no captures
    - _Requirements: 59.11, 59.12, 59.13, 59.14, 59.15, 59.16_
  
  - [x] 17.5 Add validation rules for nested functions
    - Verify nested functions cannot be declared static
    - Verify nested functions cannot contain nested functions (no multi-level nesting)
    - Ensure proper type compatibility for function pointers
    - _Requirements: 59.18, 59.19, 59.17_
  
  - [x] 17.6 Write property test for nested function translation
    - **Property 35: Nested functions translate to Rust closures**
    - **Validates: Requirements 59.11, 59.12, 59.13**
  
  - [x]17.7 Write unit tests for nested functions
    - Test parsing of nested functions
    - Test capture analysis (immutable and mutable)
    - Test scoping rules (before/after declaration)
    - Test passing nested functions as parameters
    - Test multiple nested functions sharing captures
    - Test code generation to Fn, FnMut, FnOnce
    - Test error cases (static nested functions, multi-level nesting)
    - _Requirements: 59.1-59.25_

- [ ] 18. Checkpoint - Ensure advanced features work correctly
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 19. Implement VTable to trait translation
  - [ ] 19.1 Detect VTable struct patterns
    - Identify typedef struct with function pointer fields
    - Recognize VTable suffix naming convention
    - Detect void *self or typed self pointers in function signatures
    - _Requirements: 17.1-17.5_
  
  - [ ] 19.2 Generate Rust trait definitions from VTables
    - Translate VTable structs to Rust trait definitions
    - Translate function pointer fields to trait method signatures
    - Translate void *self to &self or &mut self based on semantics
    - _Requirements: 17.6, 17.7, 17.8_
  
  - [ ] 19.3 Generate trait implementations
    - Detect structs using VTable fields
    - Generate trait implementations for those structs
    - Handle trait object usage (dyn Trait)
    - _Requirements: 17.9, 17.10_
  
  - [ ] 19.4 Write property test for VTable translation
    - **Property 17: VTable structs translate to traits**
    - **Validates: Requirements 22.6**
  
  - [ ] 19.5 Write unit tests for VTable translation
    - Test VTable detection
    - Test trait generation
    - Test trait implementation generation
    - _Requirements: 17.1-17.14_

- [ ] 20. Implement module system and visibility
  - [ ] 20.1 Add namespace parsing and code generation
    - Parse namespace declarations (namespace name { ... })
    - Support nested namespaces
    - Translate namespaces to Rust mod blocks
    - Merge multiple namespace blocks with same name
    - _Requirements: 42.1-42.7_
  
  - [ ] 20.2 Add #import and #export directive parsing and code generation
    - Parse #import directives for module imports (private use statements)
    - Parse #export directives for module re-exports (pub use statements)
    - Translate #import to Rust use statements
    - Translate #export to Rust pub use statements
    - Support importing Rust std library modules
    - _Requirements: 50.1, 50.2, 50.3, 50.4, 50.5, 50.6_
  
  - [ ] 20.3 Implement visibility rules
    - Recognize underscore-prefixed identifiers as private
    - Mark struct fields as pub or private based on naming
    - Mark functions as pub or private based on static keyword and naming
    - _Requirements: 43.1-43.6_
  
  - [ ] 20.4 Write property test for module translation
    - **Property 21: Module directives translate correctly**
    - **Validates: Requirements 47.3, 48.5**
  
  - [ ] 20.5 Write unit tests for module system
    - Test namespace parsing and generation
    - Test #import directive handling (private imports)
    - Test #export directive handling (public re-exports)
    - Test visibility rules
    - _Requirements: 50.1-50.17, 51.1-51.8, 52.1-52.6_

- [ ] 21. Implement main() function validation
  - [ ] 21.1 Add main() function detection
    - Parse main() function with C-like syntax
    - Support main() with no parameters (void)
    - Support main() with argc/argv parameters
    - _Requirements: 10.1, 10.5, 10.6_
  
  - [ ] 21.2 Validate main() function
    - Verify exactly one main() function exists
    - Report error if no main() found
    - Report error if multiple main() found
    - _Requirements: 10.2, 10.3, 10.4_
  
  - [ ] 21.3 Generate Rust main() function
    - Translate main() to Rust fn main()
    - Translate argc/argv to std::env functions
    - _Requirements: 10.7, 10.8_
  
  - [ ] 21.4 Write unit tests for main() validation
    - Test main() detection
    - Test error cases (no main, multiple main)
    - Test main() translation
    - Commit with message: "feat(task-21): implement main() function validation"
    - _Requirements: 10.1-10.8_

- [ ] 22. Verify documentation comment preservation
  - [ ] 22.1 Verify doc comment preservation in Code Generator
    - Ensure /// and /** ... */ outer doc comments are preserved in generated Rust
    - Ensure //! and /*! ... */ inner doc comments are preserved in generated Rust
    - Ensure doc comments maintain their position relative to code elements
    - _Requirements: 53.1, 53.2, 53.3, 53.10_
  
  - [ ] 22.2 Write unit tests for doc comment preservation
    - Test that doc comments are preserved during transpilation
    - Test that doc comment content is unchanged
    - Test that doc comment positions are correct
    - Commit with message: "test(task-22): verify doc comment preservation"
    - _Requirements: 53.1-53.10_
    - _Note: crustydoc tool implementation is in Phase 2_

- [ ] 23. Implement additional language features
    - Ensure /// and /** ... */ outer doc comments are preserved in generated Rust
    - Ensure //! and /*! ... */ inner doc comments are preserved in generated Rust
    - Ensure doc comments maintain their position relative to code elements
    - _Requirements: 53.1, 53.2, 53.3, 53.10_
  
  - [ ] 24.2 Write unit tests for doc comment preservation
    - Test that doc comments are preserved during transpilation
    - Test that doc comment content is unchanged
    - Test that doc comment positions are correct
    - _Requirements: 53.1-53.10_

- [ ] 25. Implement crustydoc wrapper tool
  - [ ] 25.1 Create crustydoc CLI
    - Create separate binary target for crustydoc
    - Parse command-line arguments (input file, --output, --open, -D, --document-private-items)
    - Support passing through additional rustdoc options after --
    - _Requirements: 54.1, 54.6, 54.12_
  
  - [ ] 25.2 Implement transpile-and-document workflow
    - Transpile Crusty source file to Rust
    - Invoke rustdoc on generated Rust code
    - Pass through all rustdoc command-line options
    - Capture rustdoc output and errors
    - _Requirements: 54.4, 54.5, 54.6, 54.9, 54.10_
  
  - [ ] 25.3 Implement error mapping
    - Parse rustdoc error messages
    - Map Rust source locations back to Crusty source locations
    - Report errors with Crusty file paths and line numbers
    - Preserve rustdoc error messages and suggestions
    - _Requirements: 54.11, 55.4_
  
  - [ ] 25.4 Add Cargo integration support
    - Support --manifest-path option for Cargo projects
    - Coordinate with build.rs for multi-file transpilation
    - Invoke cargo doc with appropriate options
    - _Requirements: 54.7, 54.8_
  
  - [ ] 25.5 Implement documentation validation
    - Support rustdoc's -D missing-docs flag
    - Support --document-private-items flag
    - Report documentation coverage from rustdoc output
    - _Requirements: 55.1, 55.2, 55.3, 55.5_
  
  - [ ] 25.6 Write unit tests for crustydoc
    - Test that doc comments are preserved during transpilation
    - Test that doc comment content is unchanged
    - Test that doc comment positions are correct
    - Commit with message: "test(task-22): verify doc comment preservation"
    - _Requirements: 53.1-53.10_
    - _Note: crustydoc tool implementation is in Phase 2_

- [ ] 23. Implement additional language features
  - [ ] 23.1 Add extern block support (Requirement 44)
    - Parse extern blocks with optional ABI specification (extern { ... }, extern "C" { ... }, extern "Rust" { ... })
    - Support all Rust ABI strings (extern "C", extern "cdecl", extern "stdcall", extern "fastcall", extern "system", extern "Rust", etc.)
    - Parse Crusty-style function declarations inside extern blocks
    - Support __rust__{ } blocks within extern blocks for embedding raw Rust code
    - Validate nesting rules: reject extern within extern, __rust__ within __rust__, extern within __rust__ within extern
    - Allow extern within __rust__ at module level (handled by rustc)
    - Validate FFI-compatible types in extern functions
    - Translate extern blocks to Rust with same ABI specification
    - Translate Crusty function declarations to Rust function declarations inside extern blocks
    - Emit __rust__{ } block contents directly as Rust code within extern blocks
    - _Requirements: 44.1-44.20_
  
  - [ ] 23.2 Add inline assembly support
    - Parse __asm__ macro syntax with double-underscore naming (no ! suffix)
    - Require __asm__ within unsafe blocks
    - Translate __asm__ to Rust asm! syntax in code generation (add !)
    - _Requirements: 36.1-36.7_
  
  - [ ] 23.3 Add __rust__ macro support
    - Parse __rust__ macro with double-underscore naming for embedding raw Rust code (no ! suffix)
    - Support __rust__ in expression, statement, and type contexts
    - Extract and emit __rust__ contents directly as Rust code
    - Handle nested braces within __rust__ blocks
    - _Requirements: 43.1-43.13_
  
  - [ ] 23.4 Add conditional compilation support
    - Parse #ifdef, #ifndef, #endif directives
    - Translate to Rust cfg attributes
    - Support nested conditional blocks
    - _Requirements: 41.4, 41.5, 41.6, 41.7, 41.8_
  
  - [ ] 23.5 Write unit tests for additional features
    - Test extern "C" parsing and generation
    - Test __asm__ parsing and generation (translates to Rust asm!)
    - Test __rust__ macro handling (no ! suffix in Crusty)
    - Test conditional compilation
    - Commit with message: "feat(task-23): implement additional language features"
    - _Requirements: 35.1-35.9, 36.1-36.7, 43.1-43.13, 41.4-41.8_

- [ ] 24. Implement error message improvements
  - [ ] 24.1 Integrate codespan-reporting
    - Use codespan-reporting crate for beautiful error messages
    - Display source code snippets with error locations
    - Highlight error positions with carets
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_
  
  - [ ] 24.2 Improve error messages for unsupported features
    - Provide clear explanations for why features are unsupported
    - Suggest alternative approaches
    - Reference documentation for unsupported features
    - _Requirements: 5.3, 12.1, 12.2, 12.3, 12.4, 12.5, 12.6, 12.7_
  
  - [ ] 24.3 Implement error recovery in parser
    - Add synchronization points at statement boundaries
    - Implement panic mode recovery
    - Continue parsing after errors to report multiple errors
    - _Requirements: 5.4_
  
  - [ ] 24.4 Write property test for multiple error reporting
    - **Property 3: Multiple errors are all reported**
    - **Validates: Requirements 10.4**
  
  - [ ] 24.5 Write unit tests for error messages
    - Test error message formatting
    - Test error recovery
    - Test multiple error reporting
    - Commit with message: "feat(task-24): implement error message improvements"
    - _Requirements: 5.1-5.5_

- [ ] 25. Implement pointer arithmetic and safety checks
  - [ ] 25.1 Add pointer arithmetic parsing
    - Parse pointer + offset, pointer - offset operations
    - Parse pointer - pointer operations
    - _Requirements: 24.1, 24.2_
  
  - [ ] 25.2 Implement pointer safety analysis
    - Enforce Rust's pointer arithmetic safety rules
    - Require unsafe context for raw pointer arithmetic
    - Translate safe pointer operations to slice indexing
    - Translate unsafe pointer arithmetic to unsafe blocks with offset
    - _Requirements: 24.3, 24.4, 24.5, 24.6, 24.7_
  
  - [ ] 25.3 Write unit tests for pointer arithmetic
    - Test pointer arithmetic parsing
    - Test safety analysis
    - Test code generation for safe and unsafe operations
    - Commit with message: "feat(task-25): implement pointer arithmetic"
    - _Requirements: 24.1-24.7_

- [ ] 26. Implement lifetime inference
  - [ ] 26.1 Add lifetime inference analysis
    - Infer lifetime relationships from reference parameters
    - Determine which parameters return value derives from
    - Handle multiple reference parameters
    - _Requirements: 30.6, 30.7, 30.8, 30.9_
  
  - [ ] 26.2 Generate explicit lifetime annotations
    - Add Rust lifetime annotations where necessary
    - Use inferred relationships to determine lifetime parameters
    - _Requirements: 30.12_
  
  - [ ] 26.3 Validate borrowing rules
    - Enforce one mutable reference OR multiple immutable references
    - Verify mutable references only from mutable variables
    - Verify references don't outlive referents
    - _Requirements: 30.13, 30.14, 30.15, 30.16_
  
  - [ ] 26.4 Write unit tests for lifetime inference
    - Test lifetime inference from function signatures
    - Test lifetime annotation generation
    - Test borrowing rule validation
    - Commit with message: "feat(task-26): implement lifetime inference"
    - _Requirements: 30.6-30.16_

- [ ] 27. Checkpoint - Ensure all features are implemented
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 28. Implement comprehensive integration tests
  - [ ] 28.1 Write end-to-end integration tests
    - Test complete compilation pipeline (Crusty → Rust → binary)
    - Test multi-file projects with build.rs
    - Test CLI with various options
    - Test error handling across entire pipeline
    - _Requirements: All_
  
  - [ ] 28.2 Write property-based integration tests
    - Generate random valid Crusty programs
    - Verify they compile successfully
    - Run with 100+ iterations
    - _Requirements: All_
  
  - [ ] 28.3 Create test suite with example programs
    - Create hello world example
    - Create struct and method example
    - Create error handling example
    - Create multi-file project example with build.rs
    - Create FFI example with extern "C"
    - Verify all examples compile and run correctly
    - Commit with message: "test(task-28): add comprehensive integration tests"
    - _Requirements: All_

- [ ] 29. Performance optimization
  - [ ] 29.1 Profile transpiler performance
    - Identify performance bottlenecks
    - Measure parsing, semantic analysis, and code generation times
    - _Requirements: All (non-functional)_
  
  - [ ] 29.2 Implement performance improvements
    - Add incremental parsing for large files
    - Implement parallel compilation for multi-file projects
    - Add AST caching to avoid re-parsing unchanged files
    - Implement lazy code generation
    - _Requirements: All (non-functional)_
  
  - [ ] 29.3 Write performance benchmarks
    - Benchmark parsing performance
    - Benchmark semantic analysis performance
    - Benchmark code generation performance
    - Benchmark end-to-end compilation
    - Commit with message: "perf(task-29): optimize transpiler performance"
    - _Requirements: All (non-functional)_

- [ ] 30. Documentation and polish
  - [ ] 30.1 Write user documentation
    - Create README with installation instructions
    - Document command-line options
    - Provide usage examples
    - Document supported and unsupported C features
    - Create migration guide from C to Crusty
    - Document build.rs integration patterns
    - _Requirements: 12.1-12.7_
  
  - [ ] 30.2 Write developer documentation
    - Document transpiler architecture
    - Document AST structure
    - Document adding new language features
    - Document testing strategy
    - _Requirements: All (documentation)_
  
  - [ ] 30.3 Create language specification
    - Document Crusty syntax formally
    - Document translation rules to Rust
    - Document type system
    - Document module system
    - _Requirements: All (specification)_
  
  - [ ] 30.4 Polish error messages
    - Review all error messages for clarity
    - Add helpful suggestions where possible
    - Ensure consistent error message format
    - Commit with message: "docs(task-30): complete documentation and polish"
    - _Requirements: 5.1-5.5_

- [ ] 31. Final validation and testing
  - [ ] 31.1 Run full test suite
    - Run all unit tests
    - Run all property tests with 1000 iterations
    - Run all integration tests
    - Verify 90%+ code coverage
    - _Requirements: All_
  
  - [ ] 31.2 Validate against requirements
    - Verify all Phase 1 requirements are implemented
    - Verify all correctness properties are tested
    - Check for any missing functionality
    - _Requirements: All_
  
  - [ ] 31.3 Perform manual testing
    - Test with real-world C code examples
    - Test edge cases and corner cases
    - Test error handling with invalid inputs
    - Verify generated Rust code compiles with rustc
    - Commit with message: "test(task-31): complete final validation"
    - _Requirements: All_

- [ ] 32. Validate Rust ecosystem integration
  - [ ] 32.1 Test external crate usage
    - Create test project that uses external Rust crates (e.g., serde, tokio)
    - Write Crusty code that imports and uses external types
    - Verify Crusty can call external functions
    - Verify type compatibility with external crates
    - _Requirements: 40.1, 40.2, 40.5, 40.6_
  
  - [ ] 32.2 Test Crusty crate publishing
    - Create Crusty library project with public API
    - Build library and verify .rlib generation
    - Create separate Rust project that depends on Crusty library
    - Verify Rust code can import and use Crusty library types
    - Verify Rust code can call Crusty library functions
    - Test API compatibility and type safety
    - _Requirements: 40.3, 40.4, 40.7, 40.8, 40.9, 40.10, 40.11_
  
  - [ ] 32.3 Test procedural macro usage
    - Use Rust procedural macros in Crusty code (e.g., derive macros)
    - Verify macro expansion works correctly
    - Test custom derive macros with Crusty structs
    - _Requirements: 40.12, 40.13_
  
  - [ ] 32.4 Validate performance parity
    - Create equivalent programs in Crusty and Rust
    - Benchmark execution time for both versions
    - Verify no runtime overhead from transpilation
    - Verify generated code is optimized equivalently
    - _Requirements: 40.14, 40.15_
  
  - [ ] 32.5 Write integration tests for ecosystem
    - Test importing std library modules
    - Test using external crate dependencies
    - Test publishing and consuming Crusty crates
    - Test interoperability with Rust code
    - Commit with message: "test(task-32): validate Rust ecosystem integration"
    - _Requirements: 40.1-40.15_

- [ ] 33. Final checkpoint - Phase 1 completion
  - Ensure all tests pass, ask the user if questions arise
  - Verify documentation is complete
  - Prepare Phase 1 completion report
  - _Requirements: All_

## Summary

**Phase 1 Scope**: Core Crusty → Rust transpilation

**Total Tasks**: 33 major tasks
**Completed**: 17 tasks (1-17)
**Remaining**: 16 tasks (18-33)

**Estimated Time Remaining**: 80-100 hours

**Critical Path**:
1. Tasks 17.3-17.6: Complete nested functions (10-15 hours)
2. Task 18: Checkpoint validation (2 hours)
3. Tasks 19-20: VTable and module system (15-20 hours)
4. Tasks 21-26: Core features completion (25-30 hours)
5. Tasks 27-33: Integration, testing, documentation (30-35 hours)

**Key Milestones**:
- Milestone 1: Nested functions complete (Task 17)
- Milestone 2: Module system working (Task 20)
- Milestone 3: All core features implemented (Task 26)
- Milestone 4: All tests passing (Task 31)
- Milestone 5: Phase 1 complete (Task 33)

**Out of Scope for Phase 1**:
- crustydoc tool → Phase 2
- crustyfmt tool → Phase 2
- Rust → Crusty transpilation → Phase 3
- Round-trip validation → Phase 3

**Dependencies**:
- Phase 2 depends on Phase 1 completion
- Phase 3 depends on Phase 1 and Phase 2 completion

## Notes

- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties with minimum 100 iterations
- Unit tests validate specific examples and edge cases
- The implementation follows a bottom-up approach: infrastructure → parsing → analysis → generation → advanced features
- The transpiler uses Rust's standard library directly without wrappers
- All generated Rust code must be valid and compile with rustc
- Documentation comment preservation is verified in Phase 1, but crustydoc tool is Phase 2
- All Phase 1 tasks focus on one-way transpilation (Crusty → Rust)

---

## See Also

- [SYNTAX_PHILOSOPHY.md](SYNTAX_PHILOSOPHY.md) - Core principle: syntax-only transpilation
- [requirements.md](requirements.md) - Detailed feature requirements
- [design.md](design.md) - Architecture and component design
- [../PHASE_REORGANIZATION.md](../PHASE_REORGANIZATION.md) - Phase structure explanation
- [../crusty-compiler-phase2/](../crusty-compiler-phase2/) - Phase 2 specs (tooling)
- [../crusty-compiler-phase3/](../crusty-compiler-phase3/) - Phase 3 specs (bidirectional)
- [README.md](../../../README.md) - Project overview and quick start guide
- [CONTRIBUTING.md](../../../CONTRIBUTING.md) - How to contribute

