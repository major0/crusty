# Architecture Documentation

Documentation for the Crusty compiler architecture and internal components.

## Topics

- [Overview](overview.md) - Compiler pipeline and design principles
- [Lexer](lexer.md) - Tokenization of Crusty source code
- [Parser](parser.md) - Parsing tokens into an Abstract Syntax Tree
- [Semantic Analyzer](semantic.md) - Type checking and validation
- [Code Generator](codegen.md) - Emitting Rust source from the AST
- [Build Integration](build-integration.md) - Cargo and build.rs integration
- [Error Handling](error-handling.md) - Compiler error hierarchy and reporting
- [Tooling](tooling.md) - crustydoc and crustyfmt developer tools
- [CI/CD](ci-cd.md) - Modular CI/CD pipeline architecture
