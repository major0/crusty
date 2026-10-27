# Syntax Documentation

Documentation for the Crusty programming language syntax.

Crusty provides a C-like syntax that transpiles to Rust, giving developers familiar C conventions while leveraging Rust's safety guarantees and ecosystem.

## Topics

- [Functions](functions.md) - Function declarations, parameters, and return types
- [Variables](variables.md) - Variable declarations, mutability, and scoping
- [Control Flow](control-flow.md) - If/else, loops, switch, break, continue
- [Types](types.md) - Primitive types, structs, enums, pointers, references
- [Expressions](expressions.md) - Operators, precedence, and special expressions
- [Typedef](typedef.md) - Typedef syntax for structs, impl blocks, and trait implementations
- [Nested and Anonymous Functions](nested-functions.md) - Hoisted nested `fn` items and capturing `__func__` anonymous functions
- [Error Handling](error-handling.md) - Fallible types, error propagation, and Result mapping
- [Concurrency](concurrency.md) - The `__thread__` spawn primitive; async via the `__rust__` escape hatch
- [Macros](macros.md) - #define macros, delimiter types, and double-underscore naming
- [Attributes](attributes.md) - Verbatim pass-through of `#[ … ]` outer and `#![ … ]` inner attributes
- [Modules](modules.md) - #import and #export directives for module management
- [Namespaces](namespaces.md) - Grouping items into named modules
- [Static Variables](statics.md) - Top-level static globals and mutability
- [Extern Blocks](extern-blocks.md) - FFI declarations and extern block syntax
- [Transformation Summary](transformations.md) - At-a-glance map of every Crusty form to its emitted Rust
