# Module System

## Introduction

Crusty uses `#import` and `#export` directives for module management, replacing the earlier `#use` syntax. These directives map directly to Rust's `use` statements with appropriate visibility modifiers.

## Rationale

Separating import and export into distinct directives makes visibility explicit at the point of declaration. `#import` brings modules into the current scope privately, while `#export` re-exports symbols publicly. This avoids the ambiguity of a single `#use` directive that required a `static` keyword to control visibility.

The `#include` directive from C is explicitly rejected — Crusty uses Rust's module system underneath.

## Import Directive

`#import` brings a module or symbol into the current scope as a private import.

```c
#import std.collections.HashMap
#import std.io
#import std.fs
#import mymodule
```

These translate to Rust `use` statements:
```rust
use std::collections::HashMap;
use std::io;
use std::fs;
use mymodule;
```

## Export Directive

`#export` re-exports a symbol publicly from the current module.

```c
#export std.collections.HashMap
#export mymodule.method
```

These translate to Rust `pub use` statements:
```rust
pub use std::collections::HashMap;
pub use mymodule::method;
```

## Examples

```c
// Import standard library modules
#import std.collections.HashMap
#import std.io
#import std.fs

// Re-export symbols for the module's public API
#export mylib.Config
#export mylib.run

int main() {
    var map = HashMap.new();
    return 0;
}
```

## Rejected Directives

| Directive | Error Message |
|-----------|--------------|
| `#include` | Use `#import` instead |
| `#use` | Use `#import` or `#export` instead |

## Formal Grammar

```
import_directive = "#import" module_path
export_directive = "#export" module_path "." symbol
module_path      = identifier ("." identifier)*
```
