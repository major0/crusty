# Namespaces

## Introduction

Crusty groups related items into named modules with the `namespace` keyword. A
namespace maps directly to a Rust `mod`.

## Rationale

`namespace` is the familiar C++/C-family keyword for grouping declarations under
a name. Crusty reuses it rather than Rust's `mod` so the surface stays C-flavored,
while the generated code is an ordinary Rust module.

## Syntax

```c
namespace math {
    int add(int a, int b) {
        return a + b;
    }

    static int helper(int x) {
        return x * 2;
    }
}
```

Translates to:

```rust
pub mod math {
    pub fn add(a: i32, b: i32) -> i32 {
        return a + b;
    }

    fn helper(x: i32) -> i32 {
        return x * 2;
    }
}
```

## Visibility

A namespace lowers to a **public** module (`pub mod`). Visibility of the items
*inside* follows the usual Crusty rule: a `static` item is private (no `pub`), a
non-`static` item is public. In the example above `add` is `pub fn` and the
`static` `helper` is a private `fn`.

## Nesting

Namespaces nest. Each level produces a nested Rust module:

```c
namespace outer {
    namespace inner {
        int deep(int n) {
            return n;
        }
    }
}
```

```rust
pub mod outer {
    pub mod inner {
        pub fn deep(n: i32) -> i32 {
            return n;
        }
    }
}
```

An empty namespace (`namespace empty { }`) is valid and emits an empty module.

## Contents

A namespace may contain any top-level item: functions, structs, enums,
typedefs, `#define` constants, `#macro` definitions, `#ifdef`/`#ifndef` blocks,
and further namespaces.

## Formal Grammar

```ebnf
namespace = "namespace" IDENT "{" item* "}" ;
```
