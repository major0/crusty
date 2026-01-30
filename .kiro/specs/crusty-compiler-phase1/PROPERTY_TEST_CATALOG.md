# Crusty Compiler Property-Based Test Catalog

**Date**: 2026-01-30
**Status**: Complete catalog of all 34 correctness properties
**Related**: [design.md](design.md) - Architecture and design document

## Overview

This document catalogs all 34 correctness properties that the Crusty compiler must satisfy. Each property is tested using property-based testing (PBT) with the `proptest` crate, running a minimum of 100 iterations per test.

Property-based testing validates universal properties that must hold for **all** inputs, not just specific examples. This approach catches edge cases that traditional unit tests might miss.

**See Also**: [design.md](design.md) for detailed architecture and [ERROR_CATALOG.md](ERROR_CATALOG.md) for error messages.

---

## Table of Contents

1. [Core Parsing Properties](#core-parsing-properties) (Properties 1-3)
2. [Code Generation Properties](#code-generation-properties) (Properties 4-6)
3. [Translation Properties](#translation-properties) (Properties 7-24)
4. [Bidirectional Transpilation Properties](#bidirectional-transpilation-properties) (Properties 25-27)
5. [Type System Properties](#type-system-properties) (Property 28)
6. [File I/O Properties](#file-io-properties) (Property 29)
7. [Integration Properties](#integration-properties) (Properties 30-31)
8. [Semantic Validation Properties](#semantic-validation-properties) (Properties 32)
9. [Formatting Properties](#formatting-properties) (Properties 33-34)
10. [Testing Strategy](#testing-strategy)
11. [Implementation Status](#implementation-status)

---

## Core Parsing Properties

### Property 1: Valid Crusty programs parse successfully

**Statement**: *For any* syntactically valid Crusty source file, the Parser should successfully parse it into a complete AST without errors.

**Validates**: Requirements 6.1

**Test Strategy**:
- Generate random valid Crusty programs using grammar-based generation
- Parse each program
- Assert no parse errors occur
- Assert AST is complete and well-formed

**Example Test**:
```rust
#[test]
fn test_property_1_valid_programs_parse() {
    proptest!(|(program: ValidCrustyProgram)| {
        let result = parse(&program.source);
        assert!(result.is_ok());
        assert!(result.unwrap().is_complete());
    });
}
```

**Status**: ✅ Implemented

---

### Property 2: Invalid syntax produces error reports with location

**Statement**: *For any* Crusty source file with syntax errors, the Parser should report each error with its line number, column number, and a descriptive error message.

**Validates**: Requirements 6.2, 10.1

**Test Strategy**:
- Generate programs with intentional syntax errors
- Parse each program
- Assert parse errors are reported
- Assert each error includes line and column numbers
- Assert error messages are descriptive

**Example Test**:
```rust
#[test]
fn test_property_2_invalid_syntax_reports_errors() {
    proptest!(|(program: InvalidCrustyProgram)| {
        let result = parse(&program.source);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        for error in errors {
            assert!(error.has_location());
            assert!(!error.message.is_empty());
        }
    });
}
```

**Status**: ✅ Implemented

---

### Property 3: Multiple errors are all reported

**Statement**: *For any* Crusty source file containing multiple syntax or semantic errors, the compiler should report all errors found, not just the first one.

**Validates**: Requirements 10.4

**Test Strategy**:
- Generate programs with multiple intentional errors
- Compile each program
- Assert all errors are reported
- Assert error count matches expected count

**Example Test**:
```rust
#[test]
fn test_property_3_multiple_errors_reported() {
    proptest!(|(program: MultiErrorProgram)| {
        let result = compile(&program.source);
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), program.expected_error_count);
    });
}
```

**Status**: ⏳ Partially Implemented (semantic errors only)

---

## Code Generation Properties

### Property 4: Generated Rust code is syntactically valid

**Statement**: *For any* valid AST, the Code_Generator should produce Rust source code that can be successfully parsed by the Rust parser (syn or rustc) without syntax errors.

**Validates**: Requirements 8.1

**Test Strategy**:
- Generate random valid ASTs
- Generate Rust code from each AST
- Parse generated Rust code with syn
- Assert no syntax errors occur

**Example Test**:
```rust
#[test]
fn test_property_4_generated_rust_is_valid() {
    proptest!(|(ast: ValidAst)| {
        let rust_code = generate_rust(&ast);
        let result = syn::parse_file(&rust_code);
        assert!(result.is_ok());
    });
}
```

**Status**: ✅ Implemented

---

### Property 5: Generated Rust code follows formatting conventions

**Statement**: *For any* generated Rust source code, running rustfmt on it should produce no changes, indicating it already follows Rust style conventions.

**Validates**: Requirements 8.16

**Test Strategy**:
- Generate Rust code from random ASTs
- Run rustfmt on generated code
- Assert no changes are made

**Example Test**:
```rust
#[test]
fn test_property_5_rust_follows_conventions() {
    proptest!(|(ast: ValidAst)| {
        let rust_code = generate_rust(&ast);
        let formatted = run_rustfmt(&rust_code);
        assert_eq!(rust_code, formatted);
    });
}
```

**Status**: ✅ Implemented

---

### Property 6: Transparent syntax preservation

**Statement**: *For any* AST node containing Rust-compatible syntax (tuples, array literals, macro invocations, attributes, ranges), the generated Rust code should preserve that syntax exactly as it appears in the AST.

**Validates**: Requirements 19.7, 20.4, 23.6, 25.8, 26.8

**Test Strategy**:
- Generate ASTs with Rust-compatible syntax
- Generate Rust code
- Assert syntax is preserved exactly

**Example Test**:
```rust
#[test]
fn test_property_6_transparent_syntax_preservation() {
    proptest!(|(ast: RustCompatibleAst)| {
        let rust_code = generate_rust(&ast);
        assert!(rust_code.contains(&ast.expected_syntax));
    });
}
```

**Status**: ✅ Implemented

---

## Translation Properties

### Property 7: Variable declarations translate correctly

**Statement**: *For any* variable declaration in the AST (const, var, let, static), the generated Rust code should use the corresponding Rust keyword (const, let mut, let, static) with correct semantics.

**Validates**: Requirements 35.7, 35.8, 35.9

**Test Strategy**:
- Generate ASTs with various variable declarations
- Generate Rust code
- Assert correct Rust keywords are used
- Assert semantics are preserved

**Example Test**:
```rust
#[test]
fn test_property_7_variable_declarations_translate() {
    proptest!(|(decl: VariableDeclaration)| {
        let rust_code = generate_rust(&decl.to_ast());
        match decl.kind {
            VarKind::Const => assert!(rust_code.contains("const")),
            VarKind::Var => assert!(rust_code.contains("let mut")),
            VarKind::Let => assert!(rust_code.contains("let")),
            VarKind::Static => assert!(rust_code.contains("static")),
        }
    });
}
```

**Status**: ✅ Implemented

---

### Property 8: Reference syntax translates correctly

**Statement**: *For any* reference expression in the AST (& for immutable, var & or &mut for mutable), the generated Rust code should use the corresponding Rust reference syntax (& or &mut).

**Validates**: Requirements 36.10, 36.11

**Status**: ✅ Implemented

---

### Property 9: Type casts translate to 'as' operator

**Statement**: *For any* C-style cast expression ((type)expr) in the AST, the generated Rust code should use the 'as' operator (expr as type).

**Validates**: Requirements 27.5

**Status**: ✅ Implemented

---

### Property 10: Sizeof translates to std::mem functions

**Statement**: *For any* sizeof expression in the AST, the generated Rust code should use std::mem::size_of<T>() for types or std::mem::size_of_val(&expr) for expressions.

**Validates**: Requirements 28.6

**Status**: ✅ Implemented

---

### Property 11: Increment/decrement operators translate with correct semantics

**Statement**: *For any* prefix increment (++i) in the AST, the generated Rust code should evaluate to the incremented value; for any postfix increment (i++), the generated Rust code should evaluate to the original value before incrementing.

**Validates**: Requirements 29.10, 29.11

**Status**: ✅ Implemented

---

### Property 12: Typedef translates to type alias

**Statement**: *For any* typedef declaration in the AST, the generated Rust code should create a corresponding type alias using the 'type' keyword.

**Validates**: Requirements 31.9

**Status**: ✅ Implemented

---

### Property 13: C-style enums translate to Rust enums with discriminants

**Statement**: *For any* C-style enum declaration in the AST, the generated Rust code should create a Rust enum with explicit integer discriminants matching the C-style values.

**Validates**: Requirements 32.8

**Status**: ✅ Implemented

---

### Property 14: NULL translates to Option types

**Statement**: *For any* NULL literal in the AST, the generated Rust code should use @Option.None (translating to Option::None); for any nullable pointer type, the generated Rust code should use Option<&T> or Option<&mut T>.

**Validates**: Requirements 34.4, 34.5

**Status**: ✅ Implemented

---

### Property 15: Struct initializers translate to Rust struct literals

**Statement**: *For any* struct initialization with designated initializers (.field = value) in the AST, the generated Rust code should use Rust struct literal syntax (StructName { field: value }).

**Validates**: Requirements 39.6

**Status**: ✅ Implemented

---

### Property 16: Struct methods translate to impl blocks

**Statement**: *For any* struct with methods in the AST, the generated Rust code should create a corresponding impl block containing all methods. Static method calls using @Type.method() syntax should translate to Rust Type::method() syntax.

**Validates**: Requirements 21.9

**Status**: ✅ Implemented

---

### Property 17: VTable structs translate to traits

**Statement**: *For any* struct following the vtable pattern (function pointers with self parameter) in the AST, the generated Rust code should create a corresponding trait definition.

**Validates**: Requirements 22.6

**Status**: ⏳ Not Yet Implemented

---

### Property 18: For loops translate appropriately

**Statement**: *For any* C-style for loop in the AST matching the pattern for(i=start; i<end; i++), the generated Rust code should use range syntax (for i in start..end); for multi-variable for loops, the generated Rust code should use a scoped while loop.

**Validates**: Requirements 38.4, 38.5, 38.7

**Status**: ✅ Implemented

---

### Property 19: Switch statements translate to match expressions

**Statement**: *For any* switch statement in the AST, the generated Rust code should create a corresponding match expression with all cases and default branch.

**Validates**: Requirements 45.7

**Status**: ✅ Implemented

---

### Property 20: Error handling syntax translates correctly

**Statement**: *For any* fallible return type (Type?) in the AST, the generated Rust code should use Result<Type, E>; the expr? operator passes through unchanged to Rust's ? operator.

**Validates**: Requirements 46.8, 46.10

**Status**: ✅ Implemented

---

### Property 21: Module directives translate correctly

**Statement**: *For any* #import directive in the AST, the generated Rust code should create a corresponding private use statement; for any #export directive, the generated Rust code should create a corresponding pub use statement; for any namespace declaration, the generated Rust code should create a corresponding mod block.

**Validates**: Requirements 50.3, 50.4, 50.5, 50.6, 51.5

**Status**: ✅ Implemented

---

### Property 22: #define macros translate to macro_rules!

**Statement**: *For any* #define macro definition in the AST, the generated Rust code should create a corresponding macro_rules! definition with parameters translated to pattern variables and the body wrapped in appropriate Rust macro syntax.

**Validates**: Requirements 24.7, 24.8, 24.9

**Status**: ✅ Implemented

---

### Property 23: Label syntax translates correctly

**Statement**: *For any* labeled loop (.label: loop), break statement (break label), or continue statement (continue label) in the AST, the generated Rust code should use Rust's label syntax ('label:, break 'label, continue 'label). Note: The dot is a prefix for label declarations only, not part of the label name.

**Validates**: Requirements 6.13, 6.14, 6.15

**Status**: ✅ Implemented

---

### Property 24: Explicit generic parameters translate correctly

**Statement**: *For any* explicit generic type parameter specification using parentheses/brackets syntax in the AST, the generated Rust code should use Rust's turbofish syntax with angle brackets. Nested generics should correctly alternate between parentheses and brackets in Crusty and translate to nested angle brackets in Rust.

**Validates**: Requirements 38.18, 38.19, 38.20, 38.21

**Status**: ✅ Implemented

---

## Bidirectional Transpilation Properties

### Property 25: Rust to Crusty translation preserves structure

**Statement**: *For any* Rust source file, parsing it and generating Crusty code should produce valid Crusty syntax that preserves the program structure (functions become functions, match becomes switch, etc.).

**Validates**: Requirements 47.5, 47.8

**Test Strategy**:
- Parse random Rust programs
- Generate Crusty code
- Assert Crusty code is valid
- Assert structure is preserved

**Status**: ⏳ Not Yet Implemented

---

### Property 26: Round-trip transpilation preserves semantics (CRITICAL)

**Statement**: *For any* valid Crusty source file, transpiling to Rust and then back to Crusty should produce a program that is semantically equivalent to the original (same AST structure after normalization).

**Validates**: Requirements 54.20

**Test Strategy**:
- Generate random Crusty programs
- Transpile to Rust
- Transpile back to Crusty
- Parse both original and round-trip Crusty
- Assert ASTs are semantically equivalent

**Status**: ⏳ Not Yet Implemented

---

### Property 27: Pretty-print then parse is identity (CRITICAL)

**Statement**: *For any* valid AST, pretty-printing it to Crusty source code and then parsing that source code should produce an AST that is equivalent to the original (modulo formatting differences).

**Validates**: Requirements 16.1, 16.2

**Test Strategy**:
- Generate random valid ASTs
- Pretty-print to Crusty source
- Parse the pretty-printed source
- Assert ASTs are equivalent

**Status**: ✅ Implemented

---

## Type System Properties

### Property 28: Type checking matches Rust semantics

**Statement**: *For any* type operation in a valid program (assignment, function call, operator application), the Semantic_Analyzer should accept it if and only if Rust's type system would accept it.

**Validates**: Requirements 18.9

**Test Strategy**:
- Generate random type-correct and type-incorrect programs
- Run semantic analysis
- Compare results with Rust's type checker
- Assert agreement on all type operations

**Status**: ⏳ Partially Implemented

---

## File I/O Properties

### Property 29: Valid file paths are read successfully

**Statement**: *For any* valid file path provided to crustyc, the file contents should be successfully read into memory.

**Validates**: Requirements 11.1

**Test Strategy**:
- Generate random valid file paths
- Create files at those paths
- Attempt to read with crustyc
- Assert successful reads

**Status**: ✅ Implemented

---

## Integration Properties

### Property 30: Example directory builds successfully

**Statement**: *For any* valid example project in the example/ directory, running `cargo build` should succeed without errors, and running `cargo run` should execute the example binary successfully.

**Validates**: Requirements 6.1-6.34

**Test Strategy**:
- Run `cargo build` in example directory
- Assert build succeeds
- Run `cargo run`
- Assert execution succeeds

**Status**: ✅ Implemented (CI/CD)

---

### Property 31: Rust ecosystem integration works correctly

**Statement**: *For any* Crusty project using external Rust crates, the transpiled code should compile and link correctly, with full type compatibility and API access to external types and functions.

**Validates**: Requirements 40.1-40.15

**Test Strategy**:
- Create Crusty programs using external crates
- Transpile to Rust
- Compile with rustc
- Assert successful compilation and linking

**Status**: ⏳ Not Yet Implemented

---

## Semantic Validation Properties

### Property 32: Function names with double-underscore pattern are rejected

**Statement**: *For any* function definition with both leading AND trailing double-underscores (e.g., `void __helper__()`), the Semantic_Analyzer should report an error indicating that this pattern is reserved for macros.

**Validates**: Requirements 25.10, 25.11

**Test Strategy**:
- Generate functions with double-underscore names
- Run semantic analysis
- Assert error is reported
- Assert error message mentions macro reservation

**Status**: ✅ Implemented

---

## Formatting Properties

### Property 33: crustyfmt preserves semantic meaning

**Statement**: *For any* valid Crusty source file, formatting it with crustyfmt and then parsing both the original and formatted versions should produce semantically equivalent ASTs (modulo whitespace and formatting differences).

**Validates**: Requirements 56.10

**Test Strategy**:
- Generate random Crusty programs
- Format with crustyfmt
- Parse both original and formatted
- Assert ASTs are semantically equivalent

**Status**: ⏳ Not Yet Implemented (crustyfmt not yet built)

---

### Property 34: crustyfmt is idempotent

**Statement**: *For any* valid Crusty source file, formatting it with crustyfmt multiple times should produce identical output after the first formatting pass.

**Validates**: Requirements 56.1-56.20

**Test Strategy**:
- Generate random Crusty programs
- Format with crustyfmt
- Format again
- Assert outputs are identical

**Status**: ⏳ Not Yet Implemented (crustyfmt not yet built)

---

## Testing Strategy

### Property-Based Testing Configuration

All property tests use the `proptest` crate with the following configuration:

```rust
proptest! {
    #![proptest_config(ProptestConfig {
        cases: 100,  // Minimum 100 iterations per test
        max_shrink_iters: 1000,
        ..ProptestConfig::default()
    })]
    
    #[test]
    fn test_property_name(input: InputType) {
        // Test implementation
    }
}
```

### Test Data Generators

**Valid AST Generator**:
```rust
prop_compose! {
    fn valid_ast()(
        functions in prop::collection::vec(valid_function(), 1..10),
        structs in prop::collection::vec(valid_struct(), 0..5),
    ) -> Ast {
        Ast { functions, structs, ..Default::default() }
    }
}
```

**Valid Crusty Program Generator**:
```rust
prop_compose! {
    fn valid_crusty_program()(ast in valid_ast()) -> ValidCrustyProgram {
        ValidCrustyProgram {
            source: pretty_print(&ast),
            ast,
        }
    }
}
```

**Invalid Syntax Generator**:
```rust
prop_compose! {
    fn invalid_crusty_program()(
        valid in valid_crusty_program(),
        mutation in syntax_mutation(),
    ) -> InvalidCrustyProgram {
        InvalidCrustyProgram {
            source: apply_mutation(&valid.source, mutation),
            expected_errors: mutation.expected_error_count(),
        }
    }
}
```

### Test Execution

**Local Development**:
```bash
# Run all tests including property tests (100 iterations)
cargo test

# Run only property tests
cargo test --test '*_properties'

# Run with more iterations for thorough testing
PROPTEST_CASES=1000 cargo test
```

**CI/CD Pipeline**:
- Standard tests: 100 iterations per property
- Nightly tests: 1000 iterations per property
- All tests must pass before merge

---

## Implementation Status

### Summary

**Total Properties**: 34

**By Status**:
- ✅ **Implemented**: 24 properties (71%)
- ⏳ **Partially Implemented**: 2 properties (6%)
- ⏳ **Not Yet Implemented**: 8 properties (23%)

### Implemented Properties (24)

1. ✅ Property 1: Valid programs parse
2. ✅ Property 2: Invalid syntax reports errors
4. ✅ Property 4: Generated Rust is valid
5. ✅ Property 5: Rust follows conventions
6. ✅ Property 6: Transparent syntax preservation
7. ✅ Property 7: Variable declarations translate
8. ✅ Property 8: Reference syntax translates
9. ✅ Property 9: Type casts translate
10. ✅ Property 10: Sizeof translates
11. ✅ Property 11: Increment/decrement translates
12. ✅ Property 12: Typedef translates
13. ✅ Property 13: Enums translate
14. ✅ Property 14: NULL translates
15. ✅ Property 15: Struct initializers translate
16. ✅ Property 16: Struct methods translate
18. ✅ Property 18: For loops translate
19. ✅ Property 19: Switch statements translate
20. ✅ Property 20: Error handling translates
21. ✅ Property 21: Module directives translate
22. ✅ Property 22: #define macros translate
23. ✅ Property 23: Label syntax translates
24. ✅ Property 24: Generic parameters translate
27. ✅ Property 27: Pretty-print then parse is identity
29. ✅ Property 29: File paths read successfully
30. ✅ Property 30: Example directory builds
32. ✅ Property 32: Double-underscore functions rejected

### Partially Implemented (2)

3. ⏳ Property 3: Multiple errors reported (semantic only, not parse)
28. ⏳ Property 28: Type checking matches Rust (basic types only)

### Not Yet Implemented (8)

17. ⏳ Property 17: VTable structs translate to traits
25. ⏳ Property 25: Rust to Crusty translation preserves structure
26. ⏳ Property 26: Round-trip transpilation preserves semantics (CRITICAL)
31. ⏳ Property 31: Rust ecosystem integration works
33. ⏳ Property 33: crustyfmt preserves semantic meaning
34. ⏳ Property 34: crustyfmt is idempotent

### Critical Properties

**CRITICAL** properties are essential for correctness and must be implemented before Phase 1 completion:

- ✅ Property 27: Pretty-print then parse is identity
- ⏳ Property 26: Round-trip transpilation preserves semantics (NOT YET IMPLEMENTED)

---

## Property Coverage by Requirement

### Requirements with Property Coverage

| Requirement | Properties | Status |
|-------------|-----------|--------|
| 6.1 | 1, 30 | ✅ Complete |
| 6.2 | 2 | ✅ Complete |
| 6.13-6.15 | 23 | ✅ Complete |
| 8.1 | 4 | ✅ Complete |
| 8.16 | 5 | ✅ Complete |
| 10.1 | 2 | ✅ Complete |
| 10.4 | 3 | ⏳ Partial |
| 11.1 | 29 | ✅ Complete |
| 16.1-16.2 | 27 | ✅ Complete |
| 18.9 | 28 | ⏳ Partial |
| 19.7, 20.4, 23.6, 25.8, 26.8 | 6 | ✅ Complete |
| 21.9 | 16 | ✅ Complete |
| 22.6 | 17 | ⏳ Not Implemented |
| 24.7-24.9 | 22 | ✅ Complete |
| 25.10-25.11 | 32 | ✅ Complete |
| 27.5 | 9 | ✅ Complete |
| 28.6 | 10 | ✅ Complete |
| 29.10-29.11 | 11 | ✅ Complete |
| 31.9 | 12 | ✅ Complete |
| 32.8 | 13 | ✅ Complete |
| 34.4-34.5 | 14 | ✅ Complete |
| 35.7-35.9 | 7 | ✅ Complete |
| 36.10-36.11 | 8 | ✅ Complete |
| 38.4-38.7 | 18 | ✅ Complete |
| 38.18-38.21 | 24 | ✅ Complete |
| 39.6 | 15 | ✅ Complete |
| 40.1-40.15 | 31 | ⏳ Not Implemented |
| 45.7 | 19 | ✅ Complete |
| 46.8, 46.10 | 20 | ✅ Complete |
| 47.5, 47.8 | 25 | ⏳ Not Implemented |
| 50.3-50.6, 51.5 | 21 | ✅ Complete |
| 54.20 | 26 | ⏳ Not Implemented |
| 56.1-56.20 | 33, 34 | ⏳ Not Implemented |

---

## See Also

- **[design.md](design.md)** - Architecture and design document
- **[requirements.md](requirements.md)** - Detailed requirements
- **[tasks.md](tasks.md)** - Implementation tasks
- **[ERROR_CATALOG.md](ERROR_CATALOG.md)** - Error message reference
- **[ERROR_HANDLING.md](ERROR_HANDLING.md)** - Error architecture

---

*This catalog documents all 34 correctness properties as of Phase 1. Properties will be expanded as the compiler matures.*
