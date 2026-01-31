# Requirements Document: Crusty Compiler Phase 2

## Introduction

Phase 2 of the Crusty compiler project focuses on implementing essential tooling to complete the Crusty programming language ecosystem. Building on Phase 1's core transpilation capabilities, Phase 2 introduces two critical tools: crustydoc (documentation generator) and crustyfmt (code formatter). These tools enable Crusty to function as a complete, production-ready programming language with professional development workflows.

## Glossary

- **Crusty**: A C-like programming language that transpiles to Rust
- **crustydoc**: Documentation generator for Crusty code (analogous to rustdoc)
- **crustyfmt**: Code formatter for Crusty source files (analogous to rustfmt)
- **Doc_Comment**: Documentation comments in Crusty source code (/// or /**)
- **HTML_Output**: Generated HTML documentation files
- **AST**: Abstract Syntax Tree representation of Crusty code
- **Pretty_Printer**: Component that formats AST back to source code
- **Format_Rule**: Configurable formatting guideline for code style
- **Cross_Reference**: Hyperlink between documentation pages
- **Module_Doc**: Documentation for a Crusty module or namespace
- **Item_Doc**: Documentation for functions, structs, enums, or type aliases
- **Syntax_Highlighting**: Color-coded display of code examples in documentation

## Requirements

### Requirement 1: Parse Documentation Comments

**User Story:** As a Crusty developer, I want to write documentation comments in my code, so that I can document my APIs for other developers.

#### Acceptance Criteria

1. WHEN a source file contains triple-slash comments (///), THE Parser SHALL recognize them as documentation comments
2. WHEN a source file contains block documentation comments (/** */), THE Parser SHALL recognize them as documentation comments
3. WHEN documentation comments precede an item declaration, THE Parser SHALL associate them with that item
4. THE Parser SHALL preserve documentation comment text and formatting in the AST
5. WHEN documentation comments contain Markdown syntax, THE Parser SHALL preserve the Markdown unchanged

### Requirement 2: Generate HTML Documentation

**User Story:** As a Crusty developer, I want to generate HTML documentation from my code, so that users can browse my API documentation in a web browser.

#### Acceptance Criteria

1. WHEN crustydoc processes a Crusty source file, THE System SHALL generate HTML documentation files
2. WHEN generating documentation, THE System SHALL create an index page listing all documented items
3. WHEN generating documentation, THE System SHALL create individual pages for each module
4. WHEN generating documentation, THE System SHALL create individual pages for each struct, enum, and type alias
5. THE System SHALL render Markdown in documentation comments as formatted HTML
6. THE System SHALL include syntax-highlighted code examples in the documentation
7. WHEN generating documentation, THE System SHALL create a search interface for finding items

### Requirement 3: Support Module Documentation

**User Story:** As a Crusty developer, I want to document entire modules, so that I can provide overview information about module purpose and usage.

#### Acceptance Criteria

1. WHEN a source file contains module-level documentation comments, THE System SHALL associate them with the module
2. WHEN generating module documentation, THE System SHALL display module-level comments at the top of the module page
3. THE System SHALL list all public items within a module on the module documentation page
4. WHEN a module contains sub-modules, THE System SHALL link to sub-module documentation pages

### Requirement 4: Generate Cross-References

**User Story:** As a documentation reader, I want to click on type names and function names, so that I can navigate to their documentation.

#### Acceptance Criteria

1. WHEN documentation references a type name, THE System SHALL create a hyperlink to that type's documentation
2. WHEN documentation references a function name, THE System SHALL create a hyperlink to that function's documentation
3. WHEN a type is defined in a different module, THE System SHALL create a hyperlink to the correct module page
4. THE System SHALL handle cross-references to standard library types by linking to Rust documentation

### Requirement 5: Format Crusty Source Code

**User Story:** As a Crusty developer, I want to automatically format my code, so that my codebase maintains consistent style.

#### Acceptance Criteria

1. WHEN crustyfmt processes a Crusty source file, THE System SHALL format the code according to style guidelines
2. THE System SHALL preserve all comments and documentation in their original positions
3. THE System SHALL apply consistent indentation using spaces or tabs based on configuration
4. THE System SHALL apply consistent spacing around operators and delimiters
5. THE System SHALL apply consistent line breaking for long statements
6. WHEN formatting is applied, THE System SHALL preserve the semantic meaning of the code

### Requirement 6: Support Configurable Formatting Rules

**User Story:** As a Crusty developer, I want to configure formatting rules, so that I can match my team's coding standards.

#### Acceptance Criteria

1. THE System SHALL read formatting configuration from a .crustyfmt.toml file
2. THE System SHALL support configuration for indentation width (spaces per level)
3. THE System SHALL support configuration for maximum line length
4. THE System SHALL support configuration for brace style (same line vs new line)
5. THE System SHALL support configuration for space vs tab indentation
6. WHEN no configuration file exists, THE System SHALL use default formatting rules

### Requirement 7: Preserve Comments During Formatting

**User Story:** As a Crusty developer, I want my comments to remain intact after formatting, so that I don't lose important code documentation.

#### Acceptance Criteria

1. WHEN formatting code, THE System SHALL preserve all line comments in their original positions
2. WHEN formatting code, THE System SHALL preserve all block comments in their original positions
3. WHEN formatting code, THE System SHALL preserve all documentation comments in their original positions
4. THE System SHALL maintain the relative position of comments with respect to code elements

### Requirement 8: Integrate with Build Tools

**User Story:** As a Crusty developer, I want to integrate documentation generation into my build process, so that documentation stays up-to-date automatically.

#### Acceptance Criteria

1. THE System SHALL provide a command-line interface for crustydoc
2. THE System SHALL provide a command-line interface for crustyfmt
3. WHEN invoked from build.rs, THE System SHALL generate documentation for all source files
4. WHEN invoked from build.rs, THE System SHALL format all source files
5. THE System SHALL exit with appropriate status codes for success and failure
6. THE System SHALL report errors with file names and line numbers

### Requirement 9: Handle Crusty-Specific Syntax

**User Story:** As a Crusty developer, I want documentation and formatting tools to understand Crusty syntax, so that they work correctly with Crusty-specific features.

#### Acceptance Criteria

1. THE System SHALL correctly parse and format type-scoped calls with @ prefix (@Type.method())
2. THE System SHALL correctly parse and format macro invocations with double-underscore naming (__macro__())
3. THE System SHALL correctly parse and format labeled loops with dot prefix (.label:)
4. THE System SHALL correctly parse and format explicit generic parameters with parentheses/brackets syntax
5. THE System SHALL correctly parse and format nested functions (closures)
6. THE System SHALL correctly parse and format typedef declarations
7. THE System SHALL correctly parse and format implementation blocks

### Requirement 10: Generate Documentation for Special Items

**User Story:** As a Crusty developer, I want documentation for all language constructs, so that users understand how to use every feature.

#### Acceptance Criteria

1. WHEN documenting a struct, THE System SHALL list all fields with their types and documentation
2. WHEN documenting a struct, THE System SHALL list all methods in implementation blocks
3. WHEN documenting an enum, THE System SHALL list all variants with their documentation
4. WHEN documenting a type alias, THE System SHALL show the aliased type
5. WHEN documenting a function, THE System SHALL show the function signature with parameter names and types
6. WHEN documenting a function, THE System SHALL show the return type

### Requirement 11: Support Code Examples in Documentation

**User Story:** As a documentation reader, I want to see code examples, so that I can understand how to use the API.

#### Acceptance Criteria

1. WHEN documentation contains code blocks, THE System SHALL render them with syntax highlighting
2. THE System SHALL support both Crusty and Rust code examples in documentation
3. WHEN a code example is marked as Crusty, THE System SHALL apply Crusty syntax highlighting
4. WHEN a code example is marked as Rust, THE System SHALL apply Rust syntax highlighting

### Requirement 12: Format Check Mode

**User Story:** As a CI/CD pipeline maintainer, I want to check if code is formatted correctly without modifying files, so that I can enforce formatting in pull requests.

#### Acceptance Criteria

1. WHEN crustyfmt is invoked with --check flag, THE System SHALL verify formatting without modifying files
2. WHEN code is correctly formatted, THE System SHALL exit with status code 0
3. WHEN code is incorrectly formatted, THE System SHALL exit with non-zero status code
4. WHEN code is incorrectly formatted, THE System SHALL report which files need formatting
5. THE System SHALL not write to any files when --check flag is used
