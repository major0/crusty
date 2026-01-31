# Requirements Document: Crusty Compiler Phase 3

## Introduction

Phase 3 of the Crusty compiler project implements **bidirectional transpilation** between Crusty and Rust. Building on Phase 1's Crusty-to-Rust transpilation and Phase 2's tooling ecosystem, Phase 3 adds the ability to transpile Rust code back to Crusty syntax. This capability serves as the ultimate validation of Crusty's syntax stability and completeness, proving that Crusty is a true syntactic layer over Rust.

## Glossary

- **Bidirectional Transpilation**: The ability to transpile in both directions (Crusty ↔ Rust)
- **Round-Trip**: Transpiling from Crusty to Rust and back to Crusty
- **Syntax Stability**: The property that round-trip transpilation preserves program structure
- **Semantic Equivalence**: The property that transpiled code has identical behavior
- **Rust Parser**: Component that parses Rust source code into AST
- **Unified AST**: Single AST representation that can represent both Crusty and Rust code
- **Reverse Translation**: Converting Rust syntax to equivalent Crusty syntax
- **Syntax Mapping**: Rules for translating between Crusty and Rust syntax

## Requirements

### Requirement 1: Parse Rust Source Code

**User Story:** As a Crusty developer, I want to parse Rust source code, so that I can convert existing Rust projects to Crusty syntax.

#### Acceptance Criteria

1. WHEN the system receives a Rust source file, THE System SHALL parse it into an AST
2. THE System SHALL use the syn crate for Rust parsing
3. THE System SHALL convert syn AST to the unified AST representation
4. THE System SHALL preserve all Rust syntax constructs in the AST
5. WHEN parsing fails, THE System SHALL report Rust syntax errors with file locations

### Requirement 2: Convert Rust AST to Unified AST

**User Story:** As a system architect, I want a unified AST representation, so that both Crusty and Rust code can be processed uniformly.

#### Acceptance Criteria

1. THE System SHALL convert syn::File to unified File AST
2. THE System SHALL convert syn::Item to unified Item AST
3. THE System SHALL convert syn::Expr to unified Expression AST
4. THE System SHALL convert syn::Type to unified Type AST
5. THE System SHALL convert syn::Stmt to unified Statement AST
6. THE System SHALL preserve all semantic information during conversion
7. WHEN conversion encounters unsupported Rust features, THE System SHALL report an error

### Requirement 3: Generate Crusty Code from Rust

**User Story:** As a Crusty developer, I want to convert Rust code to Crusty syntax, so that I can work with existing Rust libraries using Crusty syntax.

#### Acceptance Criteria

1. WHEN the system has a Rust AST, THE System SHALL generate equivalent Crusty source code
2. THE System SHALL translate Rust function syntax to Crusty function syntax
3. THE System SHALL translate Rust match expressions to Crusty switch statements
4. THE System SHALL translate Rust impl blocks to Crusty struct methods
5. THE System SHALL translate Rust Type::method() to Crusty @Type.method()
6. THE System SHALL translate Rust macro_name! to Crusty __macro_name__
7. THE System SHALL translate Rust turbofish Type::<T> to Crusty Type(T)

### Requirement 4: Translate Rust-Specific Syntax

**User Story:** As a Crusty developer, I want Rust-specific syntax translated to Crusty equivalents, so that I can understand Rust code in familiar Crusty syntax.

#### Acceptance Criteria

1. THE System SHALL translate Rust 'label: loop to Crusty .label: loop
2. THE System SHALL translate Rust break 'label to Crusty break label
3. THE System SHALL translate Rust continue 'label to Crusty continue label
4. THE System SHALL translate Rust Result<T, E> to Crusty T?
5. THE System SHALL translate Rust ? operator to Crusty ? operator (pass through)
6. THE System SHALL translate Rust traits to Crusty VTable structs (when possible)
7. THE System SHALL translate Rust closures to Crusty nested functions

### Requirement 5: Support Round-Trip Transpilation

**User Story:** As a language designer, I want round-trip transpilation to work correctly, so that I can prove Crusty syntax is stable and complete.

#### Acceptance Criteria

1. WHEN Crusty code is transpiled to Rust and back to Crusty, THE System SHALL produce semantically equivalent code
2. WHEN Rust code is transpiled to Crusty and back to Rust, THE System SHALL produce semantically equivalent code
3. THE System SHALL preserve program structure during round-trip transpilation
4. THE System SHALL preserve comments during round-trip transpilation
5. THE System SHALL preserve documentation during round-trip transpilation
6. WHEN round-trip transpilation fails, THE System SHALL report which constructs are not supported

### Requirement 6: Handle Rust Standard Library

**User Story:** As a Crusty developer, I want to use Rust standard library types in Crusty syntax, so that I can leverage the full Rust ecosystem.

#### Acceptance Criteria

1. THE System SHALL recognize Rust std library types (Vec, String, Option, Result, etc.)
2. THE System SHALL translate Rust std library method calls to Crusty syntax
3. THE System SHALL preserve Rust std library imports in Crusty code
4. THE System SHALL translate Rust use statements to Crusty #import directives
5. THE System SHALL translate Rust pub use statements to Crusty #export directives

### Requirement 7: Support Rust Traits

**User Story:** As a Crusty developer, I want to work with Rust traits, so that I can use trait-based abstractions in Crusty code.

#### Acceptance Criteria

1. WHEN Rust code defines a trait, THE System SHALL translate it to a Crusty VTable struct (when possible)
2. WHEN Rust code implements a trait, THE System SHALL translate it to Crusty struct methods
3. WHEN Rust code uses trait objects (dyn Trait), THE System SHALL preserve them in Crusty
4. WHEN Rust code uses trait bounds, THE System SHALL preserve them in Crusty
5. WHEN a trait cannot be translated to VTable, THE System SHALL preserve it as __rust__{ trait ... }

### Requirement 8: Handle Rust Lifetimes

**User Story:** As a Crusty developer, I want Rust lifetime annotations to be handled correctly, so that I can work with borrowed references.

#### Acceptance Criteria

1. WHEN Rust code contains explicit lifetime annotations, THE System SHALL preserve them in Crusty
2. WHEN Rust code uses lifetime elision, THE System SHALL preserve the elided form in Crusty
3. THE System SHALL translate Rust lifetime syntax ('a) to Crusty lifetime syntax ('a)
4. THE System SHALL preserve lifetime bounds in generic parameters

### Requirement 9: Support Rust Macros

**User Story:** As a Crusty developer, I want to use Rust macros in Crusty code, so that I can leverage macro-based libraries.

#### Acceptance Criteria

1. THE System SHALL translate Rust macro invocations (macro_name!) to Crusty syntax (__macro_name__)
2. THE System SHALL preserve macro arguments and delimiters
3. THE System SHALL translate Rust macro_rules! definitions to Crusty #define macros (when possible)
4. WHEN a macro cannot be translated, THE System SHALL preserve it as __rust__{ macro_rules! ... }

### Requirement 10: Handle Rust Attributes

**User Story:** As a Crusty developer, I want Rust attributes to work in Crusty code, so that I can use derive macros and conditional compilation.

#### Acceptance Criteria

1. THE System SHALL preserve Rust #[attribute] syntax in Crusty (same syntax)
2. THE System SHALL preserve Rust #[derive(...)] attributes in Crusty
3. THE System SHALL preserve Rust #[cfg(...)] attributes in Crusty
4. THE System SHALL preserve Rust #[test] attributes in Crusty
5. THE System SHALL preserve all other Rust attributes in Crusty

### Requirement 11: Support Rust Modules

**User Story:** As a Crusty developer, I want to work with Rust module structure, so that I can organize code in the same way as Rust.

#### Acceptance Criteria

1. THE System SHALL translate Rust mod blocks to Crusty namespace blocks
2. THE System SHALL translate Rust mod declarations to Crusty namespace declarations
3. THE System SHALL preserve Rust module hierarchy in Crusty
4. THE System SHALL translate Rust use statements to Crusty #import directives
5. THE System SHALL translate Rust pub use statements to Crusty #export directives

### Requirement 12: Validate Round-Trip Correctness

**User Story:** As a quality assurance engineer, I want automated validation of round-trip transpilation, so that I can ensure correctness.

#### Acceptance Criteria

1. THE System SHALL provide a test mode that validates round-trip transpilation
2. WHEN testing round-trip, THE System SHALL compare ASTs before and after
3. WHEN testing round-trip, THE System SHALL report any semantic differences
4. THE System SHALL provide property-based tests for round-trip correctness
5. THE System SHALL test round-trip with a large corpus of Rust code

### Requirement 13: Handle Unsupported Rust Features

**User Story:** As a Crusty developer, I want clear error messages for unsupported Rust features, so that I know what cannot be translated.

#### Acceptance Criteria

1. WHEN Rust code uses async/await, THE System SHALL report that it's not yet supported
2. WHEN Rust code uses advanced trait features (GATs, etc.), THE System SHALL report limitations
3. WHEN Rust code uses procedural macros, THE System SHALL preserve them as __rust__{ }
4. THE System SHALL provide clear error messages explaining why features are unsupported
5. THE System SHALL suggest workarounds when possible

### Requirement 14: Support CLI for Bidirectional Transpilation

**User Story:** As a Crusty developer, I want command-line options for bidirectional transpilation, so that I can easily convert between Crusty and Rust.

#### Acceptance Criteria

1. THE System SHALL support --from-lang=rust option to specify Rust input
2. THE System SHALL support --to-lang=crusty option to specify Crusty output
3. THE System SHALL support --to-lang=rust option to specify Rust output (Phase 1 behavior)
4. THE System SHALL support --round-trip option to validate round-trip correctness
5. THE System SHALL report transpilation direction in verbose mode

### Requirement 15: Preserve Code Style

**User Story:** As a Crusty developer, I want generated Crusty code to follow Crusty style conventions, so that it's readable and maintainable.

#### Acceptance Criteria

1. WHEN generating Crusty code from Rust, THE System SHALL apply Crusty formatting rules
2. THE System SHALL use crustyfmt (Phase 2) to format generated Crusty code
3. THE System SHALL preserve comments in their original positions
4. THE System SHALL preserve documentation comments
5. THE System SHALL apply consistent indentation and spacing

## Success Criteria

Phase 3 is successful when:

1. ✅ Rust source code can be parsed into unified AST
2. ✅ Unified AST can generate both Crusty and Rust code
3. ✅ Round-trip transpilation preserves semantic meaning
4. ✅ Round-trip transpilation preserves program structure
5. ✅ All Rust standard library types work in Crusty syntax
6. ✅ Property-based tests validate round-trip correctness
7. ✅ Large corpus of Rust code can be transpiled to Crusty
8. ✅ Generated Crusty code is readable and idiomatic

## Out of Scope

The following features are explicitly out of scope for Phase 3:

- **Async/await**: Not supported in initial Phase 3 release
- **Procedural macros**: Preserved as __rust__{ } blocks, not translated
- **Advanced trait features**: GATs, associated type bounds, etc.
- **Const generics**: Preserved but not specially handled
- **Inline assembly**: Preserved as __asm__ blocks
- **FFI**: Preserved as extern blocks

These features may be added in future phases.

## Dependencies

Phase 3 depends on:

- **Phase 1**: Core transpilation infrastructure (parser, AST, code generator)
- **Phase 2**: crustyfmt for formatting generated Crusty code
- **syn crate**: For parsing Rust source code
- **prettyplease crate**: For formatting generated Rust code

## Risks and Mitigations

### Risk 1: Rust Syntax Complexity
**Risk**: Rust has complex syntax that may be difficult to translate to Crusty
**Mitigation**: Use __rust__{ } escape hatch for untranslatable constructs

### Risk 2: Semantic Differences
**Risk**: Some Rust constructs may not have direct Crusty equivalents
**Mitigation**: Extensive testing with property-based tests and real-world code

### Risk 3: Performance
**Risk**: Bidirectional transpilation may be slow
**Mitigation**: Optimize AST conversion, cache parsed results

### Risk 4: Maintenance Burden
**Risk**: Keeping up with Rust language changes
**Mitigation**: Use syn crate which tracks Rust syntax changes

## Testing Strategy

Phase 3 requires extensive testing:

1. **Unit Tests**: Test each Rust-to-Crusty translation rule
2. **Integration Tests**: Test end-to-end transpilation
3. **Property-Based Tests**: Validate round-trip correctness
4. **Corpus Testing**: Test with large corpus of real Rust code
5. **Regression Tests**: Ensure Phase 1 and Phase 2 still work

## Timeline

Phase 3 is estimated to take 60-80 hours of development time:

- Rust parser integration: 10-15 hours
- AST conversion: 15-20 hours
- Crusty code generation: 15-20 hours
- Round-trip validation: 10-15 hours
- Testing and polish: 10-15 hours
