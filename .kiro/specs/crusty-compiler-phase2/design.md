# Design Document: Crusty Compiler Phase 2

## Overview

Phase 2 implements the essential tooling ecosystem for Crusty: **crustydoc** (documentation generator) and **crustyfmt** (code formatter). These tools complete the developer experience by providing professional documentation generation and consistent code formatting capabilities.

## Architecture

### High-Level Components

```
┌─────────────────────────────────────────────────────────────┐
│                     Phase 2 Tooling                          │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌──────────────────┐              ┌──────────────────┐     │
│  │   crustydoc      │              │   crustyfmt      │     │
│  │                  │              │                  │     │
│  │  - Parse Crusty  │              │  - Parse Crusty  │     │
│  │  - Extract Docs  │              │  - Format AST    │     │
│  │  - Generate HTML │              │  - Apply Rules   │     │
│  │  - Cross-refs    │              │  - Preserve Docs │     │
│  └──────────────────┘              └──────────────────┘     │
│           │                                  │               │
│           └──────────────┬───────────────────┘               │
│                          │                                   │
│                          ▼                                   │
│              ┌───────────────────────┐                       │
│              │   Shared Components   │                       │
│              │                       │                       │
│              │  - Parser (Phase 1)   │                       │
│              │  - AST (Phase 1)      │                       │
│              │  - Pretty Printer     │                       │
│              └───────────────────────┘                       │
└─────────────────────────────────────────────────────────────┘
```

## Component 1: crustydoc (Documentation Generator)

### Purpose
Generate HTML documentation from Crusty source code with documentation comments, similar to rustdoc.

### Design Approach

**Leverage rustdoc**: Instead of reimplementing HTML generation, crustydoc will:
1. Parse Crusty source code (reusing Phase 1 parser)
2. Extract documentation comments from AST
3. Transpile to Rust (reusing Phase 1 code generator)
4. Invoke rustdoc on generated Rust code
5. Map any errors back to Crusty source locations

This approach provides:
- Professional HTML output matching Rust ecosystem standards
- Automatic cross-referencing and search functionality
- Syntax highlighting for code examples
- Minimal maintenance burden (rustdoc handles HTML generation)

### Architecture

```
Crusty Source (.crst)
        │
        ▼
    Parser (Phase 1)
        │
        ▼
    AST with Doc Comments
        │
        ├─────────────────┐
        │                 │
        ▼                 ▼
  Code Generator    Doc Extractor
  (Phase 1)         (New)
        │                 │
        ▼                 ▼
  Rust Source       Doc Metadata
        │                 │
        └────────┬────────┘
                 │
                 ▼
            rustdoc
                 │
                 ▼
         HTML Documentation
```

### Key Components

#### 1. Documentation Comment Parser (Already in Phase 1)
- **Status**: ✅ Already implemented in lexer/parser
- **Location**: `src/lexer.rs`, `src/parser.rs`
- **Functionality**: Recognizes `///` and `/** */` comments, attaches to AST nodes

#### 2. Documentation Extractor (New)
- **Purpose**: Extract and validate documentation from AST
- **Responsibilities**:
  - Traverse AST to find documented items
  - Validate documentation completeness
  - Check for broken cross-references
  - Generate documentation metadata

#### 3. crustydoc CLI (New)
- **Purpose**: Command-line interface for documentation generation
- **Options**:
  - `crustydoc <file.crst>` - Generate docs for single file
  - `--output <dir>` - Specify output directory
  - `--open` - Open documentation in browser after generation
  - `-D missing-docs` - Warn about missing documentation
  - `--document-private-items` - Include private items in documentation
  - `-- <rustdoc-options>` - Pass additional options to rustdoc

#### 4. Error Mapper (New)
- **Purpose**: Map rustdoc errors back to Crusty source locations
- **Responsibilities**:
  - Parse rustdoc error messages
  - Map Rust line numbers to Crusty line numbers
  - Rewrite error messages with Crusty file paths

### Documentation Comment Format

Crusty uses the same documentation comment syntax as Rust:

```crusty
/// This is a line documentation comment
/// It can span multiple lines
///
/// # Examples
///
/// ```crusty
/// int x = add(5, 3);
/// ```
int add(int a, int b) {
    return a + b;
}

/**
 * This is a block documentation comment
 * It also supports Markdown
 */
struct Point {
    int x;
    int y;
}
```

### Implementation Strategy

1. **Reuse Phase 1 Infrastructure**:
   - Parser already handles doc comments
   - AST already stores doc comments
   - Code generator already preserves doc comments in Rust output

2. **Add Documentation Validation**:
   - Check for missing documentation on public items
   - Validate cross-references
   - Check code examples compile

3. **Invoke rustdoc**:
   - Generate Rust code with preserved doc comments
   - Call rustdoc with appropriate flags
   - Capture and map errors

4. **Error Mapping**:
   - Track line number mappings during transpilation
   - Parse rustdoc error output
   - Rewrite errors with Crusty locations

## Component 2: crustyfmt (Code Formatter)

### Purpose
Automatically format Crusty source code according to consistent style guidelines, similar to rustfmt.

### Design Approach

**Use Pretty Printer**: crustyfmt will:
1. Parse Crusty source code into AST
2. Apply formatting rules to AST
3. Use Pretty Printer to generate formatted source
4. Verify round-trip correctness (parse → format → parse)

### Architecture

```
Crusty Source (.crst)
        │
        ▼
    Parser (Phase 1)
        │
        ▼
       AST
        │
        ▼
  Format Rules
  Application
        │
        ▼
  Pretty Printer
  (Phase 1)
        │
        ▼
  Formatted Crusty Source
```

### Key Components

#### 1. Pretty Printer (Already in Phase 1)
- **Status**: ✅ Already implemented
- **Location**: `src/pretty.rs`
- **Functionality**: Converts AST back to Crusty source code

#### 2. Format Rules Engine (New)
- **Purpose**: Apply formatting rules to AST before pretty printing
- **Responsibilities**:
  - Normalize indentation
  - Normalize spacing around operators
  - Normalize line breaks
  - Apply brace style rules

#### 3. Configuration Loader (New)
- **Purpose**: Load formatting configuration from `.crustyfmt.toml`
- **Default Configuration**:
  ```toml
  indent_width = 4
  use_tabs = false
  max_line_length = 100
  brace_style = "same_line"  # or "next_line"
  space_before_brace = true
  space_after_comma = true
  ```

#### 4. crustyfmt CLI (New)
- **Purpose**: Command-line interface for code formatting
- **Options**:
  - `crustyfmt <file.crst>` - Format file in-place
  - `crustyfmt <dir>` - Format all .crst files in directory
  - `--check` - Check if files are formatted (CI mode)
  - `--config <file>` - Use custom configuration file
  - `--stdin` - Read from stdin, write to stdout (editor integration)

### Formatting Rules

#### Indentation
- Default: 4 spaces per level
- Configurable: spaces or tabs
- Consistent throughout file

#### Spacing
- Space after comma: `func(a, b, c)`
- Space around binary operators: `x + y`
- No space for unary operators: `!flag`, `-value`
- Space before opening brace: `if (condition) {`

#### Line Breaking
- Maximum line length (default 100 characters)
- Break long function signatures
- Break long parameter lists
- Break long expressions

#### Brace Style
- Same line (default): `if (x) {`
- Next line (optional): 
  ```crusty
  if (x)
  {
  ```

### Implementation Strategy

1. **Enhance Pretty Printer**:
   - Add formatting options to Pretty Printer
   - Support configurable indentation
   - Support configurable spacing
   - Support configurable line breaking

2. **Add Configuration System**:
   - Define FormatConfig struct
   - Implement TOML parsing
   - Apply defaults when no config file

3. **Implement Format Rules**:
   - Normalize whitespace in AST
   - Apply indentation rules
   - Apply spacing rules
   - Apply line breaking rules

4. **Add CLI**:
   - Parse command-line arguments
   - Support in-place formatting
   - Support check mode (--check)
   - Support stdin/stdout mode

5. **Verify Correctness**:
   - Parse formatted output
   - Compare AST before and after
   - Ensure semantic equivalence

## Integration with Phase 1

### Shared Components

Both crustydoc and crustyfmt reuse Phase 1 infrastructure:

1. **Parser** (`src/parser.rs`):
   - Already handles documentation comments
   - Already builds complete AST
   - No changes needed

2. **AST** (`src/ast.rs`):
   - Already stores doc comments on items
   - Already represents all Crusty syntax
   - No changes needed

3. **Code Generator** (`src/codegen.rs`):
   - Already preserves doc comments in Rust output
   - Used by crustydoc to generate Rust for rustdoc
   - No changes needed

4. **Pretty Printer** (`src/pretty.rs`):
   - Already converts AST to Crusty source
   - Used by crustyfmt for formatting
   - Needs enhancement for formatting rules

### New Modules

Phase 2 adds these new modules:

1. **`src/crustydoc.rs`**:
   - Documentation extraction
   - rustdoc invocation
   - Error mapping

2. **`src/crustyfmt.rs`**:
   - Format rules engine
   - Configuration loading
   - Formatting application

3. **`src/bin/crustydoc.rs`**:
   - crustydoc CLI entry point

4. **`src/bin/crustyfmt.rs`**:
   - crustyfmt CLI entry point

## Testing Strategy

### crustydoc Tests

#### Unit Tests
- Test documentation extraction from AST
- Test rustdoc invocation
- Test error mapping
- Test configuration parsing

#### Integration Tests
- Test end-to-end documentation generation
- Test with various Crusty syntax features
- Test cross-references
- Test code examples

#### Property-Based Tests
- **Property 1**: All public items with doc comments appear in generated documentation
- **Property 2**: Generated HTML is valid
- **Property 3**: Cross-references resolve correctly

### crustyfmt Tests

#### Unit Tests
- Test format rules application
- Test configuration loading
- Test pretty printer enhancements
- Test check mode

#### Integration Tests
- Test end-to-end formatting
- Test with various Crusty syntax features
- Test configuration options
- Test stdin/stdout mode

#### Property-Based Tests
- **Property 4**: Formatting preserves semantic meaning (parse → format → parse yields same AST)
- **Property 5**: Formatting is idempotent (format → format yields same output)
- **Property 6**: All comments are preserved

## Correctness Properties

### crustydoc Properties

**Property 1: Documentation Completeness**
```
For all public items with doc comments:
  generated_html contains documentation for item
```
**Validates: Requirements 1, 2, 10**

**Property 2: Cross-Reference Validity**
```
For all type references in documentation:
  cross_reference resolves to valid item OR external documentation
```
**Validates: Requirements 4**

**Property 3: Code Example Validity**
```
For all code examples in documentation:
  example compiles successfully OR is marked as no_run
```
**Validates: Requirements 11**

### crustyfmt Properties

**Property 4: Semantic Preservation**
```
For all Crusty source files:
  AST(source) == AST(format(source))
```
**Validates: Requirements 5, 7**

**Property 5: Idempotence**
```
For all Crusty source files:
  format(source) == format(format(source))
```
**Validates: Requirements 5**

**Property 6: Comment Preservation**
```
For all comments in source:
  format(source) contains comment at correct position
```
**Validates: Requirements 7**

## Success Criteria

### crustydoc Success Criteria
- ✅ Generates HTML documentation for all public items
- ✅ Preserves Markdown formatting in doc comments
- ✅ Creates cross-references between items
- ✅ Includes syntax-highlighted code examples
- ✅ Provides search functionality (via rustdoc)
- ✅ Maps errors back to Crusty source locations
- ✅ Integrates with build.rs

### crustyfmt Success Criteria
- ✅ Formats all Crusty syntax correctly
- ✅ Preserves all comments and documentation
- ✅ Applies consistent indentation and spacing
- ✅ Supports configurable formatting rules
- ✅ Provides check mode for CI/CD
- ✅ Supports stdin/stdout for editor integration
- ✅ Formatting is idempotent and semantics-preserving

## Performance Considerations

### crustydoc Performance
- **Bottleneck**: rustdoc invocation (external process)
- **Optimization**: Cache transpiled Rust code
- **Target**: < 5 seconds for typical project

### crustyfmt Performance
- **Bottleneck**: Parsing and pretty printing
- **Optimization**: Parallel processing for multiple files
- **Target**: < 100ms per file

## Backward Compatibility

Phase 2 is fully backward compatible with Phase 1:
- No changes to Crusty syntax
- No changes to transpilation behavior
- No changes to existing CLI options
- New tools are optional additions

## Dependencies

### External Crates
- **syn**: Already used in Phase 1 for Rust parsing
- **prettyplease**: Already used in Phase 1 for Rust formatting
- **toml**: Already used in Phase 1 for config parsing
- **clap**: Already used in Phase 1 for CLI parsing

### Internal Dependencies
- **Phase 1 Parser**: Required for parsing Crusty source
- **Phase 1 AST**: Required for representing Crusty code
- **Phase 1 Code Generator**: Required for generating Rust code
- **Phase 1 Pretty Printer**: Required for formatting Crusty code

## Implementation Phases

### Phase 2.1: crustydoc Foundation
1. Create crustydoc module structure
2. Implement documentation extraction
3. Implement rustdoc invocation
4. Implement basic error mapping

### Phase 2.2: crustydoc Features
1. Add configuration support
2. Add validation (missing docs, broken refs)
3. Add code example testing
4. Add CLI options

### Phase 2.3: crustyfmt Foundation
1. Enhance Pretty Printer with formatting rules
2. Implement configuration loading
3. Implement format rules engine
4. Add CLI

### Phase 2.4: crustyfmt Features
1. Add check mode
2. Add stdin/stdout mode
3. Add directory formatting
4. Add pre-commit hook integration

### Phase 2.5: Testing and Polish
1. Write comprehensive unit tests
2. Write integration tests
3. Write property-based tests
4. Update documentation
5. Add examples

## Future Enhancements (Post-Phase 2)

These features are out of scope for Phase 2 but may be added later:

- **crustydoc**:
  - Custom HTML themes
  - PDF documentation generation
  - API stability tracking
  - Documentation coverage reports

- **crustyfmt**:
  - Auto-fix for common issues
  - Import sorting
  - Comment reflowing
  - Custom format rules via plugins
